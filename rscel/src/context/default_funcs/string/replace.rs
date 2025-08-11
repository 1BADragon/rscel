use crate::macros::dispatch;

pub use replace::dispatch as replace;

#[dispatch]
mod replace {
    use crate::CelValue;

    fn replace(this: String, needle: String, to: String) -> String {
        this.replace(&needle, &to)
    }
}
