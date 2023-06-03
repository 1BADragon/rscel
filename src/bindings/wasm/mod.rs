mod utils;

use crate::{BindContext, CelContext, Program};
use serde::Serialize;
use serde_json::Value;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, rscel-wasm!");
}

#[derive(Serialize)]
pub struct EvalResult {
    success: bool,
    result: Option<Value>,
    error: Option<Value>,
}

impl EvalResult {
    pub fn from_error<T: std::fmt::Debug>(err: T) -> EvalResult {
        EvalResult {
            success: false,
            result: None,
            error: Some(Value::from(format!("{:?}", err))),
        }
    }

    pub fn from_value(value: Value) -> EvalResult {
        EvalResult {
            success: true,
            result: Some(value),
            error: None,
        }
    }
}

#[wasm_bindgen]
pub fn cel_eval(prog: &str, binding: JsValue) -> JsValue {
    let mut ctx = CelContext::new();
    let mut exec_ctx = BindContext::new();

    if let Err(err) = ctx.add_program_str("entry", prog) {
        return serde_wasm_bindgen::to_value(&EvalResult::from_error(err)).unwrap();
    }

    if let Err(err) =
        exec_ctx.bind_params_from_json_obj(serde_wasm_bindgen::from_value(binding).unwrap())
    {
        return serde_wasm_bindgen::to_value(&EvalResult::from_error(err)).unwrap();
    }

    let res = ctx.exec("entry", &exec_ctx);
    log(&format!("{:?}", res));

    serde_wasm_bindgen::to_value(&match res {
        Ok(ok) => EvalResult::from_value(ok.into_json_value()),
        Err(err) => EvalResult::from_error(err),
    })
    .unwrap()
}

#[wasm_bindgen]
pub fn cel_details(source: &str) -> JsValue {
    match Program::from_source(source) {
        Ok(prog) => serde_wasm_bindgen::to_value(&EvalResult::from_value(
            serde_json::to_value(&prog.details()).unwrap(),
        ))
        .unwrap(),
        Err(err) => serde_wasm_bindgen::to_value(&EvalResult::from_error(err)).unwrap(),
    }
}
