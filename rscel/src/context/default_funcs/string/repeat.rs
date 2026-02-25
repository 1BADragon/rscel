use crate::macros::dispatch;

pub use methods::dispatch as repeat;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn repeat(this: String, n: i64) -> String {
        this.repeat(n as usize)
    }

    fn repeat(this: String, n: u64) -> String {
        this.repeat(n as usize)
    }
}
