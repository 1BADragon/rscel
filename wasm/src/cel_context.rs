use wasm_bindgen::prelude::*;

use rscel::{BindContext, CelContext};

use crate::{bind_context::WasmBindContext, cel_program::WasmCelProgram, types::CelEvalResult};

#[wasm_bindgen(js_name = CelContext)]
pub struct WasmCelContext {
    ctx: CelContext,
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
        let mut bind_ctx = BindContext::new();

        for (key, value) in bindings.bindings().iter() {
            bind_ctx.bind_param(key, value.clone());
        }

        for (key, func) in bindings.funcs().iter() {
            bind_ctx.bind_func(key, func);
        }

        match self.ctx.exec(name, &bind_ctx) {
            Ok(value) => CelEvalResult::from_value(value),
            Err(err) => CelEvalResult::from_error(err),
        }
    }
}
