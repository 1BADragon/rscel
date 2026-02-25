use crate::{CelError, CelValue};

pub mod flatten;
pub mod reverse;
pub mod slice;
pub mod unique;

pub fn sum_impl(this: CelValue, args: Vec<CelValue>) -> CelValue {
    let vals: &[CelValue] = if let CelValue::List(ref l) = this {
        l.as_slice()
    } else {
        args.as_slice()
    };

    if vals.is_empty() {
        return CelValue::from_err(CelError::argument("sum() requires at least one element"));
    }

    let mut acc = vals[0].clone();
    for val in &vals[1..] {
        acc = acc + val.clone();
        if let CelValue::Err(_) = acc {
            return acc;
        }
    }
    acc
}
