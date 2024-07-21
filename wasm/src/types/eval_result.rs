use wasm_bindgen::prelude::*;

use crate::{from_jsvalue::WasmCelValue, types::api};

use super::eval_error::WasmEvalError;

#[wasm_bindgen]
pub struct EvalResult {
    result: Option<WasmCelValue>,
    error: Option<WasmEvalError>,
}

impl EvalResult {
    pub fn from_error(err: WasmEvalError) -> EvalResult {
        EvalResult {
            result: None,
            error: Some(err),
        }
    }

    pub fn from_value(value: WasmCelValue) -> EvalResult {
        EvalResult {
            result: Some(value),
            error: None,
        }
    }
}

#[wasm_bindgen]
impl EvalResult {
    #[wasm_bindgen]
    pub fn is_success(&self) -> bool {
        return self.result.is_some();
    }

    #[wasm_bindgen]
    pub fn result(&self) -> Option<api::WasmCelValue> {
        self.result
            .as_ref()
            .map(|v| Into::<JsValue>::into(v.clone()).into())
    }

    #[wasm_bindgen]
    pub fn error(&self) -> Option<api::WasmEvalError> {
        self.error
            .as_ref()
            .map(|v| Into::<JsValue>::into(v.clone()).into())
    }
}
