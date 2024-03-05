use crate::{BindContext, CelContext, CelError, CelValue, CelValueDyn};

use chrono::{DateTime, Duration, Utc};
use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
    types::{PyBool, PyBytes, PyDateTime, PyDelta, PyDict, PyFloat, PyInt, PyList, PyString},
};
use std::{collections::HashMap, sync::Arc};

mod celpycallable;

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

    ctx.add_program_str("entry", &prog_str).unwrap();

    for keyobj in bindings.keys().iter() {
        let key = keyobj.downcast::<PyString>()?;

        let val = bindings.get_item(keyobj).unwrap().unwrap();

        if !val.is_callable() {
            exec_ctx.bind_param(key.to_str()?, val.extract()?)
        }
    }

    for callable in callables.iter() {
        exec_ctx.bind_func(callable.0, &callable.1);
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
        let mut bindctx = BindContext::new();

        for (key, val) in bindings.bindings.iter() {
            bindctx.bind_param(&key, val.clone());
        }

        for (key, val) in bindings.funcs.iter() {
            bindctx.bind_func(&key, val);
        }

        match slf.ctx.exec(name, &bindctx) {
            Ok(val) => Ok(val.to_object(slf.py())),
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

    pub fn bind_param(&mut self, name: &str, val: CelValue) {
        self.bindings.insert(name.to_owned(), val);
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
fn rscel(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(eval, m)?)?;
    m.add_class::<PyCelContext>()?;
    m.add_class::<PyBindContext>()?;
    Ok(())
}

impl CelValueDyn for PyObject {
    fn as_type(&self) -> CelValue {
        Python::with_gil(|py| {
            let inner = self.as_ref(py);
            let name = inner.get_type().name().unwrap();

            CelValue::Type(format!("pyobj-{}", name))
        })
    }

    fn access(&self, key: &str) -> crate::CelResult<CelValue> {
        Python::with_gil(|py| {
            let obj = self.as_ref(py);

            match obj.getattr(key) {
                Ok(res) => match res.extract() {
                    Ok(val) => Ok(val),
                    Err(err) => Err(CelError::Misc(err.to_string())),
                },
                Err(err) => Err(CelError::Misc(err.to_string())),
            }
        })
    }

    fn eq(&self, rhs: &CelValue) -> crate::CelResult<CelValue> {
        let lhs_type = self.as_type();
        let rhs_type = self.as_type();

        if let CelValue::Dyn(rhs) = rhs {
            if let Some(rhs_obj) = rhs.any_ref().downcast_ref::<PyObject>() {
                return Python::with_gil(|py| {
                    let lhs_obj = self.as_ref(py);
                    let rhs_obj = rhs_obj.as_ref(py);

                    match lhs_obj.eq(rhs_obj) {
                        Ok(res) => Ok(CelValue::from_bool(res)),
                        Err(err) => Err(CelError::Misc(err.to_string())),
                    }
                });
            }
        }

        Err(CelError::invalid_op(&format!(
            "Invalid op == between {} and {}",
            lhs_type, rhs_type
        )))
    }

    fn is_truthy(&self) -> bool {
        Python::with_gil(|py| {
            let inner = self.as_ref(py);

            match inner.is_true() {
                Ok(res) => res,
                Err(_) => false, // this is just going to have to work. Basically is the equiv of calling bool(obj) and it throwing
            }
        })
    }

    fn any_ref<'a>(&'a self) -> &'a dyn std::any::Any {
        self
    }
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

                        map.insert(key, mapobj.get_item(keyobj).unwrap().unwrap().extract()?);
                    }

                    Ok(map.into())
                }
                "datetime" => Ok(ob
                    .downcast::<PyDateTime>()?
                    .extract::<DateTime<Utc>>()?
                    .into()),
                "timedelta" => Ok(ob.downcast::<PyDelta>()?.extract::<Duration>()?.into()),
                "NoneType" => Ok(CelValue::from_null()),
                _ => Ok(CelValue::Dyn(Arc::<PyObject>::new(ob.into()))),
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
        use crate::CelValue::*;

        match self {
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
