use js_sys::{Array, Function, Object};
use wasm_bindgen::{JsCast, JsValue};

use rscel::{CelError, CelValue};

use crate::from_jsvalue::WasmCelValue;

pub struct CelJsCallable {
    func: Function,
}

impl CelJsCallable {
    pub fn new(func: Function) -> CelJsCallable {
        CelJsCallable { func }
    }

    fn invoke(&self, this: CelValue, args: Vec<CelValue>) -> CelValue {
        let mut collected_args: Vec<CelValue> = Vec::with_capacity(1 + args.len());

        match this {
            CelValue::Null => {}
            other => collected_args.push(other),
        }

        collected_args.extend(args.into_iter());

        let js_args: Vec<JsValue> = collected_args
            .into_iter()
            .map(|value| Into::<JsValue>::into(WasmCelValue::new(value)))
            .collect();

        let arg_array = Array::new();
        for arg in js_args.iter() {
            arg_array.push(arg);
        }

        match self.func.apply(&JsValue::NULL, &arg_array) {
            Ok(result) => match WasmCelValue::try_from(result) {
                Ok(value) => value.into_inner(),
                Err(err) => CelValue::from_err(err),
            },
            Err(err) => CelValue::from_err(CelError::runtime(&stringify_js_error(err))),
        }
    }
}

fn stringify_js_error(err: JsValue) -> String {
    if let Some(s) = err.as_string() {
        return s;
    }

    if let Some(obj) = err.dyn_ref::<Object>() {
        if let Some(s) = Object::to_string(obj).as_string() {
            return s;
        }
    }

    format!("{err:?}")
}

impl FnOnce<(CelValue, Vec<CelValue>)> for CelJsCallable {
    type Output = CelValue;

    extern "rust-call" fn call_once(self, args: (CelValue, Vec<CelValue>)) -> Self::Output {
        self.invoke(args.0, args.1)
    }
}

impl FnMut<(CelValue, Vec<CelValue>)> for CelJsCallable {
    extern "rust-call" fn call_mut(&mut self, args: (CelValue, Vec<CelValue>)) -> Self::Output {
        self.invoke(args.0, args.1)
    }
}

impl Fn<(CelValue, Vec<CelValue>)> for CelJsCallable {
    extern "rust-call" fn call(&self, args: (CelValue, Vec<CelValue>)) -> Self::Output {
        self.invoke(args.0, args.1)
    }
}
