mod from_jsvalue;
mod into_jsvalue;
mod object_iter;
mod types;
mod utils;
mod wasm_program_details;

use from_jsvalue::WasmCelValue;
use object_iter::ObjectIterator;
use rscel::{BindContext, CelCompiler, CelContext, StringTokenizer};
use types::{api, EvalResult, WasmCelError};
use wasm_bindgen::prelude::*;
use wasm_program_details::WasmProgramDetails;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(msg: &str);

    #[wasm_bindgen(js_namespace = Object)]
    fn keys(obj: &JsValue) -> js_sys::Array;

    #[wasm_bindgen(js_namespace = Object)]
    fn values(obj: &JsValue) -> js_sys::Array;

    #[wasm_bindgen(js_namespace = Object)]
    fn hasOwnProperty(obj: &JsValue, property: &str) -> bool;
}

#[wasm_bindgen]
pub fn cel_eval(prog: &str, binding: api::WasmCelBinding) -> EvalResult {
    let mut ctx = CelContext::new();
    let mut exec_ctx = BindContext::new();

    if let Err(err) = ctx.add_program_str("entry", prog) {
        return EvalResult::from_error(WasmCelError::new(err).into()).into();
    }

    let binding_js: JsValue = binding.into();

    for (key, value) in ObjectIterator::new(binding_js.into()) {
        match TryInto::<WasmCelValue>::try_into(value) {
            Ok(celval) => exec_ctx.bind_param(&key, celval.into_inner()),
            Err(err) => return EvalResult::from_error(WasmCelError::new(err).into()).into(),
        }
    }

    let res = ctx.exec("entry", &exec_ctx);

    match res {
        Ok(ok) => EvalResult::from_value(WasmCelValue::new(ok)),
        Err(err) => EvalResult::from_error(WasmCelError::new(err).into()),
    }
}

#[wasm_bindgen]
pub fn cel_details(source: &str) -> Result<api::WasmProgramDetails, api::WasmEvalError> {
    let mut tokenizer = StringTokenizer::with_input(source);
    match CelCompiler::with_tokenizer(&mut tokenizer).compile() {
        Ok(prog) => {
            let default_bindings = BindContext::new();

            let mut details = prog.into_details();
            details.filter_from_bindings(&default_bindings);

            Ok(Into::<JsValue>::into(WasmProgramDetails::new(details)).into())
        }
        Err(err) => Err(Into::<JsValue>::into(WasmCelError::new(err)).into()),
    }
}
