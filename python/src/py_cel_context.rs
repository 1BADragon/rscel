use crate::{py_cel_program::PyCelProgram, py_cel_value::PyCelValue};

use super::py_bind_context::PyBindContext;
use pyo3::{
    exceptions::PyValueError, pyclass, pymethods, IntoPyObjectExt, PyObject, PyRefMut, PyResult,
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
            Some(p) => Ok(slf.ctx.add_program(name, p.clone())),
            None => Err(PyValueError::new_err("Program not populated")),
        }
    }

    pub fn exec(
        mut slf: PyRefMut<'_, Self>,
        name: &str,
        bindings: &PyBindContext,
    ) -> PyResult<PyObject> {
        let mut bindctx = BindContext::new();

        for (key, val) in bindings.bindings().iter() {
            bindctx.bind_param(&key, val.clone());
        }

        for (key, val) in bindings.funcs().iter() {
            bindctx.bind_func(&key, val);
        }

        match slf.ctx.exec(name, &bindctx) {
            Ok(val) => Ok(PyCelValue::new(val)
                .into_pyobject_or_pyerr(slf.py())?
                .unbind()),
            Err(err) => Err(PyValueError::new_err(err.to_string())),
        }
    }
}
