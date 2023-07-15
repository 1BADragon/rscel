use wasm_bindgen::{prelude::*, JsValue};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Object)]
    fn keys(obj: &JsValue) -> js_sys::Array;

    #[wasm_bindgen(js_namespace = Object)]
    fn get(obj: &JsValue, key: &str) -> JsValue;
}

pub struct ObjectIterator {
    object: JsValue,
    keys: js_sys::Array,
    index: u32,
}

impl ObjectIterator {
    pub fn new(obj: JsValue) -> ObjectIterator {
        let keys = keys(&obj);
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

        let key = self.keys.get(self.index).as_string().unwrap();
        let val = get(&self.object, &key);
        self.index += 1;

        Some((key, val))
    }
}
