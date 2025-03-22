use serde::{Deserialize, Serialize};

use crate::types::CelBytes;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Question,                        // ?
    Colon,                           // :
    Add,                             // +
    Minus,                           // -
    Multiply,                        // *
    Divide,                          // /
    Mod,                             // %
    Not,                             // !
    Dot,                             // .
    Comma,                           // ,
    LBracket,                        // [
    RBracket,                        // ]
    LBrace,                          // {
    RBrace,                          // }
    LParen,                          // (
    RParen,                          // )
    LessThan,                        // <
    GreaterThan,                     // >
    UnderScore,                      // _
    OrOr,                            // ||
    AndAnd,                          // &&
    LessEqual,                       // <=
    GreaterEqual,                    // >=
    EqualEqual,                      // ==
    NotEqual,                        // !=
    In,                              // 'in'
    Null,                            // 'null'
    Match,                           // 'match'
    Case,                            // 'case'
    BoolLit(bool),                   // true | false
    IntLit(u64),                     // [-+]?[0-9]+
    UIntLit(u64),                    // [0-9]+u
    FloatLit(f64),                   // [-+]?[0-9]*\.?[0-9]+([eE][-+]?[0-9]+)?
    StringLit(String),               // r?('|")[^\n]*('|")
    FStringLit(Vec<FStringSegment>), // f'.*'
    ByteStringLit(CelBytes),         // b('|")[^\n]('|")
    Ident(String),                   // [_A-Za-z][_A-Za-z0-9]*
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FStringSegment {
    Lit(String),
    Expr(String),
}

pub trait AsToken {
    fn as_token(&self) -> Option<&Token>;
}

pub trait IntoToken {
    fn into_token(self) -> Option<Token>;
}
