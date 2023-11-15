use wasm_bindgen::JsValue;

use crate::{program::ProgramDetails, CelError, CelValue};

impl Into<JsValue> for CelValue {
    fn into(self) -> JsValue {
        match self {
            CelValue::Int(i) => i.into(),
            CelValue::UInt(u) => u.into(),
            CelValue::Float(f) => f.into(),
            CelValue::Bool(b) => b.into(),
            CelValue::String(s) => s.into(),
            CelValue::Bytes(b) => {
                let arr = js_sys::Uint8Array::new_with_length(b.len() as u32);

                for (index, byte) in b.into_iter().enumerate() {
                    arr.set(&(byte as u64).into(), index as u32);
                }

                arr.into()
            }
            CelValue::List(l) => {
                let arr = js_sys::Array::new();

                for value in l.into_iter() {
                    arr.push(&value.into());
                }

                arr.into()
            }
            CelValue::Map(m) => {
                let obj = js_sys::Object::new();

                for (key, value) in m.into_iter() {
                    js_sys::Reflect::set(&obj, &key.into(), &value.into()).unwrap();
                }

                obj.into()
            }
            CelValue::Null => JsValue::undefined(),
            CelValue::Ident(ident) => {
                let obj = js_sys::Object::new();

                js_sys::Reflect::set(&obj, &"ident".into(), &ident.into()).unwrap();

                obj.into()
            }
            CelValue::Type(t) => {
                let obj = js_sys::Object::new();

                js_sys::Reflect::set(&obj, &"type".into(), &t.into()).unwrap();

                obj.into()
            }
            CelValue::TimeStamp(t) => js_sys::Date::new(&t.to_rfc3339().into()).into(),
            CelValue::Duration(d) => {
                let obj = js_sys::Object::new();

                js_sys::Reflect::set(&obj, &"sec".into(), &d.num_seconds().into()).unwrap();
                js_sys::Reflect::set(&obj, &"nsec".into(), &d.num_nanoseconds().into()).unwrap();

                obj.into()
            }
            CelValue::ByteCode(_) => js_sys::Object::new().into(),
        }
    }
}

impl Into<JsValue> for ProgramDetails {
    fn into(self) -> JsValue {
        let obj = js_sys::Object::new();

        js_sys::Reflect::set(&obj, &"source".into(), &self.source().into()).unwrap();

        let params_arr = js_sys::Array::new();

        for param in self.params().into_iter() {
            params_arr.push(&param.into());
        }

        js_sys::Reflect::set(&obj, &"params".into(), &params_arr.into()).unwrap();

        // im not in the mood to write to JsValue for all of the grammer right now
        if let Some(ast) = self.ast() {
            let ast_jsvalue = serde_wasm_bindgen::to_value(ast).unwrap();
            js_sys::Reflect::set(&obj, &"ast".into(), &ast_jsvalue).unwrap();
        }

        obj.into()
    }
}

impl Into<JsValue> for CelError {
    fn into(self) -> JsValue {
        let val = js_sys::Object::new();

        match self {
            CelError::Misc(err) => {
                js_sys::Reflect::set(&val, &"type".into(), &"misc".into()).unwrap();
                js_sys::Reflect::set(&val, &"msg".into(), &err.into()).unwrap();
            }
            CelError::Syntax(err) => {
                js_sys::Reflect::set(&val, &"type".into(), &"syntax".into()).unwrap();
                js_sys::Reflect::set(&val, &"line".into(), &err.line.into()).unwrap();
                js_sys::Reflect::set(&val, &"column".into(), &err.column.into()).unwrap();

                match err.message() {
                    Some(msg) => {
                        js_sys::Reflect::set(&val, &"message".into(), &msg.into()).unwrap()
                    }
                    None => js_sys::Reflect::set(&val, &"message".into(), &JsValue::undefined())
                        .unwrap(),
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
        };

        val.into()
    }
}
