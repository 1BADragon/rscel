use super::py_cel_value::PyCelValue;
use pyo3::{pyclass, pymethods, types::PyAnyMethods, Bound, PyAny, PyResult};
use rscel::CelValue;
use std::collections::HashMap;

use crate::celpycallable::CelPyCallable;

#[pyclass(name = "BindContext")]
pub struct PyBindContext {
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

    pub fn bind_func(&mut self, name: &str, val: &Bound<PyAny>) {
        self.funcs
            .insert(name.to_owned(), CelPyCallable::new(val.clone().unbind()));
    }

    pub fn bind(&mut self, name: &str, val: &Bound<PyAny>) -> PyResult<()> {
        if val.is_callable() {
            self.bind_func(name, val);
        } else {
            self.bind_param(name, val.extract()?);
        }

        Ok(())
    }
}

impl PyBindContext {
    pub fn bindings(&self) -> &HashMap<String, CelValue> {
        &self.bindings
    }

    pub fn funcs(&self) -> &HashMap<String, CelPyCallable> {
        &self.funcs
    }
}
