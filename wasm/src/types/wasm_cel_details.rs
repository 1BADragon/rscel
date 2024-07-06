use rscel::ProgramDetails;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmCelDetails(ProgramDetails);

impl From<ProgramDetails> for WasmCelDetails {
    fn from(value: ProgramDetails) -> Self {
        Self(value)
    }
}
