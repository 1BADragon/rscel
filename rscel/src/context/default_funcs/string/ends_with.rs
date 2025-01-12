use crate::macros::dispatch;

pub use ends_with_i_methods::dispatch as ends_with_i;
pub use ends_with_methods::dispatch as ends_with;

#[dispatch]
mod ends_with_methods {
    use crate::CelValue;

    fn ends_with(this: String, needle: String) -> bool {
        this.ends_with(&needle)
    }
}

#[dispatch]
mod ends_with_i_methods {
    use crate::CelValue;

    fn ends_with_i(this: String, needle: String) -> bool {
        this.to_lowercase().ends_with(&needle.to_lowercase())
    }
}
