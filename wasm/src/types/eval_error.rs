use wasm_bindgen::JsValue;

use rscel::CelError;

pub struct WasmCelError {
    inner: CelError,
}

impl WasmCelError {
    pub fn new(inner: CelError) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> CelError {
        self.inner
    }
}

#[derive(Clone)]
pub struct WasmEvalError {
    kind: String,
    msg: String,
    err: CelError,
}

impl Into<JsValue> for WasmEvalError {
    fn into(self) -> JsValue {
        let obj = js_sys::Object::new();

        js_sys::Reflect::set(&obj, &"kind".into(), &self.kind.into()).unwrap();
        js_sys::Reflect::set(&obj, &"msg".into(), &self.msg.into()).unwrap();
        js_sys::Reflect::set(&obj, &"err".into(), &WasmCelError::new(self.err).into()).unwrap();

        obj.into()
    }
}

impl Into<WasmEvalError> for WasmCelError {
    fn into(self) -> WasmEvalError {
        let inner = self.into_inner();
        WasmEvalError {
            kind: inner.type_string().to_owned(),
            msg: inner.to_string(),
            err: inner,
        }
    }
}
