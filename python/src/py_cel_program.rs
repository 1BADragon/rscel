use std::borrow::Cow;

use pyo3::{exceptions::PyValueError, pyclass, pymethods, PyRefMut, PyResult};
use rscel::Program;

#[pyclass(name = "CelProgram")]
pub struct PyCelProgram {
    program: Option<Program>,
}

#[pymethods]
impl PyCelProgram {
    #[new]
    fn new() -> PyCelProgram {
        PyCelProgram { program: None }
    }

    fn add_source(mut slf: PyRefMut<'_, PyCelProgram>, source: &str) -> PyResult<()> {
        slf.program = Some(match Program::from_source(source) {
            Ok(p) => p,
            Err(e) => return Err(PyValueError::new_err(format!("{e}"))),
        });

        Ok(())
    }

    fn add_serialized_json(
        mut slf: PyRefMut<'_, PyCelProgram>,
        serialized_json: &str,
    ) -> PyResult<()> {
        match serde_json::from_str(serialized_json) {
            Ok(p) => {
                slf.program = Some(p);
                Ok(())
            }
            Err(e) => Err(PyValueError::new_err(format!("{e}"))),
        }
    }

    fn add_serialized_bincode(
        mut slf: PyRefMut<'_, PyCelProgram>,
        serialized_bincode: &[u8],
    ) -> PyResult<()> {
        match bincode::deserialize(serialized_bincode) {
            Ok(p) => {
                slf.program = Some(p);
                Ok(())
            }
            Err(e) => Err(PyValueError::new_err(format!("{e}"))),
        }
    }

    fn serialize_to_json(slf: PyRefMut<'_, PyCelProgram>) -> PyResult<String> {
        if let Some(program) = &slf.program {
            match serde_json::to_string(&program) {
                Ok(s) => Ok(s),
                Err(e) => Err(PyValueError::new_err(format!("{e}"))),
            }
        } else {
            Err(PyValueError::new_err("Program source not set"))
        }
    }

    fn serialize_to_bincode(slf: PyRefMut<'_, PyCelProgram>) -> PyResult<Cow<'_, [u8]>> {
        if let Some(program) = &slf.program {
            match bincode::serialize(program) {
                Ok(b) => Ok(Cow::Owned(b)),
                Err(e) => Err(PyValueError::new_err(format!("{e}"))),
            }
        } else {
            Err(PyValueError::new_err("Program source not set"))
        }
    }

    fn details_json(slf: PyRefMut<'_, PyCelProgram>, pretty: bool) -> PyResult<String> {
        if let Some(program) = &slf.program {
            match if pretty {
                serde_json::to_string_pretty(program.details().ast().unwrap())
            } else {
                serde_json::to_string(program.details().ast().unwrap())
            } {
                Ok(s) => Ok(s),
                Err(e) => Err(PyValueError::new_err(format!("{e}"))),
            }
        } else {
            Err(PyValueError::new_err("Program source not set"))
        }
    }
}

impl PyCelProgram {
    pub fn as_inner(&self) -> Option<&Program> {
        self.program.as_ref()
    }
}
