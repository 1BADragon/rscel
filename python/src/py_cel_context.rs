use crate::{
    py_cel_program::PyCelProgram, py_cel_value::PyCelValue, py_interp_runtime::PyInterpRuntime,
};

use super::py_bind_context::PyBindContext;
use pyo3::{
    exceptions::PyValueError, pyclass, pymethods, IntoPyObjectExt, Py, PyAny, PyRef, PyRefMut,
    PyResult,
};
use rscel::{BindContext, CelContext};

#[pyclass(name = "CelContext")]
pub struct PyCelContext {
    ctx: CelContext,
}

#[pymethods]
impl PyCelContext {
    #[new]
    pub fn new() -> PyCelContext {
        PyCelContext {
            ctx: CelContext::new(),
        }
    }

    pub fn add_program_str(mut slf: PyRefMut<'_, Self>, name: &str, prog: &str) -> PyResult<()> {
        if let Err(err) = slf.ctx.add_program_str(name, prog) {
            Err(PyValueError::new_err(err.to_string()))
        } else {
            Ok(())
        }
    }

    pub fn add_program(
        mut slf: PyRefMut<'_, Self>,
        name: &str,
        prog: PyRefMut<PyCelProgram>,
    ) -> PyResult<()> {
        match prog.as_inner() {
            Some(p) => {
                slf.ctx.add_program(name, p.clone());
                Ok(())
            }
            None => Err(PyValueError::new_err("Program not populated")),
        }
    }

    #[pyo3(signature = (name, bindings, *, runtime=None))]
    pub fn exec(
        mut slf: PyRefMut<'_, Self>,
        name: &str,
        bindings: &PyBindContext,
        runtime: Option<PyRef<'_, PyInterpRuntime>>,
    ) -> PyResult<Py<PyAny>> {
        let mut bindctx = BindContext::new();

        for (key, val) in bindings.bindings().iter() {
            bindctx.bind_param(key, val.clone());
        }

        // bind_func borrows, so the closures must outlive bindctx's use below.
        let mut funcs = Vec::with_capacity(bindings.funcs().len());
        for (key, callable) in bindings.funcs().iter() {
            funcs.push((key, callable.clone().into_fn()));
        }
        for (key, func) in funcs.iter() {
            bindctx.bind_func(key, func);
        }

        let runtime = runtime.map(|rt| rt.inner().clone());

        match slf.ctx.exec_with_rt(name, &bindctx, runtime).0 {
            Ok(val) => Ok(PyCelValue::new(val)
                .into_pyobject_or_pyerr(slf.py())?
                .unbind()),
            Err(err) => Err(PyValueError::new_err(err.to_string())),
        }
    }
}
