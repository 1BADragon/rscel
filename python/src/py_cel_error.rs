use pyo3::{exceptions::PyRuntimeError, PyErr};
use rscel::CelError;

pub struct PyCelError {
    inner: CelError,
}

impl PyCelError {
    pub fn new(inner: CelError) -> Self {
        Self { inner }
    }
}

impl From<PyCelError> for PyErr {
    fn from(err: PyCelError) -> PyErr {
        PyErr::new::<PyRuntimeError, _>(format!("{}", err.inner))
    }
}
