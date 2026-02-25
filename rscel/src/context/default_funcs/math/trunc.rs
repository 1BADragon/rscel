use crate::macros::dispatch;

pub use methods::dispatch as trunc;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn trunc(n: i64) -> i64 {
        n
    }

    fn trunc(n: u64) -> u64 {
        n
    }

    fn trunc(n: f64) -> i64 {
        n.trunc() as i64
    }
}
