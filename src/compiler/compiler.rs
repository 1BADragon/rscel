use super::{
    ast_node::AstNode,
    compiled_node::{CompiledNode, CompiledNodeTrait},
    grammar::*,
    syntax_error::SyntaxError,
    tokenizer::Tokenizer,
    tokens::Token,
};
use crate::{interp::JmpWhen, ByteCode, CelResult, CelValue, CelValueDyn, Program};

pub struct CelCompiler<'l> {
    tokenizer: &'l mut dyn Tokenizer,
}

impl<'l> CelCompiler<'l> {
    pub fn with_tokenizer(tokenizer: &'l mut dyn Tokenizer) -> CelCompiler<'l> {
        CelCompiler { tokenizer }
    }

    pub fn compile(mut self) -> CelResult<Program> {
        let compiled = self.parse_expression()?;

        if !self.tokenizer.peek()?.is_none() {
            return Err(SyntaxError::from_location(self.tokenizer.location())
                .with_message(format!("Unexpected token: {:?}", self.tokenizer.peek()?))
                .into());
        }

        let prog =
            CompiledNode::<Expr>::into_program(compiled, self.tokenizer.source().to_string());

        Ok(prog)
    }

    fn parse_expression(&mut self) -> CelResult<CompiledNode<Expr>> {
        let start_location = self.tokenizer.location();
        let mut lhs_node = self.parse_conditional_or()?;

        let lhs_ast = lhs_node.yank_ast();

        match self.tokenizer.peek()? {
            Some(Token::Question) => {
                self.tokenizer.next()?;
                let mut true_clause_node = self.parse_conditional_or()?;
                let true_clause_ast = true_clause_node.yank_ast();

                let next = self.tokenizer.next()?;
                if next != Some(Token::Colon) {
                    return Err(SyntaxError::from_location(self.tokenizer.location())
                        .with_message(format!("Unexpected token {:?}, expected COLON", next))
                        .into());
                }

                let mut false_clause_node = self.parse_expression()?;
                let false_clause_ast = false_clause_node.yank_ast();

                Ok(lhs_node
                    .into_turnary(true_clause_node, false_clause_node)
                    .add_ast(AstNode::new(
                        Expr::Ternary {
                            condition: Box::new(lhs_ast),
                            true_clause: Box::new(true_clause_ast),
                            false_clause: Box::new(false_clause_ast),
                        },
                        start_location,
                        self.tokenizer.location(),
                    )))
            }
            _ => Ok(CompiledNode::from_node(lhs_node).add_ast(AstNode::new(
                Expr::Unary(Box::new(lhs_ast)),
                start_location,
                self.tokenizer.location(),
            ))),
        }
    }

    fn parse_conditional_or(&mut self) -> CelResult<CompiledNode<ConditionalOr>> {
        let start_loc = self.tokenizer.location();
        let mut current_node = self.parse_conditional_and()?.convert_with_ast(|lhs_ast| {
            AstNode::new(
                ConditionalOr::Unary(lhs_ast),
                start_loc,
                self.tokenizer.location(),
            )
        });
        let mut current_ast = current_node.yank_ast();

        loop {
            if let Some(Token::OrOr) = self.tokenizer.peek()? {
                self.tokenizer.next()?;
                let mut rhs_node = self.parse_conditional_and()?;

                let jmp_node = CompiledNode::<NoAst>::with_bytecode(vec![ByteCode::JmpCond {
                    when: JmpWhen::True,
                    dist: rhs_node.bytecode_len() as u32 + 1,
                    leave_val: true,
                }]);

                current_ast = AstNode::new(
                    ConditionalOr::Binary {
                        lhs: Box::new(current_ast),
                        rhs: rhs_node.yank_ast(),
                    },
                    start_loc,
                    self.tokenizer.location(),
                );
                current_node = CompiledNode::with_bytecode(vec![ByteCode::Or]).consume_children3(
                    current_node,
                    jmp_node,
                    rhs_node,
                    |lhs, rhs| lhs.or(&rhs),
                );
            } else {
                break;
            }
        }

        Ok(current_node.add_ast(current_ast))
    }

    fn parse_conditional_and(&mut self) -> CelResult<CompiledNode<ConditionalAnd>> {
        let start_loc = self.tokenizer.location();
        let mut current_node = self.parse_relation()?.convert_with_ast(|ast| {
            AstNode::new(
                ConditionalAnd::Unary(ast),
                start_loc,
                self.tokenizer.location(),
            )
        });

        let mut current_ast = current_node.yank_ast();

        loop {
            if let Some(Token::AndAnd) = self.tokenizer.peek()? {
                self.tokenizer.next()?;
                let mut rhs_node = self.parse_relation()?;
                let jmp_node = CompiledNode::<NoAst>::with_bytecode(vec![ByteCode::JmpCond {
                    when: JmpWhen::False,
                    dist: rhs_node.bytecode_len() as u32 + 1,
                    leave_val: true,
                }]);

                current_ast = AstNode::new(
                    ConditionalAnd::Binary {
                        lhs: Box::new(current_ast),
                        rhs: rhs_node.yank_ast(),
                    },
                    start_loc,
                    self.tokenizer.location(),
                );
                current_node = CompiledNode::with_bytecode(vec![ByteCode::And]).consume_children3(
                    current_node,
                    jmp_node,
                    rhs_node,
                    |lhs, rhs| lhs.and(&rhs),
                );
            } else {
                break;
            }
        }
        Ok(current_node.add_ast(current_ast))
    }

    fn parse_relation(&mut self) -> CelResult<CompiledNode<Relation>> {
        let start_loc = self.tokenizer.location();
        let mut current_node = self.parse_addition()?.convert_with_ast(|ast| {
            AstNode::new(Relation::Unary(ast), start_loc, self.tokenizer.location())
        });
        let mut current_ast = current_node.yank_ast();

        loop {
            match self.tokenizer.peek()? {
                Some(Token::LessThan) => {
                    self.tokenizer.next()?;

                    let mut rhs_node = self.parse_addition()?;

                    current_ast = AstNode::new(
                        Relation::Binary {
                            lhs: Box::new(current_ast),
                            op: Relop::Lt,
                            rhs: rhs_node.yank_ast(),
                        },
                        start_loc,
                        self.tokenizer.location(),
                    );

                    current_node = CompiledNode::with_bytecode(vec![ByteCode::Lt])
                        .consume_children2(current_node, rhs_node, |lhs, rhs| lhs.lt(&rhs));
                }
                Some(Token::LessEqual) => {
                    self.tokenizer.next()?;
                    let rhs_node = self.parse_addition()?;

                    current_ast = AstNode::new(
                        Relation::Binary {
                            lhs: Box::new(current_ast),
                            op: Relop::Le,
                            rhs: rhs_node.yank_ast(),
                        },
                        start_loc,
                        self.tokenizer.location(),
                    );

                    current_node = CompiledNode::with_bytecode(vec![ByteCode::Le])
                        .consume_children2(current_node, rhs_node, |lhs, rhs| lhs.le(&rhs));
                }
                Some(Token::EqualEqual) => {
                    self.tokenizer.next()?;
                    let mut rhs_node = self.parse_addition()?;

                    current_ast = AstNode::new(
                        Relation::Binary {
                            lhs: Box::new(current_ast),
                            op: Relop::Eq,
                            rhs: rhs_node.yank_ast(),
                        },
                        start_loc,
                        self.tokenizer.location(),
                    );

                    current_node = CompiledNode::with_bytecode(vec![ByteCode::Eq])
                        .consume_children2(current_node, rhs_node, |lhs, rhs| {
                            CelValueDyn::eq(&lhs, &rhs)
                        });
                }
                Some(Token::NotEqual) => {
                    self.tokenizer.next()?;
                    let mut rhs_node = self.parse_addition()?;

                    current_ast = AstNode::new(
                        Relation::Binary {
                            lhs: Box::new(current_ast),
                            op: Relop::Ne,
                            rhs: rhs_node.yank_ast(),
                        },
                        start_loc,
                        self.tokenizer.location(),
                    );

                    current_node = CompiledNode::with_bytecode(vec![ByteCode::Ne])
                        .consume_children2(current_node, rhs_node, |lhs, rhs| lhs.neq(&rhs));
                }
                Some(Token::GreaterEqual) => {
                    self.tokenizer.next()?;
                    let mut rhs_node = self.parse_addition()?;

                    current_ast = AstNode::new(
                        Relation::Binary {
                            lhs: Box::new(current_ast),
                            op: Relop::Ge,
                            rhs: rhs_node.yank_ast(),
                        },
                        start_loc,
                        self.tokenizer.location(),
                    );

                    current_node = CompiledNode::with_bytecode(vec![ByteCode::Ge])
                        .consume_children2(current_node, rhs_node, |lhs, rhs| lhs.ge(&rhs));
                }
                Some(Token::GreaterThan) => {
                    self.tokenizer.next()?;
                    let mut rhs_node = self.parse_addition()?;

                    current_ast = AstNode::new(
                        Relation::Binary {
                            lhs: Box::new(current_ast),
                            op: Relop::Gt,
                            rhs: rhs_node.yank_ast(),
                        },
                        start_loc,
                        self.tokenizer.location(),
                    );

                    current_node = CompiledNode::with_bytecode(vec![ByteCode::Gt])
                        .consume_children2(current_node, rhs_node, |lhs, rhs| lhs.ge(&rhs));
                }
                Some(Token::In) => {
                    self.tokenizer.next()?;
                    let mut rhs_node = self.parse_addition()?;

                    current_ast = AstNode::new(
                        Relation::Binary {
                            lhs: Box::new(current_ast),
                            op: Relop::In,
                            rhs: rhs_node.yank_ast(),
                        },
                        start_loc,
                        self.tokenizer.location(),
                    );
                    current_node = CompiledNode::with_bytecode(vec![ByteCode::In])
                        .consume_children2(current_node, rhs_node, |lhs, rhs| lhs.in_(&rhs));
                }
                _ => break,
            }
        }

        Ok(current_node.add_ast(current_ast))
    }

    fn parse_addition(&mut self) -> CelResult<CompiledNode<Addition>> {
        let start_loc = self.tokenizer.location();
        let mut current_node = self.parse_multiplication()?.convert_with_ast(|ast| {
            AstNode::new(Addition::Unary(ast), start_loc, self.tokenizer.location())
        });
        let mut current_ast = current_node.yank_ast();

        loop {
            match self.tokenizer.peek()? {
                Some(Token::Add) => {
                    self.tokenizer.next()?;

                    let mut rhs_node = self.parse_multiplication()?;

                    current_ast = AstNode::new(
                        Addition::Binary {
                            lhs: Box::new(current_ast),
                            op: AddOp::Add,
                            rhs: rhs_node.yank_ast(),
                        },
                        start_loc,
                        self.tokenizer.location(),
                    );

                    current_node = CompiledNode::with_bytecode(vec![ByteCode::Add])
                        .consume_children2(current_node, rhs_node, |lhs, rhs| lhs + rhs);
                }
                Some(Token::Minus) => {
                    self.tokenizer.next()?;

                    let mut rhs_node = self.parse_multiplication()?;

                    current_ast = AstNode::new(
                        Addition::Binary {
                            lhs: Box::new(current_ast),
                            op: AddOp::Sub,
                            rhs: rhs_node.yank_ast(),
                        },
                        start_loc,
                        self.tokenizer.location(),
                    );

                    current_node = CompiledNode::with_bytecode(vec![ByteCode::Sub])
                        .consume_children2(current_node, rhs_node, |lhs, rhs| lhs - rhs);
                }
                _ => break,
            }
        }

        Ok(current_node.add_ast(current_ast))
    }

    fn parse_multiplication(&mut self) -> CelResult<CompiledNode<Multiplication>> {
        let start_loc = self.tokenizer.location();
        let mut current_node = self.parse_unary()?.convert_with_ast(|ast| {
            AstNode::new(
                Multiplication::Unary(ast),
                start_loc,
                self.tokenizer.location(),
            )
        });
        let mut current_ast = current_node.yank_ast();

        loop {
            match self.tokenizer.peek()? {
                Some(Token::Multiply) => {
                    self.tokenizer.next()?;

                    let mut rhs_node = self.parse_unary()?;
                    current_ast = AstNode::new(
                        Multiplication::Binary {
                            lhs: Box::new(current_ast),
                            op: MultOp::Mult,
                            rhs: rhs_node.yank_ast(),
                        },
                        start_loc,
                        self.tokenizer.location(),
                    );

                    current_node = CompiledNode::with_bytecode(vec![ByteCode::Mul])
                        .consume_children2(current_node, rhs_node, |lhs, rhs| lhs * rhs);
                }
                Some(Token::Divide) => {
                    self.tokenizer.next()?;

                    let mut rhs_node = self.parse_unary()?;

                    current_ast = AstNode::new(
                        Multiplication::Binary {
                            lhs: Box::new(current_ast),
                            op: MultOp::Div,
                            rhs: rhs_node.yank_ast(),
                        },
                        start_loc,
                        self.tokenizer.location(),
                    );

                    current_node = CompiledNode::with_bytecode(vec![ByteCode::Div])
                        .consume_children2(current_node, rhs_node, |lhs, rhs| lhs / rhs);
                }
                Some(Token::Mod) => {
                    self.tokenizer.next()?;

                    let mut rhs_node = self.parse_unary()?;

                    current_ast = AstNode::new(
                        Multiplication::Binary {
                            lhs: Box::new(current_ast),
                            op: MultOp::Mod,
                            rhs: rhs_node.yank_ast(),
                        },
                        start_loc,
                        self.tokenizer.location(),
                    );

                    current_node = CompiledNode::with_bytecode(vec![ByteCode::Mod])
                        .consume_children2(current_node, rhs_node, |lhs, rhs| lhs % rhs);
                }
                _ => break,
            }
        }

        Ok(current_node.add_ast(current_ast))
    }

    fn parse_unary(&mut self) -> CelResult<CompiledNode<Unary>> {
        let start_loc = self.tokenizer.location();
        match self.tokenizer.peek()? {
            Some(Token::Not) => {
                let mut not = self.parse_not_list()?;
                let not_ast = not.yank_ast();
                let mut member = self.parse_member()?;
                let member_ast = member.yank_ast();

                Ok(member.append_result(not).add_ast(AstNode::new(
                    Unary::NotMember {
                        nots: not_ast,
                        member: member_ast,
                    },
                    start_loc,
                    self.tokenizer.location(),
                )))
            }
            Some(Token::Minus) => {
                let mut neg = self.parse_neg_list()?;
                let neg_ast = neg.yank_ast();
                let mut member = self.parse_member()?;
                let member_ast = member.yank_ast();

                Ok(member.append_result(neg).add_ast(AstNode::new(
                    Unary::NegMember {
                        negs: neg_ast,
                        member: member_ast,
                    },
                    start_loc,
                    self.tokenizer.location(),
                )))
            }
            _ => {
                let member = self.parse_member()?;

                Ok(member.convert_with_ast(|ast| {
                    AstNode::new(Unary::Member(ast), start_loc, self.tokenizer.location())
                }))
            }
        }
    }

    fn parse_not_list(&mut self) -> CelResult<CompiledNode<NotList>> {
        let start_loc = self.tokenizer.location();

        match self.tokenizer.peek()? {
            Some(Token::Not) => {
                self.tokenizer.next()?;
                let nxt_node = self.parse_not_list()?;
                Ok(nxt_node
                    .append_result(CompiledNode::<NoAst>::with_bytecode(vec![ByteCode::Not]))
                    .convert_with_ast(|ast| {
                        AstNode::new(
                            NotList::List {
                                tail: Box::new(ast),
                            },
                            start_loc,
                            self.tokenizer.location(),
                        )
                    }))
            }
            _ => Ok(CompiledNode::empty().add_ast(AstNode::new(
                NotList::EmptyList,
                start_loc,
                self.tokenizer.location(),
            ))),
        }
    }

    fn parse_neg_list(&mut self) -> CelResult<CompiledNode<NegList>> {
        let start_loc = self.tokenizer.location();

        match self.tokenizer.peek()? {
            Some(Token::Minus) => {
                self.tokenizer.next()?;
                let nxt_node = self.parse_neg_list()?;
                Ok(nxt_node
                    .append_result(CompiledNode::<NoAst>::with_bytecode(vec![ByteCode::Neg]))
                    .convert_with_ast(|ast| {
                        AstNode::new(
                            NegList::List {
                                tail: Box::new(ast),
                            },
                            start_loc,
                            self.tokenizer.location(),
                        )
                    }))
            }
            _ => Ok(CompiledNode::empty().add_ast(AstNode::new(
                NegList::EmptyList,
                start_loc,
                self.tokenizer.location(),
            ))),
        }
    }

    fn parse_member(&mut self) -> CelResult<CompiledNode<Member>> {
        let start_loc = self.tokenizer.location();
        let primary = self.parse_primary()?;
        let mut member_prime = self.parse_member_prime()?;

        let member_prime_ast = member_prime.yank_ast();

        Ok(primary.append_result(member_prime).convert_with_ast(|ast| {
            AstNode::new(
                Member {
                    primary: ast,
                    member: member_prime_ast.clone(),
                },
                start_loc,
                self.tokenizer.location(),
            )
        }))
    }

    fn parse_member_prime(&mut self) -> CelResult<CompiledNode<MemberPrime>> {
        let start_loc = self.tokenizer.location();

        match self.tokenizer.peek()? {
            Some(Token::Dot) => {
                self.tokenizer.next()?;
                match self.tokenizer.next()? {
                    Some(Token::Ident(ident)) => {
                        let res = CompiledNode::with_bytecode(vec![
                            ByteCode::Push(CelValue::from_ident(&ident)),
                            ByteCode::Access,
                        ]);

                        let ident_end = self.tokenizer.location();
                        let mut child_res = self.parse_member_prime()?;
                        let child_res_ast = child_res.yank_ast();

                        Ok(res.append_result(child_res).convert_with_ast(|_ast| {
                            AstNode::new(
                                MemberPrime::MemberAccess {
                                    ident: AstNode::new(Ident(ident), start_loc, ident_end),
                                    tail: Box::new(child_res_ast),
                                },
                                start_loc,
                                self.tokenizer.location(),
                            )
                        }))
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

                let node = self.parse_expression_list(Token::RParen)?;

                let token = self.tokenizer.next()?;
                if token != Some(Token::RParen) {
                    Err(SyntaxError::from_location(self.tokenizer.location())
                        .with_message(format!(
                            "Unexpected token {}, expected RPARAN",
                            &token.map_or("NOTHING".to_string(), |x| format!("{:?}", x))
                        ))
                        .into())
                } else {
                    Ok(
                        CompiledNode::with_bytecode(vec![ByteCode::Call(args.len() as u32)])
                            .consume_call_children(node)
                            .append_result(child_node)
                            .add_ast(AstNode::new(
                                MemberPrime::Call {
                                    call: args_ast,
                                    tail: Box::new(child_ast),
                                },
                                start_loc,
                                self.tokenizer.location(),
                            )),
                    )
                }
            }
            Some(Token::LBracket) => {
                self.tokenizer.next()?;

                let index_node = self.parse_expression()?;

                let next_token = self.tokenizer.next()?;
                match next_token {
                    Some(Token::RBracket) => {
                        let (child, child_ast) = self.parse_member_prime()?;

                        Ok(index_node
                            .append_result(CompiledNode::with_bytecode(vec![ByteCode::Index]))
                            .append_result(child)
                            .add_ast(AstNode::new(
                                MemberPrime::ArrayAccess {
                                    access: index_ast,
                                    tail: Box::new(child_ast),
                                },
                                start_loc,
                                self.tokenizer.location(),
                            )))
                    }
                    _ => Err(SyntaxError::from_location(self.tokenizer.location())
                        .with_message(format!(
                            "Unexpected token {}, expected RPARAN",
                            &next_token.map_or("NOTHING".to_string(), |x| format!("{:?}", x))
                        ))
                        .into()),
                }
            }

            _ => Ok(CompiledNode::empty().add_ast(AstNode::new(
                MemberPrime::Empty,
                start_loc,
                self.tokenizer.location(),
            ))),
        }
    }

    fn parse_primary(&mut self) -> CelResult<CompiledNode<Primary>> {
        let start_loc = self.tokenizer.location();

        match self.tokenizer.peek()? {
            Some(Token::Ident(val)) => {
                self.tokenizer.next()?;
                Ok(
                    CompiledNode::with_bytecode(vec![ByteCode::Push(CelValue::from_ident(&val))])
                        .add_ident(&val)
                        .add_ast(AstNode::new(
                            Primary::Ident(Ident(val)),
                            start_loc,
                            self.tokenizer.location(),
                        )),
                )
            }
            Some(Token::LParen) => {
                self.tokenizer.next()?;
                let expr = self.parse_expression()?;

                if let Some(Token::RParen) = self.tokenizer.peek()? {
                    self.tokenizer.next()?;
                }

                let new_ast = AstNode::new(
                    Primary::Parens(expr.ast()),
                    start_loc,
                    self.tokenizer.location(),
                );

                Ok(expr.add_ast(new_ast))
            }
            Some(Token::LBracket) => {
                self.tokenizer.next()?;
                let expr_list = self.parse_expression_list(Token::RBracket)?;

                if let Some(Token::RBracket) = self.tokenizer.peek()? {
                    self.tokenizer.next()?;
                }

                let new_ast = AstNode::new(
                    Primary::ListConstruction(expr_list.ast()),
                    start_loc,
                    self.tokenizer.location(),
                );

                Ok(
                    CompiledNode::with_bytecode(vec![ByteCode::MkList(children.len() as u32)])
                        .consume_children(children, None)?
                        .add_ast(new_ast),
                )
            }
            Some(Token::LBrace) => {
                self.tokenizer.next()?;
                let obj_init = self.parse_obj_inits()?;

                if let Some(Token::RBrace) = self.tokenizer.peek()? {
                    self.tokenizer.next()?;
                }

                let new_ast = AstNode::new(
                    Primary::ObjectInit(obj_inits.ast()),
                    start_loc,
                    self.tokenizer.location(),
                );

                Ok(
                    CompiledNode::with_bytecode(vec![ByteCode::MkDict(children.len() as u32 / 2)])
                        .consume_children(children, None)
                        .add_ast(new_ast),
                )
            }
            Some(Token::UIntLit(val)) => {
                self.tokenizer.next()?;

                Ok(CompiledNode::with_const(val.into()).add_ast(AstNode::new(
                    Primary::Literal(LiteralsAndKeywords::UnsignedLit(val)),
                    start_loc,
                    self.tokenizer.location(),
                )))
            }
            Some(Token::IntLit(val)) => {
                self.tokenizer.next()?;
                Ok(CompiledNode::with_const(val.into()).add_ast(AstNode::new(
                    Primary::Literal(LiteralsAndKeywords::IntegerLit(val)),
                    start_loc,
                    self.tokenizer.location(),
                )))
            }
            Some(Token::FloatLit(val)) => {
                self.tokenizer.next()?;
                Ok(CompiledNode::with_const(val.into()).add_ast(AstNode::new(
                    Primary::Literal(LiteralsAndKeywords::FloatingLit(val)),
                    start_loc,
                    self.tokenizer.location(),
                )))
            }
            Some(Token::StringLit(val)) => {
                self.tokenizer.next()?;
                Ok(CompiledNode::with_const(val.into()).add_ast(AstNode::new(
                    Primary::Literal(LiteralsAndKeywords::StringLit(val)),
                    start_loc,
                    self.tokenizer.location(),
                )))
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

    fn parse_expression_list(&mut self, ending: Token) -> CelResult<CompiledNode<ExprList>> {
        let start_loc = self.tokenizer.location();
        let mut bytecode = Vec::new();
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

            let compiled = self.parse_expression()?;
            ast.push(compiled.ast().expect("Internal error, no ast"));
            bytecode.push(compiled.value().into_bytecode());

            match self.tokenizer.peek()? {
                Some(Token::Comma) => {
                    self.tokenizer.next()?;
                    continue;
                }
                _ => break 'outer,
            }
        }

        Ok((
            bytecode,
            AstNode::new(
                ExprList { exprs: ast },
                start_loc,
                self.tokenizer.location(),
            ),
        ))
    }

    fn parse_obj_inits(&mut self) -> CelResult<CompiledNode<ObjInits>> {
        let start_loc = self.tokenizer.location();
        let mut vec = Vec::new();
        let mut ast = Vec::new();

        'outer: loop {
            let loop_start = self.tokenizer.location();
            if self.tokenizer.peek()? == Some(Token::RBrace) {
                break 'outer;
            }

            let compiled_key = self.parse_expression()?;

            let next_token = self.tokenizer.next()?;
            if next_token != Some(Token::Colon) {
                return Err(SyntaxError::from_location(self.tokenizer.location())
                    .with_message(format!("Invalid token: expected ':' got {:?}", next_token))
                    .into());
            }
            // MkDict expects value then key
            let compiled_value = self.parse_expression()?;

            // TODO: refactor this to build map if everything is const
            vec.push(compiled_value.value().into_bytecode());
            vec.push(compiled_key.value().into_bytecode());

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

        Ok(CompiledNode::with_bytecode(vec).add_ast(ast))
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
