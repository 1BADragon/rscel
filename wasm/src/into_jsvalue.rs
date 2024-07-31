use crate::from_jsvalue::WasmCelValue;
use rscel::CelValue;
use wasm_bindgen::JsValue;

impl Into<JsValue> for WasmCelValue {
    fn into(self) -> JsValue {
        match self.into_inner() {
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
                    arr.push(&WasmCelValue::new(value).into());
                }

                arr.into()
            }
            CelValue::Map(m) => {
                let obj = js_sys::Object::new();

                for (key, value) in m.into_iter() {
                    js_sys::Reflect::set(&obj, &key.into(), &WasmCelValue::new(value).into())
                        .unwrap();
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
                js_sys::Reflect::set(&obj, &"nsec".into(), &d.subsec_nanos().into()).unwrap();

                obj.into()
            }
            CelValue::ByteCode(_) => js_sys::Object::new().into(),
            _ => unimplemented!(),
        }
    }
}
