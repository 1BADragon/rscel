use super::{ast::Ast, Lexicons};

enum ParserItem {
    Token(Lexicons),
    Ast(Ast),
}

pub type ParserCallback = fn(&[ParserItem]) -> Ast;
