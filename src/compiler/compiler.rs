use super::{
    parse_result::ParseResult, syntax_error::SyntaxError, tokenizer::Tokenizer, tokens::Token,
};
use crate::{ByteCode, CelError, CelResult, CelValue, Program};

pub struct CelCompiler<'l> {
    tokenizer: Tokenizer<'l>,
    source: String,
}

impl<'l> CelCompiler<'l> {
    pub fn with_input(input_str: &'l str) -> CelCompiler<'l> {
        CelCompiler {
            tokenizer: Tokenizer::with_input(input_str),
            source: input_str.to_string(),
        }
    }

    pub fn compile(mut self) -> CelResult<Program> {
        let res = self.parse_expression()?;

        if !self.tokenizer.peek()?.is_none() {
            return Err(SyntaxError::from_location(self.tokenizer.location())
                .with_message(format!("Unexpected token: {:?}", self.tokenizer.peek()?))
                .into());
        }

        let prog = res.into_program(self.source);

        #[cfg(feature = "debug_output")]
        {
            println!(
                "Built program:\n{}",
                serde_json::to_string_pretty(&prog).unwrap()
            );
        }

        Ok(prog)
    }

    fn parse_expression(&mut self) -> CelResult<ParseResult> {
        let lhs = self.parse_conditional_or()?;

        match self.tokenizer.peek()? {
            Some(Token::Question) => {
                self.tokenizer.next()?;
                let true_clause = self.parse_conditional_or()?;

                let next = self.tokenizer.next()?;
                if next != Some(Token::Colon) {
                    return Err(SyntaxError::from_location(self.tokenizer.location())
                        .with_message(format!("Unexpected token {:?}, expected COLON", next))
                        .into());
                }

                let false_clause = self.parse_expression()?;

                Ok(lhs.into_turnary(true_clause, false_clause))
            }
            _ => Ok(lhs),
        }
    }

    fn parse_conditional_or(&mut self) -> CelResult<ParseResult> {
        let lhs = self.parse_conditional_and()?;

        if let Some(Token::OrOr) = self.tokenizer.peek()? {
            self.tokenizer.next()?;
            Ok(ParseResult::with_bytecode(vec![ByteCode::Or])
                .consume_children(vec![lhs, self.parse_conditional_or()?]))
        } else {
            Ok(lhs)
        }
    }

    fn parse_conditional_and(&mut self) -> CelResult<ParseResult> {
        let lhs = self.parse_relation()?;

        if let Some(Token::AndAnd) = self.tokenizer.peek()? {
            self.tokenizer.next()?;
            Ok(ParseResult::with_bytecode(vec![ByteCode::And])
                .consume_children(vec![lhs, self.parse_conditional_and()?]))
        } else {
            Ok(lhs)
        }
    }

    fn parse_relation(&mut self) -> CelResult<ParseResult> {
        let lhs = self.parse_addition()?;

        match self.tokenizer.peek()? {
            Some(Token::LessThan) => {
                self.tokenizer.next()?;
                Ok(ParseResult::with_bytecode(vec![ByteCode::Lt])
                    .consume_children(vec![lhs, self.parse_relation()?]))
            }
            Some(Token::LessEqual) => {
                self.tokenizer.next()?;
                Ok(ParseResult::with_bytecode(vec![ByteCode::Le])
                    .consume_children(vec![lhs, self.parse_relation()?]))
            }
            Some(Token::EqualEqual) => {
                self.tokenizer.next()?;
                Ok(ParseResult::with_bytecode(vec![ByteCode::Eq])
                    .consume_children(vec![lhs, self.parse_relation()?]))
            }
            Some(Token::NotEqual) => {
                self.tokenizer.next()?;
                Ok(ParseResult::with_bytecode(vec![ByteCode::Ne])
                    .consume_children(vec![lhs, self.parse_relation()?]))
            }
            Some(Token::GreaterEqual) => {
                self.tokenizer.next()?;
                Ok(ParseResult::with_bytecode(vec![ByteCode::Ge])
                    .consume_children(vec![lhs, self.parse_relation()?]))
            }
            Some(Token::GreaterThan) => {
                self.tokenizer.next()?;
                Ok(ParseResult::with_bytecode(vec![ByteCode::Gt])
                    .consume_children(vec![lhs, self.parse_relation()?]))
            }
            Some(Token::In) => {
                self.tokenizer.next()?;
                Ok(ParseResult::with_bytecode(vec![ByteCode::In])
                    .consume_children(vec![lhs, self.parse_relation()?]))
            }
            _ => Ok(lhs),
        }
    }

    fn parse_addition(&mut self) -> CelResult<ParseResult> {
        let lhs = self.parse_multiplication()?;

        match self.tokenizer.peek()? {
            Some(Token::Add) => {
                self.tokenizer.next()?;

                Ok(ParseResult::with_bytecode(vec![ByteCode::Add])
                    .consume_children(vec![lhs, self.parse_addition()?]))
            }
            Some(Token::Minus) => {
                self.tokenizer.next()?;

                Ok(ParseResult::with_bytecode(vec![ByteCode::Sub])
                    .consume_children(vec![lhs, self.parse_addition()?]))
            }
            _ => Ok(lhs),
        }
    }

    fn parse_multiplication(&mut self) -> CelResult<ParseResult> {
        let lhs = self.parse_unary()?;

        match self.tokenizer.peek()? {
            Some(Token::Multiply) => {
                self.tokenizer.next()?;

                Ok(ParseResult::with_bytecode(vec![ByteCode::Mul])
                    .consume_children(vec![lhs, self.parse_multiplication()?]))
            }
            Some(Token::Divide) => {
                self.tokenizer.next()?;

                Ok(ParseResult::with_bytecode(vec![ByteCode::Div])
                    .consume_children(vec![lhs, self.parse_multiplication()?]))
            }
            Some(Token::Mod) => {
                self.tokenizer.next()?;

                Ok(ParseResult::with_bytecode(vec![ByteCode::Mod])
                    .consume_children(vec![lhs, self.parse_multiplication()?]))
            }
            _ => Ok(lhs),
        }
    }

    fn parse_unary(&mut self) -> CelResult<ParseResult> {
        let modifier = if let Some(Token::Not) = self.tokenizer.peek()? {
            self.parse_not_list()?
        } else if let Some(Token::Minus) = self.tokenizer.peek()? {
            self.parse_neg_list()?
        } else {
            ParseResult::new()
        };

        Ok(self.parse_member()?.append_result(modifier))
    }

    fn parse_not_list(&mut self) -> CelResult<ParseResult> {
        let mut res = ParseResult::new();

        while self.tokenizer.peek()? == Some(Token::Not) {
            self.tokenizer.next()?;
            res = res.append_bytecode(vec![ByteCode::Not])
        }

        Ok(res)
    }

    fn parse_neg_list(&mut self) -> CelResult<ParseResult> {
        let mut res = ParseResult::new();

        while self.tokenizer.peek()? == Some(Token::Minus) {
            self.tokenizer.next()?;
            res = res.append_bytecode(vec![ByteCode::Neg])
        }

        Ok(res)
    }

    fn parse_member(&mut self) -> CelResult<ParseResult> {
        Ok(self
            .parse_primary()?
            .append_result(self.parse_member_prime()?))
    }

    fn parse_member_prime(&mut self) -> CelResult<ParseResult> {
        let mut res = ParseResult::new();
        'outer: loop {
            let tmp = match self.tokenizer.peek()? {
                Some(Token::Dot) => {
                    self.tokenizer.next()?;
                    match self.tokenizer.next()? {
                        Some(Token::Ident(ident)) => ParseResult::with_bytecode(vec![
                            ByteCode::Push(CelValue::from_ident(&ident)),
                            ByteCode::Access,
                        ]),
                        Some(other) => {
                            return Err(SyntaxError::from_location(self.tokenizer.location())
                                .with_message(format!("Expected ident got {:?}", other))
                                .into())
                        }
                        None => {
                            return Err(SyntaxError::from_location(self.tokenizer.location())
                                .with_message("Expected ident got NOTHING".to_string())
                                .into())
                        }
                    }
                }
                Some(Token::LParen) => {
                    self.tokenizer.next()?;

                    match self.tokenizer.peek()? {
                        Some(Token::RParen) => {
                            self.tokenizer.next()?;
                            ParseResult::with_bytecode(vec![ByteCode::Call(0)])
                        }
                        _ => {
                            let args = self.parse_expression_list()?;

                            let token = self.tokenizer.next()?;
                            if token != Some(Token::RParen) {
                                return Err(SyntaxError::from_location(self.tokenizer.location())
                                    .with_message(format!(
                                        "Unexpected token {}, expected RPARAN",
                                        &token
                                            .map_or("NOTHING".to_string(), |x| format!("{:?}", x))
                                    ))
                                    .into());
                            } else {
                                ParseResult::with_bytecode(vec![ByteCode::Call(args.len() as u32)])
                                    .consume_call_children(args)
                            }
                        }
                    }
                }
                Some(Token::LBracket) => {
                    self.tokenizer.next()?;

                    let index = self.parse_expression()?;

                    let next_token = self.tokenizer.next()?;
                    match next_token {
                        Some(Token::RBracket) => {
                            index.append_result(ParseResult::with_bytecode(vec![ByteCode::Index]))
                        }
                        _ => {
                            return Err(SyntaxError::from_location(self.tokenizer.location())
                                .with_message(format!(
                                    "Unexpected token {}, expected RPARAN",
                                    &next_token
                                        .map_or("NOTHING".to_string(), |x| format!("{:?}", x))
                                ))
                                .into())
                        }
                    }
                }
                _ => break 'outer,
            };

            res = res.append_result(tmp);
        }

        Ok(res)
    }

    fn parse_primary(&mut self) -> CelResult<ParseResult> {
        match self.tokenizer.peek()? {
            Some(Token::Type) => {
                self.tokenizer.next()?;
                Ok(ParseResult::with_bytecode(vec![ByteCode::Push(
                    CelValue::from_ident("type"),
                )]))
            }
            Some(Token::Ident(val)) => {
                self.tokenizer.next()?;
                Ok(
                    ParseResult::with_bytecode(vec![ByteCode::Push(CelValue::from_ident(&val))])
                        .add_ident(&val),
                )
            }
            Some(Token::LParen) => {
                self.tokenizer.next()?;
                let child = self.parse_expression()?;
                if let Some(Token::RParen) = self.tokenizer.peek()? {
                    self.tokenizer.next()?;
                }

                Ok(child)
            }
            Some(Token::LBracket) => {
                self.tokenizer.next()?;
                let children = self.parse_expression_list()?;

                if let Some(Token::RBracket) = self.tokenizer.peek()? {
                    self.tokenizer.next()?;
                }

                Ok(
                    ParseResult::with_bytecode(vec![ByteCode::MkList(children.len() as u32)])
                        .consume_children(children),
                )
            }
            Some(Token::LBrace) => {
                self.tokenizer.next()?;
                let children = self.parse_obj_inits()?;

                if let Some(Token::RBrace) = self.tokenizer.peek()? {
                    self.tokenizer.next()?;
                }

                Ok(
                    ParseResult::with_bytecode(vec![ByteCode::MkDict(children.len() as u32 / 2)])
                        .consume_children(children),
                )
            }
            Some(Token::UIntLit(val)) => {
                self.tokenizer.next()?;
                Ok(ParseResult::with_bytecode(vec![ByteCode::Push(
                    CelValue::from_uint(val),
                )]))
            }
            Some(Token::IntLit(val)) => {
                self.tokenizer.next()?;
                Ok(ParseResult::with_bytecode(vec![ByteCode::Push(
                    CelValue::from_int(val),
                )]))
            }
            Some(Token::FloatLit(val)) => {
                self.tokenizer.next()?;
                Ok(ParseResult::with_bytecode(vec![ByteCode::Push(
                    CelValue::from_float(val),
                )]))
            }
            Some(Token::StringLit(val)) => {
                self.tokenizer.next()?;
                Ok(ParseResult::with_bytecode(vec![ByteCode::Push(
                    CelValue::from_string(val),
                )]))
            }
            Some(Token::ByteStringLit(val)) => {
                self.tokenizer.next()?;
                Ok(ParseResult::with_bytecode(vec![ByteCode::Push(
                    CelValue::from_bytes(val),
                )]))
            }
            Some(Token::BoolLit(val)) => {
                self.tokenizer.next()?;
                Ok(ParseResult::with_bytecode(vec![ByteCode::Push(
                    CelValue::from_bool(val),
                )]))
            }
            Some(Token::Null) => {
                self.tokenizer.next()?;
                Ok(ParseResult::with_bytecode(vec![ByteCode::Push(
                    CelValue::from_null(),
                )]))
            }
            _ => Err(SyntaxError::from_location(self.tokenizer.location())
                .with_message(format!("unexpected!!! {:?}", self.tokenizer.peek()))
                .into()),
        }
    }

    fn parse_expression_list(&mut self) -> CelResult<Vec<ParseResult>> {
        let mut vec = Vec::new();

        'outer: loop {
            vec.push(self.parse_expression()?);

            match self.tokenizer.peek()? {
                Some(Token::Comma) => {
                    self.tokenizer.next()?;
                    continue;
                }
                _ => break 'outer,
            }
        }

        Ok(vec)
    }

    fn parse_obj_inits(&mut self) -> CelResult<Vec<ParseResult>> {
        let mut vec = Vec::new();

        'outer: loop {
            let key_res = self.parse_expression()?;

            let next_token = self.tokenizer.next()?;
            if next_token != Some(Token::Colon) {
                return Err(SyntaxError::from_location(self.tokenizer.location())
                    .with_message(format!("Invalid token: expected ':' got {:?}", next_token))
                    .into());
            }
            // MkDict expects value then key
            vec.push(self.parse_expression()?);
            vec.push(key_res);

            match self.tokenizer.peek()? {
                Some(Token::Comma) => {
                    self.tokenizer.next()?;
                    continue;
                }
                _ => break 'outer,
            }
        }

        Ok(vec)
    }
}

#[cfg(test)]
mod test {
    use test_case::test_case;

    use super::CelCompiler;

    #[test_case("3+1"; "addition")]
    #[test_case("(1+foo) / 23"; "with literal")]
    #[test_case("(true || false) + 23"; "with boolean")]
    #[test_case("foo.bar"; "member access")]
    #[test_case("foo[3]"; "list access")]
    #[test_case("foo.bar()"; "member call")]
    #[test_case("foo()"; "empty function call")]
    #[test_case("foo(3)"; "function call")]
    #[test_case("1"; "just 1")]
    #[test_case("foo"; "an ident")]
    #[test_case("foo.bar.baz"; "deep member access")]
    #[test_case("--foo"; "double neg")]
    #[test_case("foo || true"; "or")]
    #[test_case("int(foo.bar && foo.baz) + 4 - (8 * 7)"; "complex")]
    #[test_case("true ? 3 : 1"; "ternary")]
    #[test_case("[1, 2, 3 + 3, 4 * 2, \"fish\"]"; "list construction")]
    fn test_parser(input: &str) {
        CelCompiler::with_input(input).compile().unwrap();
    }
}
