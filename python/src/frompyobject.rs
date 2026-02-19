use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Duration, Utc};
use pyo3::{
    conversion::FromPyObject,
    exceptions::PyValueError,
    types::{
        PyAnyMethods, PyBool, PyBytes, PyDateTime, PyDelta, PyDict, PyDictMethods, PyFloat, PyInt,
        PyList, PyString, PyStringMethods, PyTuple, PyTypeMethods, PyTzInfo,
    },
    Borrowed, PyAny, PyErr, PyResult, PyTypeCheck,
};

use rscel::CelValue;

use crate::{cel_py_object::CelPyObject, py_cel_value::PyCelValue};

struct WrappedError {
    err: PyErr,
    path: Vec<String>,
}

impl WrappedError {
    fn new(err: PyErr, path: &[&str]) -> WrappedError {
        WrappedError {
            err,
            path: path.iter().map(|s| (*s).to_owned()).collect(),
        }
    }
}

trait WrappedExtract<'a, 'py> {
    fn wrapped_extract<D>(self, path: &[&str]) -> Result<D, WrappedError>
    where
        D: FromPyObject<'a, 'py, Error = PyErr>;
}

macro_rules! wrapped_extract {
    ($type_name:ident) => {
        impl<'a, 'py: 'a> WrappedExtract<'a, 'py> for Borrowed<'a, 'py, $type_name> {
            fn wrapped_extract<D>(self, path: &[&str]) -> Result<D, WrappedError>
            where
                D: FromPyObject<'a, 'py, Error = PyErr>,
            {
                match D::extract(self.cast().map_err(|e| WrappedError::new(e.into(), path))?) {
                    Ok(val) => Ok(val),
                    Err(err) => Err(WrappedError::new(err, path)),
                }
            }
        }
    };
}

wrapped_extract!(PyInt);
wrapped_extract!(PyFloat);
wrapped_extract!(PyBool);
wrapped_extract!(PyString);
wrapped_extract!(PyBytes);
wrapped_extract!(PyDateTime);
wrapped_extract!(PyDelta);

impl<'a, 'py> WrappedExtract<'a, 'py> for Borrowed<'a, 'py, PyAny> {
    fn wrapped_extract<D>(self, path: &[&str]) -> Result<D, WrappedError>
    where
        D: FromPyObject<'a, 'py, Error = PyErr>,
    {
        match D::extract(self) {
            Ok(val) => Ok(val),
            Err(err) => Err(WrappedError::new(err, path)),
        }
    }
}

trait WrappedDowncast {
    fn wrapped_downcast<D>(&self, path: &[&str]) -> Result<Borrowed<'_, '_, D>, WrappedError>
    where
        D: PyTypeCheck;
}

impl WrappedDowncast for Borrowed<'_, '_, PyAny> {
    fn wrapped_downcast<D>(&self, path: &[&str]) -> Result<Borrowed<'_, '_, D>, WrappedError>
    where
        D: PyTypeCheck,
    {
        match self.cast::<D>() {
            Ok(val) => Ok(val),
            Err(err) => Err(WrappedError {
                err: err.into(),
                path: path.iter().map(|s| (*s).to_owned()).collect(),
            }),
        }
    }
}

fn extract_celval_recurse<'a, 'source>(
    ob: Borrowed<'a, 'source, PyAny>,
    current_path: &'source [&'source str],
) -> Result<PyCelValue, WrappedError> {
    match ob.get_type().name() {
        Ok(type_name) => match type_name.to_str().expect("to get python type as str") {
            "int" => Ok(PyCelValue::new(
                ob.wrapped_downcast::<PyInt>(current_path)?
                    .wrapped_extract::<i64>(current_path)?
                    .into(),
            )),
            "float" => Ok(PyCelValue::new(
                ob.wrapped_downcast::<PyFloat>(current_path)?
                    .wrapped_extract::<f64>(current_path)?
                    .into(),
            )),
            "bool" => Ok(PyCelValue::new(
                ob.wrapped_downcast::<PyBool>(current_path)?
                    .wrapped_extract::<bool>(current_path)?
                    .into(),
            )),
            "str" => Ok(PyCelValue::new(
                ob.wrapped_downcast::<PyString>(current_path)?
                    .wrapped_extract::<String>(current_path)?
                    .into(),
            )),
            "bytes" => Ok(PyCelValue::new(
                ob.wrapped_downcast::<PyBytes>(current_path)?
                    .wrapped_extract::<Vec<u8>>(current_path)?
                    .into(),
            )),
            "list" => {
                let mut next_path: Vec<String> =
                    current_path.iter().map(|s| (*s).to_owned()).collect();
                let mut vec: Vec<CelValue> = Vec::new();

                for (i, val) in ob
                    .wrapped_downcast::<PyList>(current_path)?
                    .try_iter()
                    .expect("list to iterate")
                    .enumerate()
                {
                    let val = val.expect("val to exist");
                    next_path.push(format!("{}", i));
                    vec.push(
                        val.as_borrowed()
                            .wrapped_extract::<PyCelValue>(
                                &next_path.iter().map(String::as_str).collect::<Vec<&str>>(),
                            )?
                            .into_inner(),
                    );
                    next_path.pop();
                }

                Ok(PyCelValue::new(vec.into()))
            }
            "dict" => {
                let mut next_path: Vec<String> =
                    current_path.iter().map(|s| (*s).to_owned()).collect();
                let mut map: HashMap<String, CelValue> = HashMap::new();

                let mapobj = ob.wrapped_downcast::<PyDict>(current_path)?;
                for keyobj in mapobj.keys().try_iter().expect("keys to iterate") {
                    let keyobj = keyobj.expect("keyobj to exist");
                    let key = match keyobj.cast::<PyString>() {
                        Ok(val) => val.to_string(),
                        Err(_) => {
                            return Err(WrappedError {
                                err: PyValueError::new_err(format!(
                                    "Bad key type {}",
                                    keyobj.get_type().name().unwrap()
                                )),
                                path: next_path,
                            })
                        }
                    };

                    next_path.push(key.clone());

                    map.insert(
                        key,
                        mapobj
                            .get_item(keyobj)
                            .unwrap()
                            .unwrap()
                            .as_borrowed()
                            .wrapped_extract::<PyCelValue>(
                                &next_path.iter().map(String::as_str).collect::<Vec<&str>>(),
                            )?
                            .into_inner(),
                    );

                    next_path.pop();
                }

                Ok(PyCelValue::new(map.into()))
            }
            "datetime.datetime" | "datetime" => {
                let py = ob.py();

                let utc =
                    PyTzInfo::utc(py).map_err(|e| WrappedError::new(e.into(), current_path))?;
                let py_astimezone = match ob.getattr("astimezone") {
                    Ok(v) => v,
                    Err(e) => return Err(WrappedError::new(e, current_path)),
                };

                let args = match PyTuple::new(py, [utc]) {
                    Ok(v) => v,
                    Err(e) => return Err(WrappedError::new(e, current_path)),
                };

                let py_utc_dt = match py_astimezone.call1(args) {
                    Ok(val) => val,
                    Err(err) => return Err(WrappedError::new(err, current_path)),
                };

                let dt = (&py_utc_dt)
                    .as_borrowed()
                    .wrapped_downcast::<PyDateTime>(current_path)?
                    .wrapped_extract::<DateTime<Utc>>(current_path)?;

                Ok(PyCelValue::new(dt.into()))
            }
            "datetime.timedelta" | "timedelta" => Ok(PyCelValue::new(
                ob.wrapped_downcast::<PyDelta>(current_path)?
                    .wrapped_extract::<Duration>(current_path)?
                    .into(),
            )),
            "NoneType" => Ok(PyCelValue::new(CelValue::from_null())),
            _ => Ok(PyCelValue::new(CelValue::Dyn(Arc::<CelPyObject>::new(
                CelPyObject::new(ob.to_owned().unbind()),
            )))),
        },
        Err(_) => Err(WrappedError::new(
            PyValueError::new_err(format!("Failed to get type from {:?}", ob,)),
            current_path,
        )),
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for PyCelValue {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        match extract_celval_recurse(ob, &[]) {
            Ok(val) => Ok(val),
            Err(WrappedError { err, path }) => Err(PyValueError::new_err(format!(
                "Failed to convert '{}': {}",
                path.join("."),
                err
            ))),
        }
    }
}
