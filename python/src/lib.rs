use pyo3::{exceptions::PyRuntimeError, prelude::*, types::PyBytes};
use rscel::serde_json;
use std::collections::HashMap;

/* Eval entry point */
#[pyfunction]
fn eval(py: Python<'_>, prog_str: String, bindings: String) -> PyResult<PyObject> {
    let res = py.allow_threads(move || {
        let mut ctx = rscel::CelContext::new();
        let mut exec_ctx = rscel::ExecContext::new();

        ctx.add_program_str("entry", &prog_str).unwrap();
        exec_ctx
            .bind_params_from_json_obj(serde_json::from_str(&bindings).unwrap())
            .unwrap();

        ctx.exec("entry", &exec_ctx)
    });

    match res {
        Ok(res) => Ok(to_pyobject(py, res)),
        Err(err) => Err(PyRuntimeError::new_err(err.str().to_owned())),
    }
}

/* Module decl */
#[pymodule]
fn pyrscel(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(eval, m)?)?;
    Ok(())
}

/* private functions */

fn to_pyobject(py: Python<'_>, valcel: rscel::ValueCell) -> PyObject {
    use rscel::ValueCell::*;

    match valcel {
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
        Null => py.None(),
        _ => py.None(),
    }
}
