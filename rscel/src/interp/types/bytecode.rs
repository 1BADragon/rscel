use std::fmt;

use serde::{Deserialize, Serialize};

use crate::CelValue;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum JmpWhen {
    True,
    False,
}

impl JmpWhen {
    pub fn as_bool(&self) -> bool {
        match self {
            JmpWhen::True => true,
            JmpWhen::False => false,
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ByteCode {
    Push(CelValue),
    Pop,
    Test,
    Dup,
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
    Jmp(i32),
    JmpCond { when: JmpWhen, dist: i32 },
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
            Pop => write!(f, "POP"),
            Test => write!(f, "TEST"),
            Dup => write!(f, "DUP"),
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
            JmpCond { when, dist } => write!(f, "JMP {:?} {}", when, dist),
            MkList(size) => write!(f, "MKLIST {}", size),
            MkDict(size) => write!(f, "MKDICT {}", size),
            Index => write!(f, "INDEX"),
            Access => write!(f, "ACCESS"),
            Call(size) => write!(f, "CALL {}", size),
            FmtString(size) => write!(f, "FMT {}", size),
        }
    }
}
