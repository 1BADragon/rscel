use crate::macros::dispatch;

pub use methods::dispatch as split_whitespace;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn split_whitespace(this: String) -> Vec<CelValue> {
        this.split_whitespace().map(|t| t.into()).collect()
    }
}
