use wasm_bindgen::prelude::*;

use rscel::{BindContext, CelContext, InterpRuntime};

use crate::{
    bind_context::WasmBindContext, cel_program::WasmCelProgram, interp_runtime::WasmInterpRuntime,
    types::CelEvalResult,
};

#[wasm_bindgen(js_name = CelContext)]
pub struct WasmCelContext {
    ctx: CelContext,
}

impl Default for WasmCelContext {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = CelContext)]
impl WasmCelContext {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmCelContext {
        WasmCelContext {
            ctx: CelContext::new(),
        }
    }

    #[wasm_bindgen(js_name = addProgramStr)]
    pub fn add_program_str(&mut self, name: &str, source: &str) -> Result<(), JsValue> {
        self.ctx
            .add_program_str(name, source)
            .map_err(|err| JsValue::from_str(&err.to_string()))
    }

    #[wasm_bindgen(js_name = addProgram)]
    pub fn add_program(&mut self, name: &str, program: &WasmCelProgram) -> Result<(), JsValue> {
        match program.as_inner() {
            Some(prog) => {
                self.ctx.add_program(name, prog.clone());
                Ok(())
            }
            None => Err(JsValue::from_str("Program not populated")),
        }
    }

    #[wasm_bindgen]
    pub fn exec(&mut self, name: &str, bindings: &WasmBindContext) -> CelEvalResult {
        self.exec_impl(name, bindings, None)
    }

    /// Like `exec`, but `now()` returns `runtime.startTime`. The runtime is not consumed,
    /// so it can be reused to replay several executions with the same time.
    #[wasm_bindgen(js_name = execWithRuntime)]
    pub fn exec_with_runtime(
        &mut self,
        name: &str,
        bindings: &WasmBindContext,
        runtime: &WasmInterpRuntime,
    ) -> CelEvalResult {
        self.exec_impl(name, bindings, Some(runtime.inner().clone()))
    }
}

impl WasmCelContext {
    fn exec_impl(
        &mut self,
        name: &str,
        bindings: &WasmBindContext,
        runtime: Option<InterpRuntime>,
    ) -> CelEvalResult {
        let mut bind_ctx = BindContext::new();

        for (key, value) in bindings.bindings().iter() {
            bind_ctx.bind_param(key, value.clone());
        }

        // bind_func borrows, so the closures must outlive bind_ctx's use below.
        let mut funcs = Vec::with_capacity(bindings.funcs().len());
        for (key, callable) in bindings.funcs().iter() {
            funcs.push((key, callable.clone().into_fn()));
        }
        for (key, func) in funcs.iter() {
            bind_ctx.bind_func(key, func);
        }

        match self.ctx.exec_with_rt(name, &bind_ctx, runtime).0 {
            Ok(value) => CelEvalResult::from_value(value),
            Err(err) => CelEvalResult::from_error(err),
        }
    }
}
