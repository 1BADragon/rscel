use crate::{CelContext, ExecContext, ValueCell};
use chrono::{Datelike, Timelike};
use pyo3::{
    exceptions::PyRuntimeError,
    prelude::*,
    types::{timezone_utc, PyBytes, PyDateTime, PyDelta},
};
use serde_json;
use std::collections::HashMap;

/* Eval entry point */
#[pyfunction]
fn eval(py: Python<'_>, prog_str: String, bindings: String) -> PyResult<PyObject> {
    let mut ctx = CelContext::new();
    let mut exec_ctx = ExecContext::new();

    ctx.add_program_str("entry", &prog_str).unwrap();
    exec_ctx
        .bind_params_from_json_obj(serde_json::from_str(&bindings).unwrap())
        .unwrap();

    let res = ctx.exec("entry", &exec_ctx);

    match res {
        Ok(res) => Ok(to_pyobject(py, &res)),
        Err(err) => Err(PyRuntimeError::new_err(err.str().to_owned())),
    }
}

/* Module decl */
#[pymodule]
fn rscel(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(eval, m)?)?;
    Ok(())
}

/* private functions */

fn to_pyobject(py: Python<'_>, valcel: &ValueCell) -> PyObject {
    use crate::ValueCellInner::*;

    match valcel.inner() {
        Int(i) => i.to_object(py),
        UInt(i) => i.to_object(py),
        Float(f) => f.to_object(py),
        Bool(b) => b.to_object(py),
        String(s) => s.to_object(py),
        Bytes(s) => PyBytes::new(py, &s).into(),
        List(l) => l
            .into_iter()
            .map(|x| to_pyobject(py, x))
            .collect::<Vec<_>>()
            .to_object(py),
        Map(m) => m
            .into_iter()
            .map(|(k, v)| (k, to_pyobject(py, v)))
            .collect::<HashMap<_, _>>()
            .to_object(py),
        TimeStamp(ts) => PyDateTime::new(
            py,
            ts.year().try_into().unwrap(),
            ts.month().try_into().unwrap(),
            ts.day().try_into().unwrap(),
            ts.hour().try_into().unwrap(),
            ts.minute().try_into().unwrap(),
            ts.second().try_into().unwrap(),
            0,
            Some(timezone_utc(py)),
        )
        .unwrap()
        .into(),
        Duration(d) => match d.num_microseconds() {
            Some(usec) => {
                let n_days = usec / 86_400_000_000i64;
                let n_secs = (usec % 86_400_000_000i64) / 1_000_000i64;
                let n_usec = usec % 1_000_000i64;
                PyDelta::new(
                    py,
                    n_days.try_into().unwrap(),
                    n_secs.try_into().unwrap(),
                    n_usec.try_into().unwrap(),
                    false,
                )
                .unwrap()
                .into()
            }
            None => {
                let total_millis = d.num_milliseconds();
                let n_days = total_millis / 86_400_000i64;
                let n_sec = (total_millis % 86_400_000i64) / 1_000i64;
                let n_usec = (total_millis % 1_000i64) * 1_000i64;
                PyDelta::new(
                    py,
                    n_days.try_into().unwrap(),
                    n_sec.try_into().unwrap(),
                    n_usec.try_into().unwrap(),
                    false,
                )
                .unwrap()
                .into()
            }
        },
        Null => py.None(),
        _ => py.None(),
    }
}
