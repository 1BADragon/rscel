use wasm_bindgen::JsValue;

use rscel::CelError;

pub struct EvalError {
    kind: String,
    msg: String,
    err: CelError,
}

impl Into<JsValue> for EvalError {
    fn into(self) -> JsValue {
        let obj = js_sys::Object::new();

        js_sys::Reflect::set(&obj, &"kind".into(), &self.kind.into()).unwrap();
        js_sys::Reflect::set(&obj, &"msg".into(), &self.msg.into()).unwrap();
        js_sys::Reflect::set(&obj, &"err".into(), &self.err.into()).unwrap();

        obj.into()
    }
}

impl Into<EvalError> for CelError {
    fn into(self) -> EvalError {
        EvalError {
            kind: self.type_string().to_owned(),
            msg: self.to_string(),
            err: self,
        }
    }
}
