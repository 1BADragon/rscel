use crate::py_cel_value::{PyCelValue, PyCelValueRef};
use pyo3::{types::PyTuple, Bound, IntoPyObject, Py, PyAny, PyErr, Python};
use rscel::{CelError, CelValue};

pub struct CelPyCallable {
    func: Py<PyAny>,
}

impl CelPyCallable {
    pub fn new(func: Py<PyAny>) -> CelPyCallable {
        CelPyCallable { func }
    }
}

impl FnOnce<(CelValue, Vec<CelValue>)> for CelPyCallable {
    type Output = CelValue;

    extern "rust-call" fn call_once(self, args: (CelValue, Vec<CelValue>)) -> Self::Output {
        Python::with_gil(|py| {
            match self.func.call(
                py,
                PyTuple::new(
                    py,
                    &[args.0]
                        .iter()
                        .filter(|x| !x.is_null())
                        .map(|x| PyCelValueRef::new(x).into_pyobject(py))
                        .chain(
                            args.1
                                .into_iter()
                                .map(|x| PyCelValueRef::new(&x).into_pyobject(py)),
                        )
                        .collect::<Result<Vec<Bound<'_, PyAny>>, PyErr>>()
                        .expect("argument collection failed"),
                )
                .expect("pytuple construction failed"),
                None,
            ) {
                Ok(val) => val.extract::<PyCelValue>(py).unwrap().into_inner(),
                Err(val) => CelValue::from_err(CelError::runtime(&val.to_string())),
            }
        })
    }
}

impl FnMut<(CelValue, Vec<CelValue>)> for CelPyCallable {
    extern "rust-call" fn call_mut(&mut self, args: (CelValue, Vec<CelValue>)) -> Self::Output {
        Python::with_gil(|py| {
            match self.func.call(
                py,
                PyTuple::new(
                    py,
                    &[args.0]
                        .iter()
                        .filter(|x| !x.is_null())
                        .map(|x| {
                            PyCelValueRef::new(x)
                                .into_pyobject(py)
                                .map(|o| o.into_any())
                        })
                        .chain(
                            args.1
                                .into_iter()
                                .map(|x| PyCelValueRef::new(&x).into_pyobject(py)),
                        )
                        .collect::<Result<Vec<Bound<'_, PyAny>>, PyErr>>()
                        .expect("argument collection failed"),
                )
                .expect("pytuple construction failed"),
                None,
            ) {
                Ok(val) => val.extract::<PyCelValue>(py).unwrap().into_inner(),
                Err(val) => CelValue::from_err(CelError::runtime(&val.to_string())),
            }
        })
    }
}

impl Fn<(CelValue, Vec<CelValue>)> for CelPyCallable {
    extern "rust-call" fn call(&self, args: (CelValue, Vec<CelValue>)) -> Self::Output {
        Python::with_gil(|py| {
            match self.func.call(
                py,
                PyTuple::new(
                    py,
                    &[args.0]
                        .iter()
                        .filter(|x| !x.is_null())
                        .map(|x| {
                            PyCelValueRef::new(x)
                                .into_pyobject(py)
                                .map(|o| o.into_any())
                        })
                        .chain(
                            args.1
                                .into_iter()
                                .map(|x| PyCelValueRef::new(&x).into_pyobject(py)),
                        )
                        .collect::<Result<Vec<Bound<'_, PyAny>>, PyErr>>()
                        .expect("argument collection failed"),
                )
                .expect("pytuple construction failed"),
                None,
            ) {
                Ok(val) => val.extract::<PyCelValue>(py).unwrap().into_inner(),
                Err(val) => CelValue::from_err(CelError::runtime(&val.to_string())),
            }
        })
    }
}
