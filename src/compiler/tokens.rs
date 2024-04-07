use num_traits::Num;

use super::tokenizer::SyntaxError;

#[derive(Debug, Clone, PartialEq)]
pub struct NumericLiteral<T: Num + std::ops::Neg<Output = T>> {
    pub value: Option<T>,
    pub str_value: String,
    pub base: u32,
    pub location: (usize, usize),
}

impl<T: Num + std::ops::Neg<Output = T>> NumericLiteral<T> {
    pub fn resolve(self, is_neg: bool) -> Result<T, SyntaxError> {
        match self.value {
            Some(t) => {
                if is_neg {
                    Ok(-t)
                } else {
                    Ok(t)
                }
            }
            None => {
                let mut neg_str_value = if is_neg {
                    "-".to_string()
                } else {
                    String::new()
                };

                neg_str_value.push_str(&self.str_value);
                match T::from_str_radix(&neg_str_value, self.base) {
                    Ok(val) => Ok(val),
                    Err(_) => Err(
                        SyntaxError::from_location(self.location).with_message(format!(
                            "Failed to parse numeric literal {}",
                            self.str_value
                        )),
                    ),
                }
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Question,                      // ?
    Colon,                         // :
    Add,                           // +
    Minus,                         // -
    Multiply,                      // *
    Divide,                        // /
    Mod,                           // %
    Not,                           // !
    Dot,                           // .
    Comma,                         // ,
    LBracket,                      // [
    RBracket,                      // ]
    LBrace,                        // {
    RBrace,                        // }
    LParen,                        // (
    RParen,                        // )
    LessThan,                      // <
    GreaterThan,                   // >
    OrOr,                          // ||
    AndAnd,                        // &&
    LessEqual,                     // <=
    GreaterEqual,                  // >=
    EqualEqual,                    // ==
    NotEqual,                      // !=
    In,                            // 'in'
    Null,                          // 'null'
    BoolLit(bool),                 // true | false
    IntLit(NumericLiteral<i64>),   // [-+]?[0-9]+
    UIntLit(u64),                  // [0-9]+u
    FloatLit(NumericLiteral<f64>), // [-+]?[0-9]*\.?[0-9]+([eE][-+]?[0-9]+)?
    StringLit(String),             // r?('|")[^\n]*('|")
    ByteStringLit(Vec<u8>),        // b('|")[^\n]('|")
    Ident(String),                 // [_A-Za-z][_A-Za-z0-9]*
}
