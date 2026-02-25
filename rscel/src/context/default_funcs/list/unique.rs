use rscel_macro::dispatch;

pub use methods::dispatch as unique;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn unique(this: Vec<CelValue>) -> Vec<CelValue> {
        let mut out: Vec<CelValue> = Vec::new();
        for item in this {
            if !out.contains(&item) {
                out.push(item);
            }
        }
        out
    }
}
