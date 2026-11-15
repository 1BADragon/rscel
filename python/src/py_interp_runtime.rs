use chrono::{DateTime, Utc};
use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyDateTime, PyTuple, PyTzInfo},
};
use rscel::InterpRuntime;

/// Per-execution runtime state. Its start time is what `now()` returns during a run.
///
/// Pass the same runtime to several executions to replay them with an identical time.
#[pyclass(name = "InterpRuntime", eq, frozen, skip_from_py_object)]
#[derive(Clone, PartialEq)]
pub struct PyInterpRuntime {
    inner: InterpRuntime,
}

impl PyInterpRuntime {
    pub fn inner(&self) -> &InterpRuntime {
        &self.inner
    }
}

#[pymethods]
impl PyInterpRuntime {
    /// Create a runtime starting at `start_time`, or at the current time if omitted.
    /// Naive datetimes are interpreted as local time, the same as datetime bindings.
    #[new]
    #[pyo3(signature = (start_time=None))]
    fn new(start_time: Option<&Bound<'_, PyAny>>) -> PyResult<Self> {
        let inner = match start_time {
            Some(dt) => InterpRuntime::new(datetime_to_utc(dt)?),
            None => InterpRuntime::default(),
        };

        Ok(PyInterpRuntime { inner })
    }

    /// Create a runtime from nanoseconds since the Unix epoch, for exact replay.
    #[staticmethod]
    fn from_ns(ns: i64) -> Self {
        PyInterpRuntime {
            inner: InterpRuntime::new(DateTime::from_timestamp_nanos(ns)),
        }
    }

    /// The start time as a timezone-aware UTC datetime (microsecond precision).
    #[getter]
    fn start_time(&self) -> DateTime<Utc> {
        self.inner.start_time()
    }

    /// The start time as nanoseconds since the Unix epoch (full precision).
    #[getter]
    fn start_time_ns(&self) -> PyResult<i64> {
        self.inner
            .start_time()
            .timestamp_nanos_opt()
            .ok_or_else(|| PyValueError::new_err("start time is out of range for nanoseconds"))
    }

    fn __repr__(&self) -> String {
        format!(
            "InterpRuntime(start_time={})",
            self.inner.start_time().to_rfc3339()
        )
    }
}

fn datetime_to_utc(dt: &Bound<'_, PyAny>) -> PyResult<DateTime<Utc>> {
    let py = dt.py();

    if !dt.is_instance_of::<PyDateTime>() {
        return Err(PyValueError::new_err(
            "InterpRuntime start_time must be a datetime.datetime",
        ));
    }

    let utc = PyTzInfo::utc(py)?;
    let as_utc = dt.call_method1("astimezone", PyTuple::new(py, [utc])?)?;

    as_utc.extract::<DateTime<Utc>>()
}
