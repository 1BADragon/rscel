use pyo3::{types::PyAnyMethods, PyObject, Python};
use rscel::{CelError, CelValue, CelValueDyn};
use std::fmt;

use crate::py_cel_value::PyCelValue;

pub struct CelPyObject {
    inner: PyObject,
}

impl CelPyObject {
    pub fn new(inner: PyObject) -> Self {
        Self { inner }
    }
}

impl fmt::Display for CelPyObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CelPyObject {}", self.inner)
    }
}

impl fmt::Debug for CelPyObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CelPyObject {}", self.inner)
    }
}

impl CelValueDyn for CelPyObject {
    fn as_type(&self) -> CelValue {
        Python::with_gil(|py| {
            let inner = self.inner.bind(py);
            let name = inner.repr().unwrap();

            CelValue::Type(format!("pyobj-{}", name))
        })
    }

    fn access(&self, key: &str) -> CelValue {
        Python::with_gil(|py| {
            let obj = self.inner.bind(py);

            match obj.getattr(key) {
                Ok(res) => match res.extract::<PyCelValue>() {
                    Ok(val) => val.into_inner(),
                    Err(err) => CelValue::from_err(CelError::Misc(err.to_string())),
                },
                Err(err) => CelValue::from_err(CelError::Misc(err.to_string())),
            }
        })
    }

    fn eq(&self, rhs: &CelValue) -> CelValue {
        let lhs_type = self.as_type();
        let rhs_type = self.as_type();

        if let CelValue::Dyn(rhs) = rhs {
            if let Some(rhs_obj) = rhs.any_ref().downcast_ref::<CelPyObject>() {
                return Python::with_gil(|py| {
                    let lhs_obj = self.inner.bind(py);
                    let rhs_obj = rhs_obj.inner.bind(py);

                    match lhs_obj.eq(rhs_obj) {
                        Ok(res) => CelValue::from_bool(res),
                        Err(err) => CelValue::from_err(CelError::Misc(err.to_string())),
                    }
                });
            }
        }

        CelValue::from_err(CelError::invalid_op(&format!(
            "Invalid op == between {} and {}",
            lhs_type, rhs_type
        )))
    }

    fn is_truthy(&self) -> bool {
        Python::with_gil(|py| {
            let inner = self.inner.bind(py);

            match inner.is_truthy() {
                Ok(res) => res,
                Err(_) => false, // this is just going to have to work. Basically is the equiv of calling bool(obj) and it throwing
            }
        })
    }

    fn any_ref<'a>(&'a self) -> &'a dyn std::any::Any {
        self
    }
}
