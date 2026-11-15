use py_cel_error::PyCelError;
use py_cel_value::PyCelValue;
use rscel::{BindContext, CelContext, CelValue, Program};

use pyo3::{
    exceptions::PyException,
    prelude::*,
    types::{PyDict, PyString},
    IntoPyObjectExt,
};

mod cel_py_object;
mod celpycallable;
mod frompyobject;
mod py_bind_context;
mod py_cel_context;
mod py_cel_error;
mod py_cel_program;
mod py_cel_value;
mod py_interp_runtime;

use celpycallable::CelPyCallable;
use py_bind_context::PyBindContext;
use py_cel_context::PyCelContext;
use py_cel_program::PyCelProgram;
use py_interp_runtime::PyInterpRuntime;
use rscel_to_sql::ToSqlBuilder;

/* Eval entry point */
#[pyfunction]
#[pyo3(signature = (prog_str, bindings, *, runtime=None))]
fn eval(
    py: Python<'_>,
    prog_str: String,
    bindings: &Bound<PyDict>,
    runtime: Option<PyRef<'_, PyInterpRuntime>>,
) -> PyResult<Py<PyAny>> {
    let callables = {
        let mut callables = Vec::new();
        for keyobj in bindings.keys() {
            let key = keyobj.cast::<PyString>()?;
            let val = bindings.get_item(key).unwrap().unwrap().clone();

            if val.is_callable() {
                callables.push((key.to_str()?.to_string(), CelPyCallable::new(val.into())));
            }
        }
        callables
    };
    let mut ctx = CelContext::new();
    let mut exec_ctx = BindContext::new();

    if let Err(e) = ctx.add_program_str("entry", &prog_str) {
        return Err(PyCelError::new(e).into());
    }

    for keyobj in bindings.keys() {
        let key = keyobj.cast::<PyString>()?;

        let val = bindings.get_item(key).unwrap().unwrap();

        if !val.is_callable() {
            exec_ctx.bind_param(key.to_str()?, val.extract::<PyCelValue>()?.into_inner())
        }
    }

    // bind_func borrows, so the closures must outlive exec_ctx's use below.
    let mut funcs = Vec::with_capacity(callables.len());
    for (name, callable) in callables {
        funcs.push((name, callable.into_fn()));
    }
    for (name, func) in funcs.iter() {
        exec_ctx.bind_func(name, func);
    }

    let res = ctx
        .exec_with_rt("entry", &exec_ctx, runtime.map(|rt| rt.inner().clone()))
        .0;

    match res {
        Ok(res) => Ok(PyCelValue::new(res).into_pyobject_or_pyerr(py)?.unbind()),
        Err(err) => Err(PyException::new_err(err.to_string())),
    }
}

#[pyfunction]
fn to_sql(py: Python<'_>, prog_str: String) -> PyResult<Py<PyString>> {
    let p = match Program::from_source(&prog_str) {
        Ok(p) => p,
        Err(err) => return Err(PyException::new_err(err.to_string())),
    };

    let sql_str = if let Some(ast) = p.ast() {
        match ast.to_sql_builder() {
            Ok(b) => b.to_sql(),
            Err(e) => return Err(PyException::new_err(e.to_string())),
        }
    } else {
        return Err(PyException::new_err("Internal Error".to_string()));
    };

    match sql_str {
        Ok(s) => Ok(PyString::new(py, &s).unbind()),
        Err(e) => Err(PyException::new_err(e.to_string())),
    }
}

/* Module decl */
#[pymodule]
#[pyo3(name = "rscel")]
fn rscel_module(_py: Python<'_>, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(eval, m)?)?;
    m.add_function(wrap_pyfunction!(to_sql, m)?)?;
    m.add_class::<PyCelContext>()?;
    m.add_class::<PyBindContext>()?;
    m.add_class::<PyCelProgram>()?;
    m.add_class::<PyInterpRuntime>()?;
    Ok(())
}
