use wasm_bindgen::prelude::*;

use rscel::CelError;

#[wasm_bindgen(js_name=CelError)]
#[derive(Clone)]
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

#[wasm_bindgen(js_class=CelError)]
impl WasmCelError {
    #[wasm_bindgen(js_name=toString)]
    pub fn to_string(&self) -> String {
        format!("{}", self.inner)
    }

    #[wasm_bindgen(js_name=errorData)]
    pub fn error_data(&self) -> JsValue {
        let val = js_sys::Object::new();

        match &self.inner {
            CelError::Misc(err) => {
                js_sys::Reflect::set(&val, &"type".into(), &"misc".into()).unwrap();
                js_sys::Reflect::set(&val, &"msg".into(), &err.into()).unwrap();
            }
            CelError::Syntax(err) => {
                js_sys::Reflect::set(&val, &"type".into(), &"syntax".into()).unwrap();
                js_sys::Reflect::set(&val, &"line".into(), &err.loc().line().into()).unwrap();
                js_sys::Reflect::set(&val, &"column".into(), &err.loc().col().into()).unwrap();

                match err.message() {
                    Some(msg) => js_sys::Reflect::set(&val, &"msg".into(), &msg.into()).unwrap(),
                    None => {
                        js_sys::Reflect::set(&val, &"msg".into(), &JsValue::undefined()).unwrap()
                    }
                };
            }
            CelError::Value(msg) => {
                js_sys::Reflect::set(&val, &"type".into(), &"value".into()).unwrap();
                js_sys::Reflect::set(&val, &"msg".into(), &msg.into()).unwrap();
            }
            CelError::Argument(msg) => {
                js_sys::Reflect::set(&val, &"type".into(), &"argument".into()).unwrap();
                js_sys::Reflect::set(&val, &"msg".into(), &msg.into()).unwrap();
            }
            CelError::InvalidOp(msg) => {
                js_sys::Reflect::set(&val, &"type".into(), &"invalidOp".into()).unwrap();
                js_sys::Reflect::set(&val, &"msg".into(), &msg.into()).unwrap();
            }
            CelError::Runtime(msg) => {
                js_sys::Reflect::set(&val, &"type".into(), &"runtime".into()).unwrap();
                js_sys::Reflect::set(&val, &"msg".into(), &msg.into()).unwrap();
            }
            CelError::Binding { symbol } => {
                js_sys::Reflect::set(&val, &"type".into(), &"binding".into()).unwrap();
                js_sys::Reflect::set(&val, &"symbol".into(), &symbol.into()).unwrap();
            }
            CelError::Internal(msg) => {
                js_sys::Reflect::set(&val, &"type".into(), &"internal".into()).unwrap();
                js_sys::Reflect::set(&val, &"msg".into(), &msg.into()).unwrap();
            }
            CelError::Attribute { parent, field } => {
                js_sys::Reflect::set(&val, &"type".into(), &"attribute".into()).unwrap();
                js_sys::Reflect::set(&val, &"parent".into(), &parent.into()).unwrap();
                js_sys::Reflect::set(&val, &"field".into(), &field.into()).unwrap();
            }
            CelError::DivideByZero => {
                js_sys::Reflect::set(&val, &"type".into(), &"divide by zero".into()).unwrap();
            }
        };

        val.into()
    }
}
