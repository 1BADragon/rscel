use rscel_macro::dispatch;

pub use methods::dispatch as flatten;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn flatten(this: Vec<CelValue>) -> Vec<CelValue> {
        let mut out = Vec::new();
        for item in this {
            match item {
                CelValue::List(inner) => out.extend(inner),
                other => out.push(other),
            }
        }
        out
    }
}
