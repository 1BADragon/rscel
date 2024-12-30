use rscel_macro::dispatch;

pub use methods::dispatch as contains;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn contains(this: String, needle: String) -> bool {
        this.contains(&needle)
    }
}
