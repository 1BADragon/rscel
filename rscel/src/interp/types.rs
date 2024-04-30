use serde::{Deserialize, Serialize};

use crate::{CelError, CelResult, CelValue, RsCelFunction, RsCelMacro};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum JmpWhen {
    True,
    False,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ByteCode {
    Push(CelValue),
    Or,
    And,
    Not,
    Neg,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Lt,
    Le,
    Eq,
    Ne,
    Ge,
    Gt,
    In,
    Jmp(u32),
    JmpCond {
        when: JmpWhen,
        dist: u32,
        leave_val: bool,
    },
    MkList(u32),
    MkDict(u32),
    Index,
    Access,
    Call(u32),
    FmtString(u32),
}

impl fmt::Debug for ByteCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ByteCode::*;

        match self {
            Push(val) => write!(f, "PUSH {:?}", val),
            Or => write!(f, "OR"),
            And => write!(f, "AND"),
            Not => write!(f, "NOT"),
            Neg => write!(f, "NEG"),
            Add => write!(f, "ADD"),
            Sub => write!(f, "SUB"),
            Mul => write!(f, "MUL"),
            Div => write!(f, "DIV"),
            Mod => write!(f, "MOD"),
            Lt => write!(f, "LT"),
            Le => write!(f, "LE"),
            Eq => write!(f, "EQ"),
            Ne => write!(f, "NE"),
            Ge => write!(f, "GE"),
            Gt => write!(f, "GT"),
            In => write!(f, "IN"),
            Jmp(dist) => write!(f, "JMP {}", dist),
            JmpCond {
                when,
                dist,
                leave_val: _,
            } => write!(f, "JMP {:?} {}", when, dist),
            MkList(size) => write!(f, "MKLIST {}", size),
            MkDict(size) => write!(f, "MKDICT {}", size),
            Index => write!(f, "INDEX"),
            Access => write!(f, "ACCESS"),
            Call(size) => write!(f, "CALL {}", size),
            FmtString(size) => write!(f, "FMT {}", size),
        }
    }
}

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
