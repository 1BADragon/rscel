use js_sys::{Array, Function, Object};
use wasm_bindgen::{JsCast, JsValue};

use rscel::{CelError, CelValue};

use crate::from_jsvalue::WasmCelValue;

#[derive(Clone)]
pub struct CelJsCallable {
    func: Function,
}

impl CelJsCallable {
    pub fn new(func: Function) -> CelJsCallable {
        CelJsCallable { func }
    }

    /// Calls the JS function. A non-null `this` (from method-call syntax) is passed
    /// as the first argument, followed by `args`.
    pub fn call(&self, this: CelValue, args: Vec<CelValue>) -> CelValue {
        let mut collected_args: Vec<CelValue> = Vec::with_capacity(1 + args.len());

        match this {
            CelValue::Null => {}
            other => collected_args.push(other),
        }

        collected_args.extend(args);

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

    /// An owning closure for `BindContext::bind_func`, which needs a `'static` function.
    pub fn into_fn(self) -> impl Fn(CelValue, Vec<CelValue>) -> CelValue {
        move |this, args| self.call(this, args)
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
