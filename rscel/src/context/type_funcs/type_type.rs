use rscel_macro::dispatch;

pub use methods::dispatch as type_impl;

#[dispatch]
mod methods {
    use crate::{CelValue, CelValueDyn};

    fn ty(arg: CelValue) -> CelValue {
        arg.as_type()
    }
}
