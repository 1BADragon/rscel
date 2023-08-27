use super::{
    grammar::*, parse_result::ParseResult, syntax_error::SyntaxError, tokenizer::Tokenizer,
    tokens::Token,
};
use crate::{interp::JmpWhen, ByteCode, CelError, CelResult, CelValue, Program};

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
        let (res, ast) = self.parse_expression()?;

        if !self.tokenizer.peek()?.is_none() {
            return Err(SyntaxError::from_location(self.tokenizer.location())
                .with_message(format!("Unexpected token: {:?}", self.tokenizer.peek()?))
                .into());
        }

        let prog = res.into_program(self.source, ast);

        #[cfg(feature = "debug_output")]
        {
            println!(
                "Built program:\n{}",
                serde_json::to_string_pretty(&prog).unwrap()
            );
        }

        Ok(prog)
    }

    fn parse_expression(&mut self) -> CelResult<(ParseResult, Expr)> {
        let (lhs, lhs_ast) = self.parse_conditional_or()?;

        match self.tokenizer.peek()? {
            Some(Token::Question) => {
                self.tokenizer.next()?;
                let (true_clause, tc_ast) = self.parse_conditional_or()?;

                let next = self.tokenizer.next()?;
                if next != Some(Token::Colon) {
                    return Err(SyntaxError::from_location(self.tokenizer.location())
                        .with_message(format!("Unexpected token {:?}, expected COLON", next))
                        .into());
                }

                let (false_clause, fc_ast) = self.parse_expression()?;

                Ok((
                    lhs.into_turnary(true_clause, false_clause),
                    Expr::Ternary {
                        condition: Box::new(lhs_ast),
                        true_clause: Box::new(tc_ast),
                        false_clause: Box::new(fc_ast),
                    },
                ))
            }
            _ => Ok((lhs, Expr::Unary(Box::new(lhs_ast)))),
        }
    }

    fn parse_conditional_or(&mut self) -> CelResult<(ParseResult, ConditionalOr)> {
        let (lhs, lhs_ast) = self.parse_conditional_and()?;

        if let Some(Token::OrOr) = self.tokenizer.peek()? {
            self.tokenizer.next()?;
            let (rhs, rhs_ast) = self.parse_conditional_or()?;
            let jmp = ParseResult::with_bytecode(vec![ByteCode::JmpCond {
                when: JmpWhen::True,
                dist: rhs.bytecode().len() as u32 + 1,
                leave_val: true,
            }]);
            Ok((
                ParseResult::with_bytecode(vec![ByteCode::Or])
                    .consume_children(vec![lhs, jmp, rhs]),
                ConditionalOr::Binary {
                    lhs: lhs_ast,
                    rhs: Box::new(rhs_ast),
                },
            ))
        } else {
            Ok((lhs, ConditionalOr::Unary(lhs_ast)))
        }
    }

    fn parse_conditional_and(&mut self) -> CelResult<(ParseResult, ConditionalAnd)> {
        let (lhs, lhs_ast) = self.parse_relation()?;

        if let Some(Token::AndAnd) = self.tokenizer.peek()? {
            self.tokenizer.next()?;
            let (rhs, rhs_ast) = self.parse_conditional_and()?;
            let jmp = ParseResult::with_bytecode(vec![ByteCode::JmpCond {
                when: JmpWhen::False,
                dist: rhs.bytecode().len() as u32 + 1,
                leave_val: true,
            }]);
            Ok((
                ParseResult::with_bytecode(vec![ByteCode::And])
                    .consume_children(vec![lhs, jmp, rhs]),
                ConditionalAnd::Binary {
                    lhs: lhs_ast,
                    rhs: Box::new(rhs_ast),
                },
            ))
        } else {
            Ok((lhs, ConditionalAnd::Unary(lhs_ast)))
        }
    }

    fn parse_relation(&mut self) -> CelResult<(ParseResult, Relation)> {
        let (lhs, lhs_ast) = self.parse_addition()?;

        match self.tokenizer.peek()? {
            Some(Token::LessThan) => {
                self.tokenizer.next()?;

                let (rhs, rhs_ast) = self.parse_relation()?;

                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::Lt]).consume_children(vec![lhs, rhs]),
                    Relation::Binary {
                        lhs: lhs_ast,
                        op: Relop::Lt,
                        rhs: Box::new(rhs_ast),
                    },
                ))
            }
            Some(Token::LessEqual) => {
                self.tokenizer.next()?;
                let (rhs, rhs_ast) = self.parse_relation()?;

                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::Le]).consume_children(vec![lhs, rhs]),
                    Relation::Binary {
                        lhs: lhs_ast,
                        op: Relop::Le,
                        rhs: Box::new(rhs_ast),
                    },
                ))
            }
            Some(Token::EqualEqual) => {
                self.tokenizer.next()?;
                let (rhs, rhs_ast) = self.parse_relation()?;

                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::Eq]).consume_children(vec![lhs, rhs]),
                    Relation::Binary {
                        lhs: lhs_ast,
                        op: Relop::Eq,
                        rhs: Box::new(rhs_ast),
                    },
                ))
            }
            Some(Token::NotEqual) => {
                self.tokenizer.next()?;
                let (rhs, rhs_ast) = self.parse_relation()?;

                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::Ne]).consume_children(vec![lhs, rhs]),
                    Relation::Binary {
                        lhs: lhs_ast,
                        op: Relop::Ne,
                        rhs: Box::new(rhs_ast),
                    },
                ))
            }
            Some(Token::GreaterEqual) => {
                self.tokenizer.next()?;
                let (rhs, rhs_ast) = self.parse_relation()?;

                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::Ge]).consume_children(vec![lhs, rhs]),
                    Relation::Binary {
                        lhs: lhs_ast,
                        op: Relop::Ge,
                        rhs: Box::new(rhs_ast),
                    },
                ))
            }
            Some(Token::GreaterThan) => {
                self.tokenizer.next()?;
                let (rhs, rhs_ast) = self.parse_relation()?;

                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::Gt]).consume_children(vec![lhs, rhs]),
                    Relation::Binary {
                        lhs: lhs_ast,
                        op: Relop::Gt,
                        rhs: Box::new(rhs_ast),
                    },
                ))
            }
            Some(Token::In) => {
                self.tokenizer.next()?;
                let (rhs, rhs_ast) = self.parse_relation()?;

                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::In]).consume_children(vec![lhs, rhs]),
                    Relation::Binary {
                        lhs: lhs_ast,
                        op: Relop::In,
                        rhs: Box::new(rhs_ast),
                    },
                ))
            }
            _ => Ok((lhs, Relation::Unary(lhs_ast))),
        }
    }

    fn parse_addition(&mut self) -> CelResult<(ParseResult, Addition)> {
        let (lhs, lhs_ast) = self.parse_multiplication()?;

        match self.tokenizer.peek()? {
            Some(Token::Add) => {
                self.tokenizer.next()?;

                let (rhs, rhs_ast) = self.parse_addition()?;

                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::Add])
                        .consume_children(vec![lhs, rhs]),
                    Addition::Binary {
                        lhs: lhs_ast,
                        op: AddOp::Add,
                        rhs: Box::new(rhs_ast),
                    },
                ))
            }
            Some(Token::Minus) => {
                self.tokenizer.next()?;

                let (rhs, rhs_ast) = self.parse_addition()?;

                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::Sub])
                        .consume_children(vec![lhs, rhs]),
                    Addition::Binary {
                        lhs: lhs_ast,
                        op: AddOp::Sub,
                        rhs: Box::new(rhs_ast),
                    },
                ))
            }
            _ => Ok((lhs, Addition::Unary(lhs_ast))),
        }
    }

    fn parse_multiplication(&mut self) -> CelResult<(ParseResult, Multiplication)> {
        let (lhs, lhs_ast) = self.parse_unary()?;

        match self.tokenizer.peek()? {
            Some(Token::Multiply) => {
                self.tokenizer.next()?;

                let (rhs, rhs_ast) = self.parse_multiplication()?;

                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::Mul])
                        .consume_children(vec![lhs, rhs]),
                    Multiplication::Binary {
                        lhs: lhs_ast,
                        op: MultOp::Mult,
                        rhs: Box::new(rhs_ast),
                    },
                ))
            }
            Some(Token::Divide) => {
                self.tokenizer.next()?;

                let (rhs, rhs_ast) = self.parse_multiplication()?;

                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::Div])
                        .consume_children(vec![lhs, rhs]),
                    Multiplication::Binary {
                        lhs: lhs_ast,
                        op: MultOp::Div,
                        rhs: Box::new(rhs_ast),
                    },
                ))
            }
            Some(Token::Mod) => {
                self.tokenizer.next()?;

                let (rhs, rhs_ast) = self.parse_multiplication()?;

                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::Mod])
                        .consume_children(vec![lhs, rhs]),
                    Multiplication::Binary {
                        lhs: lhs_ast,
                        op: MultOp::Mod,
                        rhs: Box::new(rhs_ast),
                    },
                ))
            }
            _ => Ok((lhs, Multiplication::Unary(lhs_ast))),
        }
    }

    fn parse_unary(&mut self) -> CelResult<(ParseResult, Unary)> {
        match self.tokenizer.peek()? {
            Some(Token::Not) => {
                let (not_res, not_ast) = self.parse_not_list()?;
                let (member_res, member_ast) = self.parse_member()?;

                Ok((
                    member_res.append_result(not_res),
                    Unary::NotMember {
                        nots: not_ast,
                        member: member_ast,
                    },
                ))
            }
            Some(Token::Minus) => {
                let (neg_res, neg_ast) = self.parse_neg_list()?;
                let (member_res, member_ast) = self.parse_member()?;

                Ok((
                    member_res.append_result(neg_res),
                    Unary::NegMember {
                        negs: neg_ast,
                        member: member_ast,
                    },
                ))
            }
            _ => {
                let (member, member_ast) = self.parse_member()?;

                Ok((member, Unary::Member(member_ast)))
            }
        }
    }

    fn parse_not_list(&mut self) -> CelResult<(ParseResult, NotList)> {
        let mut res = ParseResult::new();

        match self.tokenizer.peek()? {
            Some(Token::Not) => {
                self.tokenizer.next()?;
                let (nxt, ast) = self.parse_not_list()?;
                Ok((
                    nxt.append_result(ParseResult::with_bytecode(vec![ByteCode::Not])),
                    NotList::List {
                        tail: Box::new(ast),
                    },
                ))
            }
            _ => Ok((ParseResult::new(), NotList::EmptyList)),
        }
    }

    fn parse_neg_list(&mut self) -> CelResult<(ParseResult, NegList)> {
        let mut res = ParseResult::new();

        match self.tokenizer.peek()? {
            Some(Token::Minus) => {
                self.tokenizer.next()?;
                let (nxt, ast) = self.parse_neg_list()?;
                Ok((
                    nxt.append_result(ParseResult::with_bytecode(vec![ByteCode::Neg])),
                    NegList::List {
                        tail: Box::new(ast),
                    },
                ))
            }
            _ => Ok((ParseResult::new(), NegList::EmptyList)),
        }
    }

    fn parse_member(&mut self) -> CelResult<(ParseResult, Member)> {
        let (primary, primary_ast) = self.parse_primary()?;
        let (member_prime, member_prime_ast) = self.parse_member_prime()?;

        Ok((
            primary.append_result(member_prime),
            Member {
                primary: primary_ast,
                member: member_prime_ast,
            },
        ))
    }

    fn parse_member_prime(&mut self) -> CelResult<(ParseResult, MemberPrime)> {
        match self.tokenizer.peek()? {
            Some(Token::Dot) => {
                self.tokenizer.next()?;
                match self.tokenizer.next()? {
                    Some(Token::Ident(ident)) => {
                        let res = ParseResult::with_bytecode(vec![
                            ByteCode::Push(CelValue::from_ident(&ident)),
                            ByteCode::Access,
                        ]);

                        let (child_res, child_ast) = self.parse_member_prime()?;
                        Ok((
                            res.append_result(child_res),
                            MemberPrime::MemberAccess {
                                ident: Ident(ident),
                                tail: Box::new(child_ast),
                            },
                        ))
                    }
                    Some(other) => Err(SyntaxError::from_location(self.tokenizer.location())
                        .with_message(format!("Expected ident got {:?}", other))
                        .into()),
                    None => Err(SyntaxError::from_location(self.tokenizer.location())
                        .with_message("Expected ident got NOTHING".to_string())
                        .into()),
                }
            }
            Some(Token::LParen) => {
                self.tokenizer.next()?;

                let (args, args_ast) = self.parse_expression_list()?;

                let token = self.tokenizer.next()?;
                if token != Some(Token::RParen) {
                    return Err(SyntaxError::from_location(self.tokenizer.location())
                        .with_message(format!(
                            "Unexpected token {}, expected RPARAN",
                            &token.map_or("NOTHING".to_string(), |x| format!("{:?}", x))
                        ))
                        .into());
                } else {
                    let (child, child_ast) = self.parse_member_prime()?;
                    Ok((
                        ParseResult::with_bytecode(vec![ByteCode::Call(args.len() as u32)])
                            .consume_call_children(args)
                            .append_result(child),
                        MemberPrime::Call {
                            call: args_ast,
                            tail: Box::new(child_ast),
                        },
                    ))
                }
            }
            Some(Token::LBracket) => {
                self.tokenizer.next()?;

                let (index, index_ast) = self.parse_expression()?;

                let next_token = self.tokenizer.next()?;
                match next_token {
                    Some(Token::RBracket) => {
                        let (child, child_ast) = self.parse_member_prime()?;

                        Ok((
                            index
                                .append_result(ParseResult::with_bytecode(vec![ByteCode::Index]))
                                .append_result(child),
                            MemberPrime::ArrayAccess {
                                access: index_ast,
                                tail: Box::new(child_ast),
                            },
                        ))
                    }
                    _ => {
                        return Err(SyntaxError::from_location(self.tokenizer.location())
                            .with_message(format!(
                                "Unexpected token {}, expected RPARAN",
                                &next_token.map_or("NOTHING".to_string(), |x| format!("{:?}", x))
                            ))
                            .into())
                    }
                }
            }
            _ => Ok((ParseResult::new(), MemberPrime::Empty)),
        }
    }

    fn parse_primary(&mut self) -> CelResult<(ParseResult, Primary)> {
        match self.tokenizer.peek()? {
            Some(Token::Type) => {
                self.tokenizer.next()?;
                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::Push(CelValue::from_ident("type"))]),
                    Primary::Type,
                ))
            }
            Some(Token::Ident(val)) => {
                self.tokenizer.next()?;
                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::Push(CelValue::from_ident(&val))])
                        .add_ident(&val),
                    Primary::Ident(Ident(val)),
                ))
            }
            Some(Token::LParen) => {
                self.tokenizer.next()?;
                let (child, expr_ast) = self.parse_expression()?;
                if let Some(Token::RParen) = self.tokenizer.peek()? {
                    self.tokenizer.next()?;
                }

                Ok((child, Primary::Parens(expr_ast)))
            }
            Some(Token::LBracket) => {
                self.tokenizer.next()?;
                let (children, expr_list_ast) = self.parse_expression_list()?;

                if let Some(Token::RBracket) = self.tokenizer.peek()? {
                    self.tokenizer.next()?;
                }

                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::MkList(children.len() as u32)])
                        .consume_children(children),
                    Primary::ListConstruction(expr_list_ast),
                ))
            }
            Some(Token::LBrace) => {
                self.tokenizer.next()?;
                let (children, obj_inits_ast) = self.parse_obj_inits()?;

                if let Some(Token::RBrace) = self.tokenizer.peek()? {
                    self.tokenizer.next()?;
                }

                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::MkDict(children.len() as u32 / 2)])
                        .consume_children(children),
                    Primary::ObjectInit(obj_inits_ast),
                ))
            }
            Some(Token::UIntLit(val)) => {
                self.tokenizer.next()?;
                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::Push(CelValue::from_uint(val))]),
                    Primary::Literal(Literal::Unsigned(val)),
                ))
            }
            Some(Token::IntLit(val)) => {
                self.tokenizer.next()?;
                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::Push(CelValue::from_int(val))]),
                    Primary::Literal(Literal::Integer(val)),
                ))
            }
            Some(Token::FloatLit(val)) => {
                self.tokenizer.next()?;
                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::Push(CelValue::from_float(val))]),
                    Primary::Literal(Literal::Floating(val)),
                ))
            }
            Some(Token::StringLit(val)) => {
                self.tokenizer.next()?;
                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::Push(CelValue::from_string(
                        val.clone(),
                    ))]),
                    Primary::Literal(Literal::String(val)),
                ))
            }
            Some(Token::ByteStringLit(val)) => {
                self.tokenizer.next()?;
                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::Push(CelValue::from_bytes(
                        val.clone(),
                    ))]),
                    Primary::Literal(Literal::ByteString(val)),
                ))
            }
            Some(Token::BoolLit(val)) => {
                self.tokenizer.next()?;
                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::Push(CelValue::from_bool(val))]),
                    Primary::Literal(Literal::Boolean(val)),
                ))
            }
            Some(Token::Null) => {
                self.tokenizer.next()?;
                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::Push(CelValue::from_null())]),
                    Primary::Literal(Literal::Null),
                ))
            }
            _ => Err(SyntaxError::from_location(self.tokenizer.location())
                .with_message(format!("unexpected!!! {:?}", self.tokenizer.peek()))
                .into()),
        }
    }

    fn parse_expression_list(&mut self) -> CelResult<(Vec<ParseResult>, ExprList)> {
        let mut vec = Vec::new();
        let mut ast = Vec::new();

        'outer: loop {
            if self.tokenizer.peek()? == Some(Token::RParen) {
                break 'outer;
            }

            let (expr_res, expr_ast) = self.parse_expression()?;
            vec.push(expr_res);
            ast.push(expr_ast);

            match self.tokenizer.peek()? {
                Some(Token::Comma) => {
                    self.tokenizer.next()?;
                    continue;
                }
                _ => break 'outer,
            }
        }

        Ok((vec, ExprList { exprs: ast }))
    }

    fn parse_obj_inits(&mut self) -> CelResult<(Vec<ParseResult>, ObjInits)> {
        let mut vec = Vec::new();
        let mut ast = Vec::new();

        'outer: loop {
            let (key_res, key_ast) = self.parse_expression()?;

            let next_token = self.tokenizer.next()?;
            if next_token != Some(Token::Colon) {
                return Err(SyntaxError::from_location(self.tokenizer.location())
                    .with_message(format!("Invalid token: expected ':' got {:?}", next_token))
                    .into());
            }
            // MkDict expects value then key
            let (value_res, value_ast) = self.parse_expression()?;
            vec.push(value_res);
            vec.push(key_res);

            ast.push(ObjInit {
                key: key_ast,
                value: value_ast,
            });

            match self.tokenizer.peek()? {
                Some(Token::Comma) => {
                    self.tokenizer.next()?;
                    continue;
                }
                _ => break 'outer,
            }
        }

        Ok((vec, ObjInits { inits: ast }))
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
