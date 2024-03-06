use super::{
    ast_node::AstNode, grammar::*, parse_result::ParseResult, syntax_error::SyntaxError,
    tokenizer::Tokenizer, tokens::Token,
};
use crate::{interp::JmpWhen, ByteCode, CelResult, CelValue, Program};

pub struct CelCompiler<'l> {
    tokenizer: &'l mut dyn Tokenizer,
}

impl<'l> CelCompiler<'l> {
    pub fn with_tokenizer(tokenizer: &'l mut dyn Tokenizer) -> CelCompiler<'l> {
        CelCompiler { tokenizer }
    }

    pub fn compile(mut self) -> CelResult<Program> {
        let (res, ast) = self.parse_expression()?;

        if !self.tokenizer.peek()?.is_none() {
            return Err(SyntaxError::from_location(self.tokenizer.location())
                .with_message(format!("Unexpected token: {:?}", self.tokenizer.peek()?))
                .into());
        }

        let prog = res.into_program(self.tokenizer.source().to_string(), ast);

        Ok(prog)
    }

    fn parse_expression(&mut self) -> CelResult<(ParseResult, AstNode<Expr>)> {
        let start_location = self.tokenizer.location();
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
                    AstNode::new(
                        Expr::Ternary {
                            condition: Box::new(lhs_ast),
                            true_clause: Box::new(tc_ast),
                            false_clause: Box::new(fc_ast),
                        },
                        start_location,
                        self.tokenizer.location(),
                    ),
                ))
            }
            _ => Ok((
                lhs,
                AstNode::new(
                    Expr::Unary(Box::new(lhs_ast)),
                    start_location,
                    self.tokenizer.location(),
                ),
            )),
        }
    }

    fn parse_conditional_or(&mut self) -> CelResult<(ParseResult, AstNode<ConditionalOr>)> {
        let start_loc = self.tokenizer.location();
        let (mut current_node, lhs_ast) = self.parse_conditional_and()?;

        let (lhs_start, lhs_end) = (lhs_ast.start(), lhs_ast.end());
        let mut current_ast = AstNode::new(ConditionalOr::Unary(lhs_ast), lhs_start, lhs_end);

        loop {
            if let Some(Token::OrOr) = self.tokenizer.peek()? {
                self.tokenizer.next()?;
                let (rhs, rhs_ast) = self.parse_conditional_and()?;
                let jmp = ParseResult::with_bytecode(vec![ByteCode::JmpCond {
                    when: JmpWhen::True,
                    dist: rhs.bytecode().len() as u32 + 1,
                    leave_val: true,
                }]);
                current_node = ParseResult::with_bytecode(vec![ByteCode::Or])
                    .consume_children(vec![current_node, jmp, rhs]);
                current_ast = AstNode::new(
                    ConditionalOr::Binary {
                        lhs: Box::new(current_ast),
                        rhs: rhs_ast,
                    },
                    start_loc,
                    self.tokenizer.location(),
                );
            } else {
                break;
            }
        }
        Ok((current_node, current_ast))
    }

    fn parse_conditional_and(&mut self) -> CelResult<(ParseResult, AstNode<ConditionalAnd>)> {
        let start_loc = self.tokenizer.location();
        let (mut current_node, lhs_ast) = self.parse_relation()?;

        let (lhs_start, lhs_end) = (lhs_ast.start(), lhs_ast.end());
        let mut current_ast = AstNode::new(ConditionalAnd::Unary(lhs_ast), lhs_start, lhs_end);

        loop {
            if let Some(Token::AndAnd) = self.tokenizer.peek()? {
                self.tokenizer.next()?;
                let (rhs, rhs_ast) = self.parse_relation()?;
                let jmp = ParseResult::with_bytecode(vec![ByteCode::JmpCond {
                    when: JmpWhen::False,
                    dist: rhs.bytecode().len() as u32 + 1,
                    leave_val: true,
                }]);

                current_node = ParseResult::with_bytecode(vec![ByteCode::And])
                    .consume_children(vec![current_node, jmp, rhs]);
                current_ast = AstNode::new(
                    ConditionalAnd::Binary {
                        lhs: Box::new(current_ast),
                        rhs: rhs_ast,
                    },
                    start_loc,
                    self.tokenizer.location(),
                );
            } else {
                break;
            }
        }
        Ok((current_node, current_ast))
    }

    fn parse_relation(&mut self) -> CelResult<(ParseResult, AstNode<Relation>)> {
        let start_loc = self.tokenizer.location();
        let (mut current_node, lhs_ast) = self.parse_addition()?;

        let (lhs_start, lhs_end) = (lhs_ast.start(), lhs_ast.end());
        let mut current_ast = AstNode::new(Relation::Unary(lhs_ast), lhs_start, lhs_end);

        loop {
            match self.tokenizer.peek()? {
                Some(Token::LessThan) => {
                    self.tokenizer.next()?;

                    let (rhs, rhs_ast) = self.parse_addition()?;

                    current_node = ParseResult::with_bytecode(vec![ByteCode::Lt])
                        .consume_children(vec![current_node, rhs]);
                    current_ast = AstNode::new(
                        Relation::Binary {
                            lhs: Box::new(current_ast),
                            op: Relop::Lt,
                            rhs: rhs_ast,
                        },
                        start_loc,
                        self.tokenizer.location(),
                    );
                }
                Some(Token::LessEqual) => {
                    self.tokenizer.next()?;
                    let (rhs, rhs_ast) = self.parse_addition()?;

                    current_node = ParseResult::with_bytecode(vec![ByteCode::Le])
                        .consume_children(vec![current_node, rhs]);
                    current_ast = AstNode::new(
                        Relation::Binary {
                            lhs: Box::new(current_ast),
                            op: Relop::Le,
                            rhs: rhs_ast,
                        },
                        start_loc,
                        self.tokenizer.location(),
                    );
                }
                Some(Token::EqualEqual) => {
                    self.tokenizer.next()?;
                    let (rhs, rhs_ast) = self.parse_addition()?;

                    current_node = ParseResult::with_bytecode(vec![ByteCode::Eq])
                        .consume_children(vec![current_node, rhs]);
                    current_ast = AstNode::new(
                        Relation::Binary {
                            lhs: Box::new(current_ast),
                            op: Relop::Eq,
                            rhs: rhs_ast,
                        },
                        start_loc,
                        self.tokenizer.location(),
                    );
                }
                Some(Token::NotEqual) => {
                    self.tokenizer.next()?;
                    let (rhs, rhs_ast) = self.parse_addition()?;

                    current_node = ParseResult::with_bytecode(vec![ByteCode::Ne])
                        .consume_children(vec![current_node, rhs]);
                    current_ast = AstNode::new(
                        Relation::Binary {
                            lhs: Box::new(current_ast),
                            op: Relop::Ne,
                            rhs: rhs_ast,
                        },
                        start_loc,
                        self.tokenizer.location(),
                    );
                }
                Some(Token::GreaterEqual) => {
                    self.tokenizer.next()?;
                    let (rhs, rhs_ast) = self.parse_addition()?;

                    current_node = ParseResult::with_bytecode(vec![ByteCode::Ge])
                        .consume_children(vec![current_node, rhs]);
                    current_ast = AstNode::new(
                        Relation::Binary {
                            lhs: Box::new(current_ast),
                            op: Relop::Ge,
                            rhs: rhs_ast,
                        },
                        start_loc,
                        self.tokenizer.location(),
                    );
                }
                Some(Token::GreaterThan) => {
                    self.tokenizer.next()?;
                    let (rhs, rhs_ast) = self.parse_addition()?;

                    current_node = ParseResult::with_bytecode(vec![ByteCode::Gt])
                        .consume_children(vec![current_node, rhs]);
                    current_ast = AstNode::new(
                        Relation::Binary {
                            lhs: Box::new(current_ast),
                            op: Relop::Gt,
                            rhs: rhs_ast,
                        },
                        start_loc,
                        self.tokenizer.location(),
                    );
                }
                Some(Token::In) => {
                    self.tokenizer.next()?;
                    let (rhs, rhs_ast) = self.parse_addition()?;

                    current_node = ParseResult::with_bytecode(vec![ByteCode::In])
                        .consume_children(vec![current_node, rhs]);
                    current_ast = AstNode::new(
                        Relation::Binary {
                            lhs: Box::new(current_ast),
                            op: Relop::In,
                            rhs: rhs_ast,
                        },
                        start_loc,
                        self.tokenizer.location(),
                    );
                }
                _ => break,
            }
        }

        Ok((current_node, current_ast))
    }

    fn parse_addition(&mut self) -> CelResult<(ParseResult, AstNode<Addition>)> {
        let start_loc = self.tokenizer.location();
        let (mut current_node, lhs_ast) = self.parse_multiplication()?;

        let (lhs_start, lhs_end) = (lhs_ast.start(), lhs_ast.end());
        let mut current_ast = AstNode::new(Addition::Unary(lhs_ast), lhs_start, lhs_end);

        loop {
            match self.tokenizer.peek()? {
                Some(Token::Add) => {
                    self.tokenizer.next()?;

                    let (rhs, rhs_ast) = self.parse_multiplication()?;

                    current_node = ParseResult::with_bytecode(vec![ByteCode::Add])
                        .consume_children(vec![current_node, rhs]);
                    current_ast = AstNode::new(
                        Addition::Binary {
                            lhs: Box::new(current_ast),
                            op: AddOp::Add,
                            rhs: rhs_ast,
                        },
                        start_loc,
                        self.tokenizer.location(),
                    );
                }
                Some(Token::Minus) => {
                    self.tokenizer.next()?;

                    let (rhs, rhs_ast) = self.parse_multiplication()?;

                    current_node = ParseResult::with_bytecode(vec![ByteCode::Sub])
                        .consume_children(vec![current_node, rhs]);
                    current_ast = AstNode::new(
                        Addition::Binary {
                            lhs: Box::new(current_ast),
                            op: AddOp::Sub,
                            rhs: rhs_ast,
                        },
                        start_loc,
                        self.tokenizer.location(),
                    );
                }
                _ => break,
            }
        }

        Ok((current_node, current_ast))
    }

    fn parse_multiplication(&mut self) -> CelResult<(ParseResult, AstNode<Multiplication>)> {
        let start_loc = self.tokenizer.location();
        let (mut current_node, lhs_ast) = self.parse_unary()?;

        let (lhs_start, lhs_end) = (lhs_ast.start(), lhs_ast.end());
        let mut current_ast = AstNode::new(Multiplication::Unary(lhs_ast), lhs_start, lhs_end);

        loop {
            match self.tokenizer.peek()? {
                Some(Token::Multiply) => {
                    self.tokenizer.next()?;

                    let (rhs, rhs_ast) = self.parse_unary()?;

                    current_node = ParseResult::with_bytecode(vec![ByteCode::Mul])
                        .consume_children(vec![current_node, rhs]);
                    current_ast = AstNode::new(
                        Multiplication::Binary {
                            lhs: Box::new(current_ast),
                            op: MultOp::Mult,
                            rhs: rhs_ast,
                        },
                        start_loc,
                        self.tokenizer.location(),
                    );
                }
                Some(Token::Divide) => {
                    self.tokenizer.next()?;

                    let (rhs, rhs_ast) = self.parse_unary()?;

                    current_node = ParseResult::with_bytecode(vec![ByteCode::Div])
                        .consume_children(vec![current_node, rhs]);
                    current_ast = AstNode::new(
                        Multiplication::Binary {
                            lhs: Box::new(current_ast),
                            op: MultOp::Div,
                            rhs: rhs_ast,
                        },
                        start_loc,
                        self.tokenizer.location(),
                    );
                }
                Some(Token::Mod) => {
                    self.tokenizer.next()?;

                    let (rhs, rhs_ast) = self.parse_unary()?;

                    current_node = ParseResult::with_bytecode(vec![ByteCode::Mod])
                        .consume_children(vec![current_node, rhs]);
                    current_ast = AstNode::new(
                        Multiplication::Binary {
                            lhs: Box::new(current_ast),
                            op: MultOp::Mod,
                            rhs: rhs_ast,
                        },
                        start_loc,
                        self.tokenizer.location(),
                    );
                }
                _ => break,
            }
        }

        Ok((current_node, current_ast))
    }

    fn parse_unary(&mut self) -> CelResult<(ParseResult, AstNode<Unary>)> {
        let start_loc = self.tokenizer.location();
        match self.tokenizer.peek()? {
            Some(Token::Not) => {
                let (not_res, not_ast) = self.parse_not_list()?;
                let (member_res, member_ast) = self.parse_member()?;

                Ok((
                    member_res.append_result(not_res),
                    AstNode::new(
                        Unary::NotMember {
                            nots: not_ast,
                            member: member_ast,
                        },
                        start_loc,
                        self.tokenizer.location(),
                    ),
                ))
            }
            Some(Token::Minus) => {
                let (neg_res, neg_ast) = self.parse_neg_list()?;
                let (member_res, member_ast) = self.parse_member()?;

                Ok((
                    member_res.append_result(neg_res),
                    AstNode::new(
                        Unary::NegMember {
                            negs: neg_ast,
                            member: member_ast,
                        },
                        start_loc,
                        self.tokenizer.location(),
                    ),
                ))
            }
            _ => {
                let (member, member_ast) = self.parse_member()?;

                Ok((
                    member,
                    AstNode::new(
                        Unary::Member(member_ast),
                        start_loc,
                        self.tokenizer.location(),
                    ),
                ))
            }
        }
    }

    fn parse_not_list(&mut self) -> CelResult<(ParseResult, AstNode<NotList>)> {
        let start_loc = self.tokenizer.location();
        let _res = ParseResult::new();

        match self.tokenizer.peek()? {
            Some(Token::Not) => {
                self.tokenizer.next()?;
                let (nxt, ast) = self.parse_not_list()?;
                Ok((
                    nxt.append_result(ParseResult::with_bytecode(vec![ByteCode::Not])),
                    AstNode::new(
                        NotList::List {
                            tail: Box::new(ast),
                        },
                        start_loc,
                        self.tokenizer.location(),
                    ),
                ))
            }
            _ => Ok((
                ParseResult::new(),
                AstNode::new(NotList::EmptyList, start_loc, self.tokenizer.location()),
            )),
        }
    }

    fn parse_neg_list(&mut self) -> CelResult<(ParseResult, AstNode<NegList>)> {
        let start_loc = self.tokenizer.location();
        let _res = ParseResult::new();

        match self.tokenizer.peek()? {
            Some(Token::Minus) => {
                self.tokenizer.next()?;
                let (nxt, ast) = self.parse_neg_list()?;
                Ok((
                    nxt.append_result(ParseResult::with_bytecode(vec![ByteCode::Neg])),
                    AstNode::new(
                        NegList::List {
                            tail: Box::new(ast),
                        },
                        start_loc,
                        self.tokenizer.location(),
                    ),
                ))
            }
            _ => Ok((
                ParseResult::new(),
                AstNode::new(NegList::EmptyList, start_loc, self.tokenizer.location()),
            )),
        }
    }

    fn parse_member(&mut self) -> CelResult<(ParseResult, AstNode<Member>)> {
        let start_loc = self.tokenizer.location();
        let (primary, primary_ast) = self.parse_primary()?;
        let (member_prime, member_prime_ast) = self.parse_member_prime()?;

        Ok((
            primary.append_result(member_prime),
            AstNode::new(
                Member {
                    primary: primary_ast,
                    member: member_prime_ast,
                },
                start_loc,
                self.tokenizer.location(),
            ),
        ))
    }

    fn parse_member_prime(&mut self) -> CelResult<(ParseResult, AstNode<MemberPrime>)> {
        let start_loc = self.tokenizer.location();

        match self.tokenizer.peek()? {
            Some(Token::Dot) => {
                self.tokenizer.next()?;
                match self.tokenizer.next()? {
                    Some(Token::Ident(ident)) => {
                        let res = ParseResult::with_bytecode(vec![
                            ByteCode::Push(CelValue::from_ident(&ident)),
                            ByteCode::Access,
                        ]);

                        let ident_end = self.tokenizer.location();
                        let (child_res, child_ast) = self.parse_member_prime()?;
                        Ok((
                            res.append_result(child_res),
                            AstNode::new(
                                MemberPrime::MemberAccess {
                                    ident: AstNode::new(Ident(ident), start_loc, ident_end),
                                    tail: Box::new(child_ast),
                                },
                                start_loc,
                                self.tokenizer.location(),
                            ),
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

                let (args, args_ast) = self.parse_expression_list(Token::RParen)?;

                let token = self.tokenizer.next()?;
                if token != Some(Token::RParen) {
                    Err(SyntaxError::from_location(self.tokenizer.location())
                        .with_message(format!(
                            "Unexpected token {}, expected RPARAN",
                            &token.map_or("NOTHING".to_string(), |x| format!("{:?}", x))
                        ))
                        .into())
                } else {
                    let (child, child_ast) = self.parse_member_prime()?;
                    Ok((
                        ParseResult::with_bytecode(vec![ByteCode::Call(args.len() as u32)])
                            .consume_call_children(args)
                            .append_result(child),
                        AstNode::new(
                            MemberPrime::Call {
                                call: args_ast,
                                tail: Box::new(child_ast),
                            },
                            start_loc,
                            self.tokenizer.location(),
                        ),
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
                            AstNode::new(
                                MemberPrime::ArrayAccess {
                                    access: index_ast,
                                    tail: Box::new(child_ast),
                                },
                                start_loc,
                                self.tokenizer.location(),
                            ),
                        ))
                    }
                    _ => Err(SyntaxError::from_location(self.tokenizer.location())
                        .with_message(format!(
                            "Unexpected token {}, expected RPARAN",
                            &next_token.map_or("NOTHING".to_string(), |x| format!("{:?}", x))
                        ))
                        .into()),
                }
            }
            _ => Ok((
                ParseResult::new(),
                AstNode::new(MemberPrime::Empty, start_loc, self.tokenizer.location()),
            )),
        }
    }

    fn parse_primary(&mut self) -> CelResult<(ParseResult, AstNode<Primary>)> {
        let start_loc = self.tokenizer.location();

        match self.tokenizer.peek()? {
            Some(Token::Ident(val)) => {
                self.tokenizer.next()?;
                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::Push(CelValue::from_ident(&val))])
                        .add_ident(&val),
                    AstNode::new(
                        Primary::Ident(Ident(val)),
                        start_loc,
                        self.tokenizer.location(),
                    ),
                ))
            }
            Some(Token::LParen) => {
                self.tokenizer.next()?;
                let (child, expr_ast) = self.parse_expression()?;
                if let Some(Token::RParen) = self.tokenizer.peek()? {
                    self.tokenizer.next()?;
                }

                Ok((
                    child,
                    AstNode::new(
                        Primary::Parens(expr_ast),
                        start_loc,
                        self.tokenizer.location(),
                    ),
                ))
            }
            Some(Token::LBracket) => {
                self.tokenizer.next()?;
                let (children, expr_list_ast) = self.parse_expression_list(Token::RBracket)?;

                if let Some(Token::RBracket) = self.tokenizer.peek()? {
                    self.tokenizer.next()?;
                }

                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::MkList(children.len() as u32)])
                        .consume_children(children),
                    AstNode::new(
                        Primary::ListConstruction(expr_list_ast),
                        start_loc,
                        self.tokenizer.location(),
                    ),
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
                    AstNode::new(
                        Primary::ObjectInit(obj_inits_ast),
                        start_loc,
                        self.tokenizer.location(),
                    ),
                ))
            }
            Some(Token::UIntLit(val)) => {
                self.tokenizer.next()?;
                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::Push(CelValue::from_uint(val))]),
                    AstNode::new(
                        Primary::Literal(LiteralsAndKeywords::UnsignedLit(val)),
                        start_loc,
                        self.tokenizer.location(),
                    ),
                ))
            }
            Some(Token::IntLit(val)) => {
                self.tokenizer.next()?;
                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::Push(CelValue::from_int(val))]),
                    AstNode::new(
                        Primary::Literal(LiteralsAndKeywords::IntegerLit(val)),
                        start_loc,
                        self.tokenizer.location(),
                    ),
                ))
            }
            Some(Token::FloatLit(val)) => {
                self.tokenizer.next()?;
                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::Push(CelValue::from_float(val))]),
                    AstNode::new(
                        Primary::Literal(LiteralsAndKeywords::FloatingLit(val)),
                        start_loc,
                        self.tokenizer.location(),
                    ),
                ))
            }
            Some(Token::StringLit(val)) => {
                self.tokenizer.next()?;
                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::Push(CelValue::from_string(
                        val.clone(),
                    ))]),
                    AstNode::new(
                        Primary::Literal(LiteralsAndKeywords::StringLit(val)),
                        start_loc,
                        self.tokenizer.location(),
                    ),
                ))
            }
            Some(Token::ByteStringLit(val)) => {
                self.tokenizer.next()?;
                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::Push(CelValue::from_bytes(
                        val.clone(),
                    ))]),
                    AstNode::new(
                        Primary::Literal(LiteralsAndKeywords::ByteStringLit(val)),
                        start_loc,
                        self.tokenizer.location(),
                    ),
                ))
            }
            Some(Token::BoolLit(val)) => {
                self.tokenizer.next()?;
                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::Push(CelValue::from_bool(val))]),
                    AstNode::new(
                        Primary::Literal(LiteralsAndKeywords::BooleanLit(val)),
                        start_loc,
                        self.tokenizer.location(),
                    ),
                ))
            }
            Some(Token::Null) => {
                self.tokenizer.next()?;
                Ok((
                    ParseResult::with_bytecode(vec![ByteCode::Push(CelValue::from_null())]),
                    AstNode::new(
                        Primary::Literal(LiteralsAndKeywords::NullLit),
                        start_loc,
                        self.tokenizer.location(),
                    ),
                ))
            }
            _ => Err(SyntaxError::from_location(self.tokenizer.location())
                .with_message(format!("unexpected!!! {:?}", self.tokenizer.peek()))
                .into()),
        }
    }

    fn parse_expression_list(
        &mut self,
        ending: Token,
    ) -> CelResult<(Vec<ParseResult>, AstNode<ExprList>)> {
        let start_loc = self.tokenizer.location();
        let mut vec = Vec::new();
        let mut ast = Vec::new();

        'outer: loop {
            match self.tokenizer.peek()? {
                Some(val) => {
                    if val == ending {
                        break 'outer;
                    }
                }
                None => {}
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

        Ok((
            vec,
            AstNode::new(
                ExprList { exprs: ast },
                start_loc,
                self.tokenizer.location(),
            ),
        ))
    }

    fn parse_obj_inits(&mut self) -> CelResult<(Vec<ParseResult>, AstNode<ObjInits>)> {
        let start_loc = self.tokenizer.location();
        let mut vec = Vec::new();
        let mut ast = Vec::new();

        'outer: loop {
            let loop_start = self.tokenizer.location();
            if self.tokenizer.peek()? == Some(Token::RBrace) {
                break 'outer;
            }

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

            ast.push(AstNode::new(
                ObjInit {
                    key: key_ast,
                    value: value_ast,
                },
                loop_start,
                self.tokenizer.location(),
            ));

            match self.tokenizer.peek()? {
                Some(Token::Comma) => {
                    self.tokenizer.next()?;
                    continue;
                }
                _ => break 'outer,
            }
        }

        Ok((
            vec,
            AstNode::new(
                ObjInits { inits: ast },
                start_loc,
                self.tokenizer.location(),
            ),
        ))
    }
}

#[cfg(test)]
mod test {
    use test_case::test_case;

    use crate::compiler::string_tokenizer::StringTokenizer;

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
        let mut tokenizer = StringTokenizer::with_input(input);
        CelCompiler::with_tokenizer(&mut tokenizer)
            .compile()
            .unwrap();
    }
}
