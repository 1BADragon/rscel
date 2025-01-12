use crate::macros::dispatch;

pub use methods::dispatch as log;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn log(n: i64) -> i64 {
        n.ilog10() as i64
    }

    fn log(n: u64) -> u64 {
        n.ilog10() as u64
    }

    fn log(n: f64) -> f64 {
        n.log10()
    }
}
