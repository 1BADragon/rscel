use crate::macros::dispatch;

pub use remove::dispatch as remove;

#[dispatch]
mod remove {
    use crate::CelValue;

    fn remove(mut this: String, pattern: String) -> String {
        this.remove_matches(&pattern);

        this
    }
}
