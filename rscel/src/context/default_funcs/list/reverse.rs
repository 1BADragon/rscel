use rscel_macro::dispatch;

pub use methods::dispatch as reverse;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn reverse(mut this: Vec<CelValue>) -> Vec<CelValue> {
        this.reverse();
        this
    }
}
