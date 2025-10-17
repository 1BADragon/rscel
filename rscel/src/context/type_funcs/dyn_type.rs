use rscel_macro::dispatch;

pub use methods::dispatch as dyn_impl;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn into_dyn(arg: CelValue) -> CelValue {
        arg
    }
}
