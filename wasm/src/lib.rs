mod bind_context;
mod cel_context;
mod cel_js_callable;
mod cel_program;
mod from_jsvalue;
mod interp_runtime;
mod into_jsvalue;
mod object_iter;
mod types;
mod utils;

use cel_js_callable::CelJsCallable;
use from_jsvalue::WasmCelValue;
use interp_runtime::WasmInterpRuntime;
use object_iter::ObjectIterator;
use rscel::{BindContext, CelCompiler, CelContext, CelError, InterpRuntime, StringTokenizer};
use types::{api, CelDetailsResult, CelEvalResult};
use wasm_bindgen::{prelude::*, JsCast, JsValue};

pub use bind_context::WasmBindContext;
pub use cel_context::WasmCelContext;
pub use cel_program::WasmCelProgram;

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

#[wasm_bindgen(js_name = celEval)]
pub fn cel_eval(prog: &str, binding: api::WasmCelBinding) -> CelEvalResult {
    cel_eval_impl(prog, binding, None)
}

/// Like `celEval`, but `now()` returns `runtime.startTime`. The runtime is not consumed,
/// so it can be reused to replay several evaluations with the same time.
#[wasm_bindgen(js_name = celEvalWithRuntime)]
pub fn cel_eval_with_runtime(
    prog: &str,
    binding: api::WasmCelBinding,
    runtime: &WasmInterpRuntime,
) -> CelEvalResult {
    cel_eval_impl(prog, binding, Some(runtime.inner().clone()))
}

fn cel_eval_impl(
    prog: &str,
    binding: api::WasmCelBinding,
    runtime: Option<InterpRuntime>,
) -> CelEvalResult {
    let mut ctx = CelContext::new();
    let mut exec_ctx = BindContext::new();

    if let Err(err) = ctx.add_program_str("entry", prog) {
        return CelEvalResult::from_error(err);
    }

    let binding_js: JsValue = binding.into();
    let mut callables: Vec<(String, CelJsCallable)> = Vec::new();

    for (key, value) in ObjectIterator::new(binding_js.into()) {
        if value.is_function() {
            match value.dyn_into::<js_sys::Function>() {
                Ok(function) => callables.push((key, CelJsCallable::new(function))),
                Err(_) => {
                    return CelEvalResult::from_error(CelError::value(
                        "bindFunc expects a callable value",
                    ))
                }
            }
        } else {
            match TryInto::<WasmCelValue>::try_into(value) {
                Ok(celval) => exec_ctx.bind_param(&key, celval.into_inner()),
                Err(err) => return CelEvalResult::from_error(err),
            }
        }
    }

    // bind_func borrows, so the closures must outlive exec_ctx's use below.
    let mut funcs = Vec::with_capacity(callables.len());
    for (name, callable) in callables {
        funcs.push((name, callable.into_fn()));
    }
    for (name, func) in funcs.iter() {
        exec_ctx.bind_func(name, func);
    }

    match ctx.exec_with_rt("entry", &exec_ctx, runtime).0 {
        Ok(ok) => CelEvalResult::from_value(ok),
        Err(err) => CelEvalResult::from_error(err),
    }
}

#[wasm_bindgen(js_name = celDetails)]
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
