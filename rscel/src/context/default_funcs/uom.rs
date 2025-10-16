use rscel_macro::dispatch;

pub use methods::dispatch as uom_convert;

#[dispatch]
pub mod methods {
    use crate::{context::default_funcs::uom::uom_convert_internal, CelValue};

    fn uom_convert(base: u64, from: String, to: String) -> f64 {
        uom_convert_internal(base as f64, &from, &to)
    }

    fn uom_convert(base: i64, from: String, to: String) -> f64 {
        uom_convert_internal(base as f64, &from, &to)
    }

    fn uom_convert(base: f64, from: String, to: String) -> f64 {
        uom_convert_internal(base as f64, &from, &to)
    }
}

fn uom_convert_internal(base: f64, from: &str, to: &str) -> f64 {
    todo!()
}
