use crate::macros::dispatch;

pub use methods::dispatch as pow;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn pow(n1: i64, n2: i64) -> i64 {
        n1.pow(n2 as u32)
    }

    fn pow(n1: i64, n2: u64) -> i64 {
        n1.pow(n2 as u32)
    }

    fn pow(n1: i64, n2: f64) -> i64 {
        n1.pow(n2 as u32)
    }

    fn pow(n1: u64, n2: i64) -> u64 {
        n1.pow(n2 as u32)
    }

    fn pow(n1: u64, n2: u64) -> u64 {
        n1.pow(n2 as u32)
    }

    fn pow(n1: u64, n2: f64) -> u64 {
        n1.pow(n2 as u32)
    }

    fn pow(n1: f64, n2: i64) -> f64 {
        n1.powi(n2 as i32)
    }

    fn pow(n1: f64, n2: u64) -> f64 {
        n1.powi(n2 as i32)
    }

    fn pow(n1: f64, n2: f64) -> f64 {
        n1.powf(n2)
    }
}
