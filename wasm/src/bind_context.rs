use std::collections::HashMap;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use rscel::CelValue;

use crate::{cel_js_callable::CelJsCallable, from_jsvalue::WasmCelValue};

#[wasm_bindgen(js_name = BindContext)]
pub struct WasmBindContext {
    bindings: HashMap<String, CelValue>,
    funcs: HashMap<String, CelJsCallable>,
}

#[wasm_bindgen(js_class = BindContext)]
impl WasmBindContext {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmBindContext {
        WasmBindContext {
            bindings: HashMap::new(),
            funcs: HashMap::new(),
        }
    }

    #[wasm_bindgen(js_name = bindParam)]
    pub fn bind_param(&mut self, name: &str, value: JsValue) -> Result<(), JsValue> {
        match WasmCelValue::try_from(value) {
            Ok(val) => {
                self.bindings.insert(name.to_owned(), val.into_inner());
                Ok(())
            }
            Err(err) => Err(JsValue::from_str(&err.to_string())),
        }
    }

    #[wasm_bindgen(js_name = bindFunc)]
    pub fn bind_func(&mut self, name: &str, func: JsValue) -> Result<(), JsValue> {
        match func.dyn_into::<js_sys::Function>() {
            Ok(function) => {
                self.funcs
                    .insert(name.to_owned(), CelJsCallable::new(function));
                Ok(())
            }
            Err(_) => Err(JsValue::from_str("bindFunc expects a function")),
        }
    }

    #[wasm_bindgen]
    pub fn bind(&mut self, name: &str, value: JsValue) -> Result<(), JsValue> {
        if value.is_function() {
            self.bind_func(name, value)
        } else {
            self.bind_param(name, value)
        }
    }
}

impl WasmBindContext {
    pub(crate) fn bindings(&self) -> &HashMap<String, CelValue> {
        &self.bindings
    }

    pub(crate) fn funcs(&self) -> &HashMap<String, CelJsCallable> {
        &self.funcs
    }
}
