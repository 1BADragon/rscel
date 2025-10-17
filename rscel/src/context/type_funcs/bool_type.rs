use rscel_macro::dispatch;

pub use methods::dispatch as bool_impl;

fn parse_bool_literal(s: &str) -> Option<bool> {
    match s {
        "1" | "t" | "true" | "TRUE" | "True" => Some(true),
        "0" | "f" | "false" | "FALSE" | "False" => Some(false),
        _ => None,
    }
}

#[dispatch]
mod methods {
    use super::parse_bool_literal;
    use crate::{CelError, CelResult, CelValue};

    #[cfg(feature = "type_prop")]
    use crate::types::cel_value_dyn::CelValueDyn;

    fn bool(arg: bool) -> bool {
        arg
    }

    fn bool(arg: String) -> CelResult<bool> {
        if let Some(value) = parse_bool_literal(&arg) {
            Ok(value)
        } else if cfg!(feature = "type_prop") {
            Ok(!arg.is_empty())
        } else {
            Err(CelError::Value(format!(
                "value '{}' cannot be converted to bool",
                arg
            )))
        }
    }

    #[cfg(feature = "type_prop")]
    fn bool(arg: CelValue) -> bool {
        arg.is_truthy()
    }
}
