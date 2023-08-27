mod object_iter;
mod utils;

use std::collections::HashMap;

use crate::{BindContext, CelCompiler, CelContext, CelError, CelValue};
use object_iter::ObjectIterator;
use serde::Serialize;
use serde_json::Value;
use wasm_bindgen::prelude::*;

use self::utils::generic_of_js;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(msg: &str);

    #[wasm_bindgen(js_namespace = Object)]
    fn keys(obj: &JsValue) -> js_sys::Array;

    #[wasm_bindgen(js_namespace = Object)]
    fn values(obj: &JsValue) -> js_sys::Array;

    #[wasm_bindgen(js_namespace = Object)]
    fn hasOwnProperty(obj: &JsValue, property: &str) -> bool;
}

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
pub struct CelFloat {
    cel_floatval: f64,
}

#[wasm_bindgen]
impl CelFloat {
    #[wasm_bindgen(constructor)]
    pub fn new(val: f64) -> Option<CelFloat> {
        Some(CelFloat { cel_floatval: val })
    }
}

#[wasm_bindgen]
pub fn cel_eval(prog: &str, binding: JsValue) -> JsValue {
    let mut ctx = CelContext::new();
    let mut exec_ctx = BindContext::new();

    if let Err(err) = ctx.add_program_str("entry", prog) {
        return serde_wasm_bindgen::to_value(&EvalResult::from_error(err)).unwrap();
    }

    for (key, value) in ObjectIterator::new(binding) {
        match value.try_into() {
            Ok(celval) => exec_ctx.bind_param(&key, celval),
            Err(err) => return serde_wasm_bindgen::to_value(&EvalResult::from_error(err)).unwrap(),
        }
    }

    let res = ctx.exec("entry", &exec_ctx);

    serde_wasm_bindgen::to_value(&match res {
        Ok(ok) => EvalResult::from_value(ok.into_json_value()),
        Err(err) => EvalResult::from_error(err),
    })
    .unwrap()
}

#[wasm_bindgen(js_name = floatVal)]
pub fn float_val(val: f64) -> Option<CelFloat> {
    CelFloat::new(val)
}

#[wasm_bindgen]
pub fn cel_details(source: &str) -> JsValue {
    match CelCompiler::with_input(source).compile() {
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

impl TryFrom<JsValue> for CelValue {
    type Error = CelError;
    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        if value.is_object() {
            match generic_of_js::<CelFloat>(value.clone(), "CelFloat") {
                Ok(cel_float) => Ok(CelValue::from_float(cel_float.cel_floatval)),
                Err(_) => {
                    let mut map = HashMap::new();

                    for (key, value) in ObjectIterator::new(value) {
                        map.insert(key, value.try_into()?);
                    }

                    Ok(CelValue::from_map(map))
                }
            }
        } else if let Some(numval) = value.dyn_ref::<js_sys::Number>() {
            if numval
                .to_string(10)
                .is_ok_and(|x| x.as_string().is_some_and(|s| s.contains('.')))
            {
                Ok(CelValue::from_float(numval.value_of()))
            } else {
                Ok(CelValue::from_int(numval.value_of() as i64))
            }
        } else if value.is_string() {
            Ok(CelValue::from_string(value.as_string().unwrap()))
        } else if value.is_array() {
            let mut list: Vec<CelValue> = Vec::new();

            for list_value in values(&value).into_iter() {
                list.push(list_value.try_into()?);
            }

            Ok(CelValue::from_list(list))
        } else if value.is_null() || value.is_undefined() {
            Ok(CelValue::from_null())
        } else if value.is_truthy() {
            Ok(CelValue::from_bool(true))
        } else if value.is_falsy() {
            Ok(CelValue::from_bool(false))
        } else {
            Err(CelError::value("Unknown js binding"))
        }
    }
}
