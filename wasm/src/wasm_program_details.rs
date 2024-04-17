use rscel::ProgramDetails;
use wasm_bindgen::JsValue;

pub struct WasmProgramDetails {
    inner: ProgramDetails,
}

impl WasmProgramDetails {
    pub fn new(inner: ProgramDetails) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> ProgramDetails {
        self.inner
    }
}

impl Into<JsValue> for WasmProgramDetails {
    fn into(self) -> JsValue {
        let details = self.into_inner();
        let obj = js_sys::Object::new();

        js_sys::Reflect::set(&obj, &"source".into(), &details.source().into()).unwrap();

        let params_arr = js_sys::Array::new();

        for param in details.params().into_iter() {
            params_arr.push(&param.into());
        }

        js_sys::Reflect::set(&obj, &"params".into(), &params_arr.into()).unwrap();

        // im not in the mood to write to JsValue for all of the grammer right now
        if let Some(ast) = details.ast() {
            let ast_jsvalue = serde_wasm_bindgen::to_value(ast).unwrap();
            js_sys::Reflect::set(&obj, &"ast".into(), &ast_jsvalue).unwrap();
        }

        obj.into()
    }
}
