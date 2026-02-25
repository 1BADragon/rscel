use crate::macros::dispatch;

pub use methods::dispatch as cbrt;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn cbrt(n: i64) -> f64 {
        (n as f64).cbrt()
    }

    fn cbrt(n: u64) -> f64 {
        (n as f64).cbrt()
    }

    fn cbrt(n: f64) -> f64 {
        n.cbrt()
    }
}
