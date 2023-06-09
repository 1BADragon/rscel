use crate::{BindContext, CelContext, CelValue};

use chrono::{DateTime, Duration, Utc};
use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
    types::{PyBool, PyBytes, PyDateTime, PyDelta, PyDict, PyFloat, PyInt, PyList, PyString},
};
use std::collections::HashMap;

/* Eval entry point */
#[pyfunction]
fn eval(py: Python<'_>, prog_str: String, bindings: &PyDict) -> PyResult<PyObject> {
    let mut ctx = CelContext::new();
    let mut exec_ctx = BindContext::new();

    ctx.add_program_str("entry", &prog_str).unwrap();

    for keyobj in bindings.keys().iter() {
        let key = keyobj.downcast::<PyString>()?;
        exec_ctx.bind_param(key.to_str()?, bindings.get_item(keyobj).unwrap().extract()?)
    }

    let res = ctx.exec("entry", &exec_ctx);

    match res {
        Ok(res) => Ok(res.to_object(py)),
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
        match slf.ctx.exec(name, &bindings.ctx) {
            Ok(val) => Ok(val.to_object(slf.py())),
            Err(err) => Err(PyValueError::new_err(err.to_string())),
        }
    }
}

#[pyclass(name = "BindContext")]
struct PyBindContext {
    ctx: BindContext,
}

#[pymethods]
impl PyBindContext {
    #[new]
    pub fn new() -> PyBindContext {
        PyBindContext {
            ctx: BindContext::new(),
        }
    }

    pub fn bind(mut slf: PyRefMut<'_, Self>, name: &str, val: CelValue) {
        slf.ctx.bind_param(name, val);
    }
}

/* Module decl */
#[pymodule]
fn rscel(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(eval, m)?)?;
    m.add_class::<PyCelContext>()?;
    m.add_class::<PyBindContext>()?;
    Ok(())
}

impl<'source> FromPyObject<'source> for CelValue {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        match ob.get_type().name() {
            Ok(type_name) => match type_name {
                "int" => Ok(ob.downcast::<PyInt>()?.extract::<i64>()?.into()),
                "float" => Ok(ob.downcast::<PyFloat>()?.extract::<f64>()?.into()),
                "bool" => Ok(ob.downcast::<PyBool>()?.extract::<bool>()?.into()),
                "str" => Ok(ob.downcast::<PyString>()?.extract::<String>()?.into()),
                "bytes" => Ok(ob.downcast::<PyBytes>()?.extract::<Vec<u8>>()?.into()),
                "list" => {
                    let mut vec: Vec<CelValue> = Vec::new();

                    for val in ob.downcast::<PyList>()?.iter() {
                        vec.push(val.extract()?)
                    }

                    Ok(vec.into())
                }
                "dict" => {
                    let mut map: HashMap<String, CelValue> = HashMap::new();

                    let mapobj = ob.downcast::<PyDict>()?;
                    for keyobj in mapobj.keys().iter() {
                        let key = keyobj.downcast::<PyString>()?.to_string();

                        map.insert(key, mapobj.get_item(keyobj).unwrap().extract()?);
                    }

                    Ok(map.into())
                }
                "datetime.datetime" => Ok(ob
                    .downcast::<PyDateTime>()?
                    .extract::<DateTime<Utc>>()?
                    .into()),
                "datetime.timedelta" => Ok(ob.downcast::<PyDelta>()?.extract::<Duration>()?.into()),
                "NoneType" => Ok(CelValue::from_null()),
                other => Err(PyValueError::new_err(format!(
                    "{} is not a compatable rscel type",
                    other
                ))),
            },
            Err(_) => PyResult::Err(PyValueError::new_err(format!(
                "Failed to get type from {:?}",
                ob,
            ))),
        }
    }
}

/* private functions */
impl ToPyObject for CelValue {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        use crate::CelValueInner::*;

        match self.inner() {
            Int(i) => i.to_object(py),
            UInt(i) => i.to_object(py),
            Float(f) => f.to_object(py),
            Bool(b) => b.to_object(py),
            String(s) => s.to_object(py),
            Bytes(s) => s.to_object(py),
            List(l) => l
                .into_iter()
                .map(|x| x.to_object(py))
                .collect::<Vec<_>>()
                .to_object(py),
            Map(m) => m
                .into_iter()
                .map(|(k, v)| (k, v.to_object(py)))
                .collect::<HashMap<_, _>>()
                .to_object(py),
            TimeStamp(ts) => ts.to_object(py),
            Duration(d) => d.to_object(py),
            Null => py.None(),
            _ => py.None(),
        }
    }
}
