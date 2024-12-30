use rscel_macro::dispatch;

pub use methods::dispatch as contains_l;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn contains_i(this: String, needle: String) -> bool {
        this.to_lowercase().contains(&needle.to_lowercase())
    }
}
