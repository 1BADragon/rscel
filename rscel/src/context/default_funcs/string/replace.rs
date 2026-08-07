use crate::macros::dispatch;

pub use replace::dispatch as replace;

#[dispatch]
mod replace {
    use crate::CelValue;

    fn replace(this: String, needle: String, to: String) -> String {
        this.replace(&needle, &to)
    }

    fn replace(this: String, needle: String, to: String, count: i64) -> String {
        this.replacen(&needle, &to, count as usize)
    }
}
