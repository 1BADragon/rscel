use crate::macros::dispatch;

pub use methods::dispatch as exp;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn exp(n: i64) -> f64 {
        (n as f64).exp()
    }

    fn exp(n: u64) -> f64 {
        (n as f64).exp()
    }

    fn exp(n: f64) -> f64 {
        n.exp()
    }
}
