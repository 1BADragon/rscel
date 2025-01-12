use std::collections::HashMap;
use std::fmt;

use pyo3::{types::PyBytes, IntoPyObject, Python};
use pyo3::{Bound, IntoPyObjectExt, PyAny, PyErr};
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

impl<'py> IntoPyObject<'py> for PyCelValue {
    type Error = PyErr;
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyCelValueRef::new(&self.0).into_pyobject(py)
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

impl<'py, 'a> IntoPyObject<'py> for PyCelValueRef<'a> {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        use crate::CelValue::*;

        match self.0 {
            Int(i) => i.into_pyobject_or_pyerr(py).map(|o| o.into_any()),
            UInt(i) => i.into_pyobject_or_pyerr(py).map(|o| o.into_any()),
            Float(f) => f.into_pyobject_or_pyerr(py).map(|o| o.into_any()),
            Bool(b) => b
                .into_pyobject_or_pyerr(py)
                .map(|o| o.to_owned().into_any()),
            String(s) => s.into_pyobject_or_pyerr(py).map(|o| o.into_any()),
            Bytes(s) => Ok(PyBytes::new(py, s.as_slice()).into_any()),
            List(l) => l
                .into_iter()
                .map(|x| {
                    PyCelValueRef(x)
                        .into_pyobject_or_pyerr(py)
                        .map(|o| o.into_any())
                })
                .collect::<Result<Vec<_>, PyErr>>()?
                .into_pyobject_or_pyerr(py)
                .map(|o| o.into_any()),
            Map(m) => m
                .into_iter()
                .map(|(k, v)| PyCelValueRef(v).into_pyobject_or_pyerr(py).map(|o| (k, o)))
                .collect::<Result<HashMap<_, _>, PyErr>>()?
                .into_pyobject_or_pyerr(py)
                .map(|o| o.into_any()),
            TimeStamp(ts) => ts.into_pyobject_or_pyerr(py).map(|o| o.into_any()),
            Duration(d) => d.into_pyobject_or_pyerr(py).map(|o| o.into_any()),
            Null => Ok(py.None().bind(py).to_owned()),
            Dyn(d) => {
                match d.any_ref().downcast_ref::<CelPyObject>() {
                    Some(obj) => Ok(obj.as_inner().clone().bind(py).to_owned()),
                    // This *should* never happen. If this downcase were to fail that would
                    // mean that the data in this dyn isn't a CelPyObject which should be impossible
                    // for these bidnings
                    None => Ok(py.None().bind(py).to_owned()),
                }
            }
            _ => Ok(py.None().bind(py).to_owned()),
        }
    }
}
