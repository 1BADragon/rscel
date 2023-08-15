#[derive(Debug)]
pub enum Token {
    Question,            // ?
    Colon,               // :
    Add,                 // +
    Minus,               // -
    Multiply,            // *
    Divide,              // /
    Mod,                 // %
    Not,                 // !
    Dot,                 // .
    LBracket,            // [
    RBracket,            // ]
    LBrace,              // {
    RBrace,              // }
    LParen,              // (
    RParen,              // )
    LessThan,            // <
    GreaterThan,         // >
    OrOr,                // ||
    AndAnd,              // &&
    LessEqual,           // <=
    GreaterEqual,        // >=
    EqualEqual,          // ==
    NotEqual,            // !=
    In,                  // 'in'
    Type,                // 'type'
    Null,                // 'null'
    BoolLit(bool),       // true | false
    IntLit(i64),         // [-+]?[0-9]+
    UInt(u64),           // [0-9]+u
    Float(f64),          // [-+]?[0-9]*\.?[0-9]+([eE][-+]?[0-9]+)?
    String(String),      // r?('|")[^\n]*('|")
    ByteString(Vec<u8>), // b('|")[^\n]('|")
    Ident(String),       // [_A-Za-z][_A-Za-z0-9]*
}
