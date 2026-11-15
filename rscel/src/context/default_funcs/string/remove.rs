use crate::macros::dispatch;

pub use remove_methods::dispatch as remove;

#[dispatch]
mod remove_methods {
    use crate::CelValue;

    fn remove(this: String, pattern: String) -> String {
        this.replace(&pattern, "")
    }
}
