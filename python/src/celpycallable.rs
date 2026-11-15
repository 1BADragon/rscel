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

    /// Calls the Python function. A non-null `this` (from method-call syntax) is passed
    /// as the first positional argument, followed by `args`.
    pub fn call(&self, this: CelValue, args: Vec<CelValue>) -> CelValue {
        Python::attach(|py| {
            match self.func.call(
                py,
                PyTuple::new(
                    py,
                    [this]
                        .iter()
                        .filter(|x| !x.is_null())
                        .map(|x| {
                            PyCelValueRef::new(x)
                                .into_pyobject(py)
                                .map(|o| o.into_any())
                        })
                        .chain(
                            args.into_iter()
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

    /// An owning closure for `BindContext::bind_func`, which needs a `'static` function.
    pub fn into_fn(self) -> impl Fn(CelValue, Vec<CelValue>) -> CelValue {
        move |this, args| self.call(this, args)
    }
}

impl Clone for CelPyCallable {
    fn clone(&self) -> Self {
        Python::attach(|py| CelPyCallable::new(self.func.clone_ref(py)))
    }
}
