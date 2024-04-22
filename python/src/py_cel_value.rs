use std::collections::HashMap;
use std::fmt;

use pyo3::{types::PyBytes, PyObject, Python, ToPyObject};
use rscel::CelValue;

use crate::cel_py_object::CelPyObject;

pub struct PyCelValue(CelValue);

pub struct PyCelValueRef<'a>(&'a CelValue);

impl<'a> PyCelValueRef<'a> {
    pub fn new(inner: &'a CelValue) -> Self {
        Self(inner)
    }
}

impl rscel::CelValueDyn for PyCelValue {
    fn as_type(&self) -> CelValue {
        <CelValue as rscel::CelValueDyn>::as_type(&self.0)
    }

    fn access(&self, key: &str) -> CelValue {
        <CelValue as rscel::CelValueDyn>::access(&self.0, key)
    }

    fn eq(&self, rhs_val: &CelValue) -> CelValue {
        <CelValue as rscel::CelValueDyn>::eq(&self.0, rhs_val)
    }

    fn is_truthy(&self) -> bool {
        <CelValue as rscel::CelValueDyn>::is_truthy(&self.0)
    }

    fn any_ref<'a>(&'a self) -> &'a dyn std::any::Any {
        <CelValue as rscel::CelValueDyn>::any_ref(&self.0)
    }
}

impl PyCelValue {
    pub fn new(inner: CelValue) -> Self {
        Self(inner)
    }

    pub fn into_inner(self) -> CelValue {
        self.0
    }
}

impl ToPyObject for PyCelValue {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        PyCelValueRef(&self.0).to_object(py)
    }
}

impl fmt::Display for PyCelValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for PyCelValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> ToPyObject for PyCelValueRef<'a> {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        use crate::CelValue::*;

        match self.0 {
            Int(i) => i.to_object(py),
            UInt(i) => i.to_object(py),
            Float(f) => f.to_object(py),
            Bool(b) => b.to_object(py),
            String(s) => s.to_object(py),
            Bytes(s) => PyBytes::new_bound(py, &s).into(),
            List(l) => l
                .into_iter()
                .map(|x| PyCelValueRef(x).to_object(py))
                .collect::<Vec<_>>()
                .to_object(py),
            Map(m) => m
                .into_iter()
                .map(|(k, v)| (k, PyCelValueRef(v).to_object(py)))
                .collect::<HashMap<_, _>>()
                .to_object(py),
            TimeStamp(ts) => ts.to_object(py),
            Duration(d) => d.to_object(py),
            Null => py.None(),
            Dyn(d) => {
                match d.any_ref().downcast_ref::<CelPyObject>() {
                    Some(obj) => obj.as_inner().clone(),
                    // This *should* never happen. If this downcase were to fail that would
                    // mean that the data in this dyn isn't a CelPyObject which should be impossible
                    // for these bidnings
                    None => py.None(),
                }
            }
            _ => py.None(),
        }
    }
}
