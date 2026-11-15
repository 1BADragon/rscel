use crate::{compiler::tokens::Token, ByteCode};

use super::MatchCmpOp;

pub enum PrefixPattern {
    Eq,
    Neq,
    Gt,
    Ge,
    Lt,
    Le,
}

impl PrefixPattern {
    pub fn from_token(token: &Token) -> Option<Self> {
        match token {
            Token::EqualEqual => Some(PrefixPattern::Eq),
            Token::NotEqual => Some(PrefixPattern::Neq),
            Token::GreaterThan => Some(PrefixPattern::Gt),
            Token::GreaterEqual => Some(PrefixPattern::Ge),
            Token::LessThan => Some(PrefixPattern::Lt),
            Token::LessEqual => Some(PrefixPattern::Le),
            _ => None,
        }
    }

    pub fn as_bytecode(&self) -> ByteCode {
        match self {
            PrefixPattern::Eq => ByteCode::Eq,
            PrefixPattern::Neq => ByteCode::Ne,
            PrefixPattern::Gt => ByteCode::Gt,
            PrefixPattern::Ge => ByteCode::Ge,
            PrefixPattern::Lt => ByteCode::Lt,
            PrefixPattern::Le => ByteCode::Le,
        }
    }

    pub fn as_ast(&self) -> MatchCmpOp {
        match self {
            PrefixPattern::Eq => MatchCmpOp::Eq,
            PrefixPattern::Neq => MatchCmpOp::Neq,
            PrefixPattern::Gt => MatchCmpOp::Gt,
            PrefixPattern::Ge => MatchCmpOp::Ge,
            PrefixPattern::Lt => MatchCmpOp::Lt,
            PrefixPattern::Le => MatchCmpOp::Le,
        }
    }
}
