use pyo3::{exceptions::PyException, PyErr};
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
        PyErr::new::<PyException, _>(format!("{}", err.inner))
    }
}
