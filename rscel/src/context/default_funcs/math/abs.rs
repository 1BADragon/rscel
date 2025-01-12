use crate::macros::dispatch;

pub use methods::dispatch as abs;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn abs(n: i64) -> i64 {
        n.abs()
    }

    fn abs(n: u64) -> u64 {
        n
    }

    fn abs(n: f64) -> f64 {
        n.abs()
    }
}
