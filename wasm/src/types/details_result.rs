use super::api;
use crate::types::{wasm_program_details::WasmProgramDetails, WasmCelError};
use rscel::{CelError, ProgramDetails};
use wasm_bindgen::prelude::*;

enum DetailsResultInner {
    Success(ProgramDetails),
    Error(WasmCelError),
}

#[wasm_bindgen]
pub struct CelDetailsResult {
    inner: DetailsResultInner,
}

impl CelDetailsResult {
    pub fn from_details(details: ProgramDetails) -> CelDetailsResult {
        CelDetailsResult {
            inner: DetailsResultInner::Success(details),
        }
    }

    pub fn from_error(error: CelError) -> CelDetailsResult {
        CelDetailsResult {
            inner: DetailsResultInner::Error(WasmCelError::new(error)),
        }
    }
}

#[wasm_bindgen]
impl CelDetailsResult {
    #[wasm_bindgen(js_name=isSuccess)]
    pub fn is_success(&self) -> bool {
        matches!(self.inner, DetailsResultInner::Success(_))
    }

    #[wasm_bindgen]
    pub fn details(&self) -> Option<api::WasmProgramDetails> {
        match &self.inner {
            DetailsResultInner::Success(s) => {
                Some(Into::<JsValue>::into(WasmProgramDetails::new(s.clone())).into())
            }
            _ => None,
        }
    }

    #[wasm_bindgen]
    pub fn error(&self) -> Option<WasmCelError> {
        match &self.inner {
            DetailsResultInner::Error(e) => Some(e.clone()),
            _ => None,
        }
    }
}
