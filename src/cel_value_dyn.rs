use std::fmt;

use crate::CelValue;

pub trait CelValueDyn: fmt::Debug + fmt::Display {
    fn as_type(&self) -> CelValue;
}
