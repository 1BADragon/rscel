use crate::macros::dispatch;

pub use remove_methods::dispatch as remove;

#[dispatch]
mod remove_methods {
    use crate::CelValue;

    fn remove(mut this: String, pattern: String) -> String {
        this.remove_matches(&pattern);

        this
    }
}
