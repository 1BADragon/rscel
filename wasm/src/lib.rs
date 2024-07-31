mod from_jsvalue;
mod into_jsvalue;
mod object_iter;
mod types;
mod utils;

use from_jsvalue::WasmCelValue;
use object_iter::ObjectIterator;
use rscel::{BindContext, CelCompiler, CelContext, StringTokenizer};
use types::{api, CelDetailsResult, CelEvalResult};
use wasm_bindgen::prelude::*;

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

#[wasm_bindgen(js_name=celEval)]
pub fn cel_eval(prog: &str, binding: api::WasmCelBinding) -> CelEvalResult {
    let mut ctx = CelContext::new();
    let mut exec_ctx = BindContext::new();

    if let Err(err) = ctx.add_program_str("entry", prog) {
        return CelEvalResult::from_error(err);
    }

    let binding_js: JsValue = binding.into();

    for (key, value) in ObjectIterator::new(binding_js.into()) {
        match TryInto::<WasmCelValue>::try_into(value) {
            Ok(celval) => exec_ctx.bind_param(&key, celval.into_inner()),
            Err(err) => return CelEvalResult::from_error(err),
        }
    }

    let res = ctx.exec("entry", &exec_ctx);

    match res {
        Ok(ok) => CelEvalResult::from_value(ok),
        Err(err) => CelEvalResult::from_error(err),
    }
}

#[wasm_bindgen(js_name=celDetails)]
pub fn cel_details(source: &str) -> CelDetailsResult {
    let mut tokenizer = StringTokenizer::with_input(source);
    match CelCompiler::with_tokenizer(&mut tokenizer).compile() {
        Ok(prog) => {
            let default_bindings = BindContext::new();

            let mut details = prog.into_details();
            details.filter_from_bindings(&default_bindings);

            CelDetailsResult::from_details(details)
        }
        Err(err) => CelDetailsResult::from_error(err),
    }
}
