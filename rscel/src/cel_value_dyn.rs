use std::{any::Any, fmt};

use crate::CelValue;

// unsure how much i want to support with this...
// im thinking I allow object like things only for the first iteration and
// slowly move towards the whole CelValue's set of operations
pub trait CelValueDyn: fmt::Debug + fmt::Display + Send + Sync {
    fn as_type(&self) -> CelValue;
    fn access(&self, key: &str) -> CelValue;
    fn eq(&self, rhs: &CelValue) -> CelValue;
    fn is_truthy(&self) -> bool;
    fn any_ref<'a>(&'a self) -> &'a dyn Any;
}
