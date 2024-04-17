use wasm_bindgen::JsValue;

use rscel::{CelValue, Program, ProgramDetails};

use super::eval_error::EvalError;

pub struct EvalResult {
    success: bool,
    program_details: Option<ProgramDetails>,
    result: Option<CelValue>,
    error: Option<EvalError>,
}

impl EvalResult {
    pub fn from_error(err: EvalError) -> EvalResult {
        EvalResult {
            success: false,
            program_details: None,
            result: None,
            error: Some(err),
        }
    }

    pub fn from_value(value: CelValue) -> EvalResult {
        EvalResult {
            success: true,
            program_details: None,
            result: Some(value),
            error: None,
        }
    }

    pub fn from_program(value: Program) -> EvalResult {
        EvalResult {
            success: true,
            program_details: Some(value.into_details()),
            result: None,
            error: None,
        }
    }
}

impl Into<JsValue> for EvalResult {
    fn into(self) -> JsValue {
        let obj = js_sys::Object::new();

        if self.success {
            js_sys::Reflect::set(
                &obj,
                &"result".into(),
                &self.result.map_or(JsValue::undefined(), |x| x.into()),
            )
            .unwrap();
        } else {
            js_sys::Reflect::set(
                &obj,
                &"error".into(),
                &self.error.map_or(JsValue::undefined(), |x| x.into()),
            )
            .unwrap();
        }

        js_sys::Reflect::set(
            &obj,
            &"details".into(),
            &self
                .program_details
                .map_or(JsValue::undefined(), |x| x.into()),
        )
        .unwrap();

        js_sys::Reflect::set(&obj, &"success".into(), &self.success.into()).unwrap();

        obj.into()
    }
}
