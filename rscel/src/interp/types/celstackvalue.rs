use crate::{CelError, CelResult, CelValue};

use super::RsCallable;

#[derive(Debug, Clone)]
pub enum CelStackValue<'a> {
    Value(CelValue),
    BoundCall {
        callable: RsCallable<'a>,
        value: CelValue,
    },
}

impl<'a> CelStackValue<'a> {
    pub fn into_value(self) -> CelResult<CelValue> {
        match self {
            CelStackValue::Value(val) => Ok(val),
            _ => Err(CelError::internal("Expected value")),
        }
    }

    pub fn as_value(&'a self) -> CelResult<&'a CelValue> {
        match self {
            CelStackValue::Value(val) => Ok(val),
            _ => Err(CelError::internal("Expected value")),
        }
    }

    pub fn as_bound_call(&'a self) -> Option<(&'a RsCallable<'a>, &'a CelValue)> {
        match self {
            CelStackValue::BoundCall { callable, value } => Some((callable, value)),
            _ => None,
        }
    }
}

impl<'a> Into<CelStackValue<'a>> for CelValue {
    fn into(self) -> CelStackValue<'a> {
        CelStackValue::Value(self)
    }
}

impl<'a> TryInto<CelValue> for CelStackValue<'a> {
    type Error = CelError;
    fn try_into(self) -> Result<CelValue, Self::Error> {
        if let CelStackValue::Value(val) = self {
            Ok(val)
        } else {
            Err(CelError::internal("Expected value 2"))
        }
    }
}
