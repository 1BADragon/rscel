use std::str::FromStr;

use rscel::ProgramDetails;
use wasm_bindgen::prelude::*;

pub struct WasmProgramDetails(ProgramDetails);

impl WasmProgramDetails {
    pub fn new(inner: ProgramDetails) -> WasmProgramDetails {
        WasmProgramDetails(inner)
    }
}

impl From<ProgramDetails> for WasmProgramDetails {
    fn from(value: ProgramDetails) -> Self {
        WasmProgramDetails(value)
    }
}

impl From<WasmProgramDetails> for JsValue {
    fn from(value: WasmProgramDetails) -> Self {
        let js = js_sys::Object::new();

        let dets = value.0;

        if let Some(source) = dets.source() {
            js_sys::Reflect::set(
                &js,
                &js_sys::JsString::from_str("source").unwrap(),
                &js_sys::JsString::from_str(source).unwrap(),
            )
            .unwrap();
        }

        let list = js_sys::Array::new();
        for param in dets.params().into_iter() {
            list.push(&js_sys::JsString::from_str(param).unwrap().into());
        }
        js_sys::Reflect::set(
            &js,
            &js_sys::JsString::from_str("params").unwrap(),
            &list.into(),
        )
        .unwrap();

        if let Some(ast) = dets.ast() {
            js_sys::Reflect::set(
                &js,
                &js_sys::JsString::from_str("ast").unwrap(),
                &serde_wasm_bindgen::to_value(ast).unwrap(),
            )
            .unwrap();
        }

        js.into()
    }
}
