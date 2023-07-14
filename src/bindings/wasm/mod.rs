mod utils;

use crate::{BindContext, CelContext, CelError, Program};
use serde::Serialize;
use serde_json::Value;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
pub struct EvalError {
    kind: String,
    msg: String,
    err: CelError,
}

#[derive(Serialize)]
pub struct EvalResult {
    success: bool,
    result: Option<Value>,
    error: Option<EvalError>,
}

impl EvalResult {
    pub fn from_error(err: CelError) -> EvalResult {
        EvalResult {
            success: false,
            result: None,
            error: Some(EvalError {
                kind: err.type_string().to_owned(),
                msg: err.to_string(),
                err,
            }),
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

    serde_wasm_bindgen::to_value(&match res {
        Ok(ok) => EvalResult::from_value(ok.into_json_value()),
        Err(err) => EvalResult::from_error(err),
    })
    .unwrap()
}

#[wasm_bindgen]
pub fn cel_details(source: &str) -> JsValue {
    match Program::from_source(source) {
        Ok(prog) => {
            let mut details = prog.details().clone();
            let default_bindings = BindContext::new();

            details.filter_from_bindings(&default_bindings);

            serde_wasm_bindgen::to_value(&EvalResult::from_value(
                serde_json::to_value(&details).unwrap(),
            ))
            .unwrap()
        }
        Err(err) => serde_wasm_bindgen::to_value(&EvalResult::from_error(err)).unwrap(),
    }
}
