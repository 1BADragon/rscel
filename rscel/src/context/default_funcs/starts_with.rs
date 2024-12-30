use crate::macros::dispatch;

pub use methods::dispatch as starts_with;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn starts_with(this: String, needle: String) -> bool {
        this.starts_with(&needle)
    }
}
