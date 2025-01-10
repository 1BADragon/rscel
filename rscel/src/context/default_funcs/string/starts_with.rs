use crate::macros::dispatch;

pub use starts_with_i_methods::dispatch as starts_with_i;
pub use starts_with_methods::dispatch as starts_with;

#[dispatch]
mod starts_with_methods {
    use crate::CelValue;

    fn starts_with(this: String, needle: String) -> bool {
        this.starts_with(&needle)
    }
}

#[dispatch]
mod starts_with_i_methods {
    use crate::CelValue;

    fn starts_with_i(this: String, needle: String) -> bool {
        this.to_lowercase().starts_with(&needle.to_lowercase())
    }
}
