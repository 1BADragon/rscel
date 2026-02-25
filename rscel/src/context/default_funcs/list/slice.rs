use rscel_macro::dispatch;

pub use methods::dispatch as slice;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn slice(this: Vec<CelValue>, start: i64, end: i64) -> CelValue {
        internal::slice(this, start as isize, end as isize)
    }

    fn slice(this: Vec<CelValue>, start: u64, end: u64) -> CelValue {
        internal::slice(this, start as isize, end as isize)
    }

    fn slice(this: Vec<CelValue>, start: i64, end: u64) -> CelValue {
        internal::slice(this, start as isize, end as isize)
    }

    fn slice(this: Vec<CelValue>, start: u64, end: i64) -> CelValue {
        internal::slice(this, start as isize, end as isize)
    }

    mod internal {
        use crate::{CelError, CelValue};

        pub fn slice(this: Vec<CelValue>, start: isize, end: isize) -> CelValue {
            let len = this.len() as isize;

            let resolve = |idx: isize| -> isize {
                if idx < 0 {
                    (len + idx).max(0)
                } else {
                    idx.min(len)
                }
            };

            let start = resolve(start) as usize;
            let end = resolve(end) as usize;

            if start > end {
                return CelValue::from_err(CelError::argument(
                    "slice() start index must be <= end index",
                ));
            }

            this.into_iter().skip(start).take(end - start).collect::<Vec<_>>().into()
        }
    }
}
