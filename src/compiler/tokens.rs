#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Question,               // ?
    Colon,                  // :
    Add,                    // +
    Minus,                  // -
    Multiply,               // *
    Divide,                 // /
    Mod,                    // %
    Not,                    // !
    Dot,                    // .
    Comma,                  // ,
    LBracket,               // [
    RBracket,               // ]
    LBrace,                 // {
    RBrace,                 // }
    LParen,                 // (
    RParen,                 // )
    LessThan,               // <
    GreaterThan,            // >
    OrOr,                   // ||
    AndAnd,                 // &&
    LessEqual,              // <=
    GreaterEqual,           // >=
    EqualEqual,             // ==
    NotEqual,               // !=
    In,                     // 'in'
    Int,                    // 'int'
    Uint,                   // 'uint'
    Float,                  // 'float'
    Bool,                   // 'bool'
    String,                 // 'string'
    Bytes,                  // 'bytes'
    NullType,               // 'null_type'
    Type,                   // 'type'
    Timestamp,              // 'timestamp'
    Duration,               // 'duration'
    Null,                   // 'null'
    BoolLit(bool),          // true | false
    IntLit(i64),            // [-+]?[0-9]+
    UIntLit(u64),           // [0-9]+u
    FloatLit(f64),          // [-+]?[0-9]*\.?[0-9]+([eE][-+]?[0-9]+)?
    StringLit(String),      // r?('|")[^\n]*('|")
    ByteStringLit(Vec<u8>), // b('|")[^\n]('|")
    Ident(String),          // [_A-Za-z][_A-Za-z0-9]*
}
