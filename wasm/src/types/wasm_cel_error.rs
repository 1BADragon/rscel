use rscel::CelError;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmCelError(CelError);

#[wasm_bindgen]
impl WasmCelError {
    pub fn to_string(&self) -> String {
        format!("{}", self.0)
    }
}

impl From<CelError> for WasmCelError {
    fn from(value: CelError) -> Self {
        Self(value)
    }
}
