use rscel_macro::dispatch;

pub use methods::dispatch as bytes_impl;

#[dispatch]
mod methods {
    use crate::types::CelBytes;
    use crate::CelValue;

    fn bytes(arg: String) -> CelValue {
        CelValue::from_bytes(arg.into_bytes())
    }

    fn bytes(arg: CelBytes) -> CelValue {
        CelValue::Bytes(arg)
    }
}
