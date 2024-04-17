#![feature(fn_traits)]
#![feature(unboxed_closures)]
use py_cel_error::PyCelError;
use py_cel_value::PyCelValue;
use rscel::{BindContext, CelContext, CelValue};

use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
    types::{PyDict, PyString},
};
use std::collections::HashMap;

mod cel_py_object;
mod celpycallable;
mod frompyobject;
mod py_cel_error;
mod py_cel_value;

use celpycallable::CelPyCallable;

/* Eval entry point */
#[pyfunction]
fn eval(py: Python<'_>, prog_str: String, bindings: &PyDict) -> PyResult<PyObject> {
    let callables = {
        let mut callables = Vec::new();
        for keyobj in bindings.keys().iter() {
            let key = keyobj.downcast::<PyString>()?;
            let val = bindings.get_item(keyobj).unwrap().unwrap();

            if val.is_callable() {
                callables.push((key.to_str()?, CelPyCallable::new(val.into())));
            }
        }
        callables
    };
    let mut ctx = CelContext::new();
    let mut exec_ctx = BindContext::new();

    if let Err(e) = ctx.add_program_str("entry", &prog_str) {
        return Err(PyCelError::new(e).into());
    }

    for keyobj in bindings.keys().iter() {
        let key = keyobj.downcast::<PyString>()?;

        let val = bindings.get_item(keyobj).unwrap().unwrap();

        if !val.is_callable() {
            exec_ctx.bind_param(key.to_str()?, val.extract::<PyCelValue>()?.into_inner())
        }
    }

    for callable in callables.iter() {
        exec_ctx.bind_func(callable.0, &callable.1);
    }

    let res = ctx.exec("entry", &exec_ctx);

    match res {
        Ok(res) => Ok(PyCelValue::new(res).to_object(py)),
        Err(err) => Err(PyRuntimeError::new_err(err.to_string())),
    }
}

#[pyclass(name = "CelContext")]
struct PyCelContext {
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

    pub fn exec(
        mut slf: PyRefMut<'_, Self>,
        name: &str,
        bindings: &PyBindContext,
    ) -> PyResult<PyObject> {
        let mut bindctx = BindContext::new();

        for (key, val) in bindings.bindings.iter() {
            bindctx.bind_param(&key, val.clone());
        }

        for (key, val) in bindings.funcs.iter() {
            bindctx.bind_func(&key, val);
        }

        match slf.ctx.exec(name, &bindctx) {
            Ok(val) => Ok(PyCelValue::new(val).to_object(slf.py())),
            Err(err) => Err(PyValueError::new_err(err.to_string())),
        }
    }
}

#[pyclass(name = "BindContext")]
struct PyBindContext {
    bindings: HashMap<String, CelValue>,
    funcs: HashMap<String, CelPyCallable>,
}

#[pymethods]
impl PyBindContext {
    #[new]
    pub fn new() -> PyBindContext {
        PyBindContext {
            bindings: HashMap::new(),
            funcs: HashMap::new(),
        }
    }

    pub fn bind_param(&mut self, name: &str, val: PyCelValue) {
        self.bindings.insert(name.to_owned(), val.into_inner());
    }

    pub fn bind_func(&mut self, name: &str, val: &PyAny) {
        self.funcs
            .insert(name.to_owned(), CelPyCallable::new(val.into()));
    }

    pub fn bind(&mut self, name: &str, val: &PyAny) -> PyResult<()> {
        if val.is_callable() {
            self.bind_func(name, val);
        } else {
            self.bind_param(name, val.extract()?);
        }

        Ok(())
    }
}

/* Module decl */
#[pymodule]
fn rscel_module(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(eval, m)?)?;
    m.add_class::<PyCelContext>()?;
    m.add_class::<PyBindContext>()?;
    Ok(())
}
