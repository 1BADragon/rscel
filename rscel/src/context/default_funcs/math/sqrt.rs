use crate::macros::dispatch;

pub use methods::dispatch as sqrt;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn sqrt(n: i64) -> f64 {
        (n as f64).sqrt()
    }

    fn sqrt(n: u64) -> f64 {
        (n as f64).sqrt()
    }

    fn sqrt(n: f64) -> f64 {
        n.sqrt()
    }
}
