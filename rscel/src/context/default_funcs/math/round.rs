use crate::macros::dispatch;

pub use methods::dispatch as round;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn round(n: i64) -> i64 {
        n
    }

    fn round(n: u64) -> u64 {
        n
    }

    fn round(n: f64) -> i64 {
        n.round() as i64
    }
}
