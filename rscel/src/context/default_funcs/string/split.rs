use crate::macros::dispatch;

pub use rsplit_methods::dispatch as rsplit;
pub use split_at_methods::dispatch as split_at;
pub use split_methods::dispatch as split;

#[dispatch]
mod rsplit_methods {
    use crate::CelValue;

    fn rsplit(this: String, needle: String) -> Vec<CelValue> {
        this.rsplit(&needle).map(|s| s.into()).collect()
    }
}

#[dispatch]
mod split_methods {
    use crate::CelValue;

    fn split(this: String, needle: String) -> Vec<CelValue> {
        this.split(&needle).map(|s| s.into()).collect()
    }

    fn split(this: String, needle: String, limit: i64) -> Vec<CelValue> {
        if limit <= 0 {
            return Vec::new();
        }
        this.splitn(limit as usize, &needle)
            .map(|s| s.into())
            .collect()
    }
}

#[dispatch]
mod split_at_methods {
    use crate::CelValue;

    fn split_at(this: String, at: i64) -> Vec<CelValue> {
        let (left, right) = this.split_at(at as usize);

        vec![left.into(), right.into()]
    }

    fn split_at(this: String, at: u64) -> Vec<CelValue> {
        let (left, right) = this.split_at(at as usize);

        vec![left.into(), right.into()]
    }
}
