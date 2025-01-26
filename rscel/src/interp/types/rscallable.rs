use std::fmt;

use crate::{RsCelFunction, RsCelMacro};

/// Wrapper enum that contains either an RsCelCallable or an RsCelFunction. Used
/// as a ValueCell value.
#[derive(Clone)]
pub enum RsCallable<'a> {
    Function(&'a RsCelFunction),
    Macro(&'a RsCelMacro),
}

impl<'a> fmt::Debug for RsCallable<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Function(_) => write!(f, "Function"),
            Self::Macro(_) => write!(f, "Macro"),
        }
    }
}

impl<'a> PartialEq for RsCallable<'a> {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}
