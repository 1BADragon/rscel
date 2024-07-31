use rscel::{CelError, CelValue};
use wasm_bindgen::prelude::*;

use crate::{
    from_jsvalue::WasmCelValue,
    types::{api, WasmCelError},
};

enum CelEvalResultInner {
    Result(CelValue),
    Error(WasmCelError),
}

#[wasm_bindgen]
pub struct CelEvalResult {
    inner: CelEvalResultInner,
}

impl CelEvalResult {
    pub fn from_error(err: CelError) -> CelEvalResult {
        CelEvalResult {
            inner: CelEvalResultInner::Error(WasmCelError::new(err)),
        }
    }

    pub fn from_value(value: CelValue) -> CelEvalResult {
        CelEvalResult {
            inner: CelEvalResultInner::Result(value),
        }
    }
}

#[wasm_bindgen]
impl CelEvalResult {
    #[wasm_bindgen(js_name=isSuccess)]
    pub fn is_success(&self) -> bool {
        matches!(self.inner, CelEvalResultInner::Result(_))
    }

    #[wasm_bindgen]
    pub fn result(&self) -> Option<api::WasmCelValue> {
        match &self.inner {
            CelEvalResultInner::Result(r) => {
                Some(Into::<JsValue>::into(WasmCelValue::new(r.clone())).into())
            }
            _ => None,
        }
    }

    #[wasm_bindgen]
    pub fn error(&self) -> Option<WasmCelError> {
        match &self.inner {
            CelEvalResultInner::Error(e) => Some(e.clone()),
            _ => None,
        }
    }
}
