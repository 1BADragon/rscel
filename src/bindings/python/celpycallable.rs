use pyo3::{types::PyTuple, Py, PyAny, PyObject, Python, ToPyObject};

use crate::{CelError, CelValue};

pub struct CelPyCallable {
    func: Py<PyAny>,
}

impl CelPyCallable {
    pub fn new(func: Py<PyAny>) -> CelPyCallable {
        CelPyCallable { func }
    }
}

impl FnOnce<(CelValue, &[CelValue])> for CelPyCallable {
    type Output = CelValue;

    extern "rust-call" fn call_once(self, args: (CelValue, &[CelValue])) -> Self::Output {
        Python::with_gil(|py| {
            match self.func.call(
                py,
                PyTuple::new(
                    py,
                    &[args.0]
                        .iter()
                        .filter(|x| !x.is_null())
                        .map(|x| x.to_object(py))
                        .chain(args.1.into_iter().map(|x| x.to_object(py)))
                        .collect::<Vec<PyObject>>(),
                ),
                None,
            ) {
                Ok(val) => val.extract(py).unwrap(),
                Err(val) => CelValue::from_err(CelError::runtime(&val.to_string())),
            }
        })
    }
}

impl FnMut<(CelValue, &[CelValue])> for CelPyCallable {
    extern "rust-call" fn call_mut(&mut self, args: (CelValue, &[CelValue])) -> Self::Output {
        Python::with_gil(|py| {
            match self.func.call(
                py,
                PyTuple::new(
                    py,
                    &[args.0]
                        .iter()
                        .filter(|x| !x.is_null())
                        .map(|x| x.to_object(py))
                        .chain(args.1.into_iter().map(|x| x.to_object(py)))
                        .collect::<Vec<PyObject>>(),
                ),
                None,
            ) {
                Ok(val) => val.extract(py).unwrap(),
                Err(val) => CelValue::from_err(CelError::runtime(&val.to_string())),
            }
        })
    }
}

impl Fn<(CelValue, &[CelValue])> for CelPyCallable {
    extern "rust-call" fn call(&self, args: (CelValue, &[CelValue])) -> Self::Output {
        Python::with_gil(|py| {
            match self.func.call(
                py,
                PyTuple::new(
                    py,
                    &[args.0]
                        .iter()
                        .filter(|x| !x.is_null())
                        .map(|x| x.to_object(py))
                        .chain(args.1.into_iter().map(|x| x.to_object(py)))
                        .collect::<Vec<PyObject>>(),
                ),
                None,
            ) {
                Ok(val) => val.extract(py).unwrap(),
                Err(val) => CelValue::from_err(CelError::runtime(&val.to_string())),
            }
        })
    }
}
