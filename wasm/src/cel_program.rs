use wasm_bindgen::prelude::*;

use rscel::Program;

#[wasm_bindgen(js_name = CelProgram)]
pub struct WasmCelProgram {
    program: Option<Program>,
}

#[wasm_bindgen(js_class = CelProgram)]
impl WasmCelProgram {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmCelProgram {
        WasmCelProgram { program: None }
    }

    #[wasm_bindgen(js_name = addSource)]
    pub fn add_source(&mut self, source: &str) -> Result<(), JsValue> {
        match Program::from_source(source) {
            Ok(program) => {
                self.program = Some(program);
                Ok(())
            }
            Err(err) => Err(JsValue::from_str(&err.to_string())),
        }
    }

    #[wasm_bindgen(js_name = addSerializedJson)]
    pub fn add_serialized_json(&mut self, serialized_json: &str) -> Result<(), JsValue> {
        match serde_json::from_str(serialized_json) {
            Ok(program) => {
                self.program = Some(program);
                Ok(())
            }
            Err(err) => Err(JsValue::from_str(&err.to_string())),
        }
    }

    #[wasm_bindgen(js_name = addSerializedBincode)]
    pub fn add_serialized_bincode(
        &mut self,
        serialized_bincode: js_sys::Uint8Array,
    ) -> Result<(), JsValue> {
        let mut buf = vec![0u8; serialized_bincode.length() as usize];
        serialized_bincode.copy_to(&mut buf[..]);

        match bincode::deserialize::<Program>(&buf) {
            Ok(program) => {
                self.program = Some(program);
                Ok(())
            }
            Err(err) => Err(JsValue::from_str(&err.to_string())),
        }
    }

    #[wasm_bindgen(js_name = serializeToJson)]
    pub fn serialize_to_json(&self) -> Result<String, JsValue> {
        match &self.program {
            Some(program) => match serde_json::to_string(program) {
                Ok(json) => Ok(json),
                Err(err) => Err(JsValue::from_str(&err.to_string())),
            },
            None => Err(JsValue::from_str("Program source not set")),
        }
    }

    #[wasm_bindgen(js_name = serializeToBincode)]
    pub fn serialize_to_bincode(&self) -> Result<js_sys::Uint8Array, JsValue> {
        match &self.program {
            Some(program) => match bincode::serialize(program) {
                Ok(bytes) => Ok(js_sys::Uint8Array::from(bytes.as_slice())),
                Err(err) => Err(JsValue::from_str(&err.to_string())),
            },
            None => Err(JsValue::from_str("Program source not set")),
        }
    }

    #[wasm_bindgen(js_name = detailsJson)]
    pub fn details_json(&self, pretty: bool) -> Result<String, JsValue> {
        let program = self
            .program
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Program source not set"))?;

        let details = program.details();
        let ast = details
            .ast()
            .ok_or_else(|| JsValue::from_str("Program AST not available"))?;

        if pretty {
            serde_json::to_string_pretty(ast)
        } else {
            serde_json::to_string(ast)
        }
        .map_err(|err| JsValue::from_str(&err.to_string()))
    }
}

impl WasmCelProgram {
    pub(crate) fn as_inner(&self) -> Option<&Program> {
        self.program.as_ref()
    }
}
