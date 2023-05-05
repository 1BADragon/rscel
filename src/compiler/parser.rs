use super::{
    lexer::Parser,
    types::{ast::Ast, errors::SemanticError},
};

pub fn parse_str(prog_str: &str) -> Result<Box<Ast>, SemanticError> {
    let mut parser = Parser::with_input(prog_str);

    parse_expr(parser)
}

pub fn parse_expr(parser: Parser) -> Result<Box<Ast>, SemanticError> {
    let children: Vec<Box<Ast>> = Vec::new();
}
