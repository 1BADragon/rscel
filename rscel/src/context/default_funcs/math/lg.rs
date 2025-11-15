use crate::macros::dispatch;

pub use methods::dispatch as lg;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn lg(n: i64) -> i64 {
        n.ilog2() as i64
    }

    fn lg(n: u64) -> u64 {
        n.ilog2() as u64
    }

    fn lg(n: f64) -> f64 {
        n.log2()
    }
}
