use crate::macros::dispatch;

pub use methods::dispatch as floor;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn floor(n: i64) -> i64 {
        n
    }

    fn floor(n: u64) -> u64 {
        n
    }

    fn floor(n: f64) -> i64 {
        n.floor() as i64
    }
}
