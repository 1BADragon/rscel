use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Object)]
    fn keys(obj: &JsValue) -> js_sys::Array;
}

pub struct ObjectIterator {
    object: js_sys::Object,
    keys: js_sys::Array,
    index: u32,
}

impl ObjectIterator {
    pub fn new(obj: js_sys::Object) -> ObjectIterator {
        let keys = js_sys::Reflect::own_keys(&obj).unwrap();
        ObjectIterator {
            object: obj,
            keys,
            index: 0,
        }
    }
}

impl Iterator for ObjectIterator {
    type Item = (String, JsValue);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.keys.length() {
            return None;
        }

        let key = self.keys.get(self.index);
        let val = js_sys::Reflect::get(&self.object, &key).unwrap();
        self.index += 1;

        Some((key.as_string().unwrap(), val))
    }
}
