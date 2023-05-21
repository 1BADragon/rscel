use pyo3::{prelude::*, types::PyString};
use rscel::serde_json;

#[pyclass]
pub struct PyValueCell(rscel::ValueCell);

#[pymethods]
impl PyValueCell {
    pub fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
    pub fn __str__(&self) -> String {
        format!("{:?}", self.0)
    }
}

#[pyfunction]
fn eval(prog_bytes: String, bindings: String) -> PyValueCell {
    let mut ctx = rscel::CelContext::new();
    let mut exec_ctx = rscel::ExecContext::new();

    ctx.add_program_str("entry", &prog_bytes).unwrap();
    exec_ctx.bind_params_from_json_obj(serde_json::from_str(&bindings).unwrap());

    PyValueCell(ctx.exec("entry", &exec_ctx).unwrap())
}

#[pymodule]
fn pyrscel(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyValueCell>()?;
    m.add_function(wrap_pyfunction!(eval, m)?)?;
    Ok(())
}
