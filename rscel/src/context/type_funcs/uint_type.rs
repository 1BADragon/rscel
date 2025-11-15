use rscel_macro::dispatch;

pub use methods::dispatch as uint_impl;

#[dispatch]
mod methods {
    use crate::{CelError, CelResult, CelValue};

    fn uint(arg: u64) -> u64 {
        arg
    }

    fn uint(arg: i64) -> u64 {
        arg as u64
    }

    fn uint(arg: f64) -> u64 {
        arg as u64
    }

    fn uint(arg: bool) -> u64 {
        if arg {
            1
        } else {
            0
        }
    }

    fn uint(arg: String) -> CelResult<u64> {
        arg.parse::<u64>()
            .map_err(|_| CelError::value(&format!("int conversion invalid for \"{}\"", arg)))
    }
}
