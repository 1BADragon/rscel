use crate::macros::dispatch;

pub use rsplit::dispatch as rsplit;
pub use split::dispatch as split;
pub use split_at::dispatch as split_at;

#[dispatch]
mod rsplit {
    use crate::CelValue;

    fn rsplit(this: String, needle: String) -> Vec<CelValue> {
        this.rsplit(&needle).map(|s| s.into()).collect()
    }
}

#[dispatch]
mod split {
    use crate::CelValue;

    fn split(this: String, needle: String) -> Vec<CelValue> {
        this.split(&needle).map(|s| s.into()).collect()
    }
}

#[dispatch]
mod split_at {
    use crate::CelValue;

    fn split_at(this: String, at: i64) -> Vec<CelValue> {
        let (left, right) = this.split_at(at as usize);

        vec![left.into(), right.into()].into()
    }
}
