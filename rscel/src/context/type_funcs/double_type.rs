use rscel_macro::dispatch;

pub use methods::dispatch as double_impl;

#[dispatch]
mod methods {
    use crate::{CelError, CelResult, CelValue};

    fn double(arg: f64) -> f64 {
        arg
    }

    fn double(arg: i64) -> f64 {
        arg as f64
    }

    fn double(arg: u64) -> f64 {
        arg as f64
    }

    fn double(arg: bool) -> f64 {
        if arg {
            1.0
        } else {
            0.0
        }
    }

    fn double(arg: String) -> CelResult<f64> {
        arg.parse::<f64>()
            .map_err(|_| CelError::value(&format!("int conversion invalid for \"{}\"", arg)))
    }
}
