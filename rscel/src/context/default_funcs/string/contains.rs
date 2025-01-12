use rscel_macro::dispatch;

pub use contains_i_methods::dispatch as contains_i;
pub use contains_methods::dispatch as contains;

#[dispatch]
mod contains_methods {
    use crate::CelValue;

    fn contains(this: String, needle: String) -> bool {
        this.contains(&needle)
    }
}

#[dispatch]
mod contains_i_methods {
    use crate::CelValue;

    fn contains_i(this: String, needle: String) -> bool {
        this.to_lowercase().contains(&needle.to_lowercase())
    }
}
