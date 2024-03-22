use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Duration, Utc};
use pyo3::{
    exceptions::PyValueError,
    types::{
        timezone_utc, PyBool, PyBytes, PyDateTime, PyDelta, PyDict, PyFloat, PyInt, PyList,
        PyString, PyTuple,
    },
    FromPyObject, PyAny, PyErr, PyObject, PyResult, PyTryFrom, Python,
};

use crate::CelValue;

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

trait WrappedExtract<'a> {
    fn wrapped_extract<D>(&'a self, path: &[&str]) -> Result<D, WrappedError>
    where
        D: FromPyObject<'a>;
}

macro_rules! wrapped_extract {
    ($type_name:ident) => {
        impl<'a> WrappedExtract<'a> for $type_name {
            fn wrapped_extract<D>(&'a self, path: &[&str]) -> Result<D, WrappedError>
            where
                D: FromPyObject<'a>,
            {
                match self.extract::<D>() {
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
impl<'a> WrappedExtract<'a> for &PyAny {
    fn wrapped_extract<D>(&'a self, path: &[&str]) -> Result<D, WrappedError>
    where
        D: FromPyObject<'a>,
    {
        match self.extract() {
            Ok(val) => Ok(val),
            Err(err) => Err(WrappedError::new(err, path)),
        }
    }
}

trait WrappedDowncast {
    fn wrapped_downcast<'a, D>(&'a self, path: &[&str]) -> Result<&'a D, WrappedError>
    where
        D: PyTryFrom<'a>;
}

impl WrappedDowncast for &PyAny {
    fn wrapped_downcast<'a, D>(&'a self, path: &[&str]) -> Result<&'a D, WrappedError>
    where
        D: PyTryFrom<'a>,
    {
        match self.downcast::<D>() {
            Ok(val) => Ok(val),
            Err(err) => Err(WrappedError {
                err: err.into(),
                path: path.iter().map(|s| (*s).to_owned()).collect(),
            }),
        }
    }
}

fn extract_celval_recurse<'source>(
    ob: &'source PyAny,
    current_path: &'source [&'source str],
) -> Result<CelValue, WrappedError> {
    match ob.get_type().name() {
        Ok(type_name) => match type_name {
            "int" => Ok(ob
                .wrapped_downcast::<PyInt>(current_path)?
                .wrapped_extract::<i64>(current_path)?
                .into()),
            "float" => Ok(ob
                .wrapped_downcast::<PyFloat>(current_path)?
                .wrapped_extract::<f64>(current_path)?
                .into()),
            "bool" => Ok(ob
                .wrapped_downcast::<PyBool>(current_path)?
                .wrapped_extract::<bool>(current_path)?
                .into()),
            "str" => Ok(ob
                .wrapped_downcast::<PyString>(current_path)?
                .wrapped_extract::<String>(current_path)?
                .into()),
            "bytes" => Ok(ob
                .wrapped_downcast::<PyBytes>(current_path)?
                .wrapped_extract::<Vec<u8>>(current_path)?
                .into()),
            "list" => {
                let mut next_path: Vec<String> =
                    current_path.iter().map(|s| (*s).to_owned()).collect();
                let mut vec: Vec<CelValue> = Vec::new();

                for (i, val) in ob
                    .wrapped_downcast::<PyList>(current_path)?
                    .iter()
                    .enumerate()
                {
                    next_path.push(format!("{}", i));
                    vec.push(val.wrapped_extract(
                        &next_path.iter().map(String::as_str).collect::<Vec<&str>>(),
                    )?);
                    next_path.pop();
                }

                Ok(vec.into())
            }
            "dict" => {
                let mut next_path: Vec<String> =
                    current_path.iter().map(|s| (*s).to_owned()).collect();
                let mut map: HashMap<String, CelValue> = HashMap::new();

                let mapobj = ob.wrapped_downcast::<PyDict>(current_path)?;
                for keyobj in mapobj.keys().iter() {
                    let key = match keyobj.downcast::<PyString>() {
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
                        mapobj.get_item(keyobj).unwrap().unwrap().wrapped_extract(
                            &next_path.iter().map(String::as_str).collect::<Vec<&str>>(),
                        )?,
                    );

                    next_path.pop();
                }

                Ok(map.into())
            }
            "datetime" => {
                let py_utc_dt = match Python::with_gil(|py| {
                    let utc = timezone_utc(py);
                    let py_astimezone = ob.getattr("astimezone")?;

                    let args = PyTuple::new(py, [utc]);

                    py_astimezone.call1(args)
                }) {
                    Ok(val) => val,
                    Err(err) => return Err(WrappedError::new(err, current_path)),
                };

                let dt = py_utc_dt
                    .wrapped_downcast::<PyDateTime>(current_path)?
                    .wrapped_extract::<DateTime<Utc>>(current_path)?;

                Ok(dt.with_timezone(&Utc).into())
            }
            "timedelta" => Ok(ob
                .wrapped_downcast::<PyDelta>(current_path)?
                .wrapped_extract::<Duration>(current_path)?
                .into()),
            "NoneType" => Ok(CelValue::from_null()),
            _ => Ok(CelValue::Dyn(Arc::<PyObject>::new(ob.into()))),
        },
        Err(_) => Err(WrappedError::new(
            PyValueError::new_err(format!("Failed to get type from {:?}", ob,)),
            current_path,
        )),
    }
}

impl<'source> FromPyObject<'source> for CelValue {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
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
