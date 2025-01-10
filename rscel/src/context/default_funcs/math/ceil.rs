use crate::macros::dispatch;

pub use methods::dispatch as ceil;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn ceil(n: i64) -> i64 {
        n
    }

    fn ceil(n: u64) -> u64 {
        n
    }

    fn ceil(n: f64) -> i64 {
        n.ceil() as i64
    }
}
