use crate::macros::dispatch;

pub use methods::dispatch as ln;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn ln(n: i64) -> f64 {
        (n as f64).ln()
    }

    fn ln(n: u64) -> f64 {
        (n as f64).ln()
    }

    fn ln(n: f64) -> f64 {
        n.ln()
    }
}
