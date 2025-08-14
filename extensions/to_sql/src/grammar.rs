use crate::{
    error::{ToSqlError, ToSqlResult},
    traits::*,
};
use rscel::{
    Addition, ConditionalAnd, ConditionalOr, Expr, ExprList, Ident, LiteralsAndKeywords, MatchCase,
    MatchPattern, Member, MemberPrime, Multiplication, NegList, NotList, ObjInit, ObjInits,
    Primary, Relation, Unary,
};

impl IntoSqlBuilder for Expr {
    fn into_sql_builder(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        match self {
            Expr::Ternary {
                condition,
                true_clause,
                false_clause,
            } => {
                let condition_builder = condition.into_sql_builder()?;
                let true_builder = true_clause.into_sql_builder()?;
                let false_builder = false_clause.into_sql_builder()?;

                Ok(Box::new(TurnaryExpressionBuilder {
                    condition: condition_builder,
                    true_clause: true_builder,
                    false_clause: false_builder,
                }))
            }
            Expr::Match {
                condition: _,
                cases: _,
            } => Ok(Box::new(UnsupportedBuilder {
                message: "Match expressions not supported currently".to_string(),
            })),
            Expr::Unary(ast_node) => ast_node.into_sql_builder(),
        }
    }
}

impl IntoSqlBuilder for MatchCase {
    fn into_sql_builder(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        Ok(Box::new(UnsupportedBuilder {
            message: "MatchCase not implemented yet".to_string(),
        }))
    }
}

impl IntoSqlBuilder for MatchPattern {
    fn into_sql_builder(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        Ok(Box::new(UnsupportedBuilder {
            message: "MatchPattern not implemented yet".to_string(),
        }))
    }
}

impl IntoSqlBuilder for ConditionalOr {
    fn into_sql_builder(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        match self {
            ConditionalOr::Binary { lhs, rhs } => {
                let lhs_builder = lhs.into_sql_builder()?;
                let rhs_builder = rhs.into_sql_builder()?;

                Ok(Box::new(BinaryOperationBuilder {
                    lhs: lhs_builder,
                    operator: StaticSqlBuilder::boxed("OR"),
                    rhs: rhs_builder,
                }))
            }
            ConditionalOr::Unary(ast_node) => ast_node.into_sql_builder(),
        }
    }
}

impl IntoSqlBuilder for ConditionalAnd {
    fn into_sql_builder(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        match self {
            ConditionalAnd::Binary { lhs, rhs } => {
                let lhs_builder = lhs.into_sql_builder()?;
                let rhs_builder = rhs.into_sql_builder()?;

                Ok(Box::new(BinaryOperationBuilder {
                    lhs: lhs_builder,
                    operator: StaticSqlBuilder::boxed("AND"),
                    rhs: rhs_builder,
                }))
            }
            ConditionalAnd::Unary(ast_node) => ast_node.into_sql_builder(),
        }
    }
}

impl IntoSqlBuilder for Relation {
    fn into_sql_builder(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        match self {
            Relation::Binary { lhs, op, rhs } => {
                let op_str = match op {
                    rscel::Relop::Le => "<=",
                    rscel::Relop::Lt => "<",
                    rscel::Relop::Ge => ">=",
                    rscel::Relop::Gt => ">",
                    rscel::Relop::Eq => "=",
                    rscel::Relop::Ne => "<>",
                    rscel::Relop::In => "in",
                };

                let lhs_builder = lhs.into_sql_builder()?;
                let rhs_builder = rhs.into_sql_builder()?;

                Ok(Box::new(BinaryOperationBuilder {
                    lhs: lhs_builder,
                    operator: StaticSqlBuilder::boxed(op_str),
                    rhs: rhs_builder,
                }))
            }
            Relation::Unary(ast_node) => ast_node.into_sql_builder(),
        }
    }
}

impl IntoSqlBuilder for Addition {
    fn into_sql_builder(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        match self {
            Addition::Binary { lhs, op, rhs } => {
                let op_str = match op {
                    rscel::AddOp::Add => "+",
                    rscel::AddOp::Sub => "-",
                };

                let lhs_builder = lhs.into_sql_builder()?;
                let rhs_builder = rhs.into_sql_builder()?;

                Ok(Box::new(BinaryOperationBuilder {
                    lhs: lhs_builder,
                    operator: StaticSqlBuilder::boxed(op_str),
                    rhs: rhs_builder,
                }))
            }
            Addition::Unary(ast_node) => ast_node.into_sql_builder(),
        }
    }
}

impl IntoSqlBuilder for Multiplication {
    fn into_sql_builder(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        match self {
            Multiplication::Binary { lhs, op, rhs } => {
                let op_str = match op {
                    rscel::MultOp::Mult => "*",
                    rscel::MultOp::Div => "/",
                    rscel::MultOp::Mod => "%",
                };

                let lhs_builder = lhs.into_sql_builder()?;
                let rhs_builder = rhs.into_sql_builder()?;

                Ok(Box::new(BinaryOperationBuilder {
                    lhs: lhs_builder,
                    operator: StaticSqlBuilder::boxed(op_str),
                    rhs: rhs_builder,
                }))
            }
            Multiplication::Unary(ast_node) => ast_node.into_sql_builder(),
        }
    }
}

impl IntoSqlBuilder for Unary {
    fn into_sql_builder(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        match self {
            Unary::Member(ast_node) => ast_node.into_sql_builder(),
            Unary::NotMember { nots, member } => {
                let nots_builder = nots.into_sql_builder()?;
                let member_builder = member.into_sql_builder()?;

                Ok(Box::new(UnaryOperationBuilder {
                    operator: nots_builder,
                    operand: member_builder,
                }))
            }
            Unary::NegMember { negs, member } => {
                let negs_builder = negs.into_sql_builder()?;
                let member_builder = member.into_sql_builder()?;

                Ok(Box::new(UnaryOperationBuilder {
                    operator: negs_builder,
                    operand: member_builder,
                }))
            }
        }
    }
}

impl IntoSqlBuilder for NotList {
    fn into_sql_builder(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        match self {
            NotList::List { tail } => {
                let tail_builder = tail.into_sql_builder()?;
                Ok(Box::new(LiteralBuilder {
                    value: format!("!{}", tail_builder.to_sql()?),
                }))
            }
            NotList::EmptyList => Ok(Box::new(LiteralBuilder {
                value: String::new(),
            })),
        }
    }
}

impl IntoSqlBuilder for NegList {
    fn into_sql_builder(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        match self {
            NegList::List { tail } => {
                let tail_builder = tail.into_sql_builder()?;
                Ok(Box::new(LiteralBuilder {
                    value: format!("-{}", tail_builder.to_sql()?),
                }))
            }
            NegList::EmptyList => Ok(Box::new(LiteralBuilder {
                value: String::new(),
            })),
        }
    }
}

impl IntoSqlBuilder for Member {
    fn into_sql_builder(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        let primary_builder = self.primary.into_sql_builder()?;

        // Check if this is a single function call
        if self.member.len() == 1 {
            if let MemberPrime::Call { call } = self.member[0].node() {
                // Get the function arguments (reversed to fix parser ordering)
                let mut args = call
                    .node()
                    .exprs
                    .iter()
                    .map(|expr| expr.into_sql_builder())
                    .collect::<Result<Vec<_>, ToSqlError>>()?;
                args.reverse();

                // Check if this is a type casting operation
                if let Primary::Ident(ident) = self.primary.node() {
                    let ident_name = &ident.0;
                    let sql_type = match ident_name.as_str() {
                        "int" => Some("integer"),
                        "uint" => Some("bigint"),
                        "float" => Some("double precision"),
                        "double" => Some("double precision"),
                        "string" => Some("text"),
                        "bool" => Some("boolean"),
                        "bytes" => Some("bytea"),
                        "timestamp" => Some("timestamp"),
                        "duration" => Some("interval"),
                        _ => None,
                    };

                    if let Some(cast_type) = sql_type {
                        // This is a type casting operation
                        if args.len() == 1 {
                            return Ok(Box::new(CastBuilder {
                                value: args.remove(0),
                                cast_type: StaticSqlBuilder::boxed(cast_type),
                            }));
                        } else if args.is_empty() {
                            // Handle cases like int() - cast null to type
                            return Ok(Box::new(CastBuilder {
                                value: StaticSqlBuilder::boxed("NULL"),
                                cast_type: StaticSqlBuilder::boxed(cast_type),
                            }));
                        }
                    }
                }

                // Not a type cast, this is a regular function call
                return Ok(Box::new(FunctionCallBuilder {
                    primary: primary_builder,
                    args: args,
                }));
            }
        }

        let mut builder = primary_builder;

        for (i, member) in self.member.iter().enumerate() {
            match member.node() {
                MemberPrime::MemberAccess { ident } => {
                    builder = Box::new(JsonMemberAccessBuilder {
                        object: builder,
                        field: ident.node().into_sql_builder()?,
                        extract_text: i == (self.member.len() - 1),
                    })
                }
                MemberPrime::Call { call } => {
                    builder = Box::new(FunctionCallBuilder {
                        primary: builder,
                        args: call
                            .node()
                            .exprs
                            .iter()
                            .map(|a| a.node().into_sql_builder())
                            .collect::<ToSqlResult<Vec<_>>>()?,
                    });
                }
                MemberPrime::ArrayAccess { access } => {
                    builder = Box::new(ArrayAccessBuilder {
                        array: builder,
                        member: access.node().into_sql_builder()?,
                    })
                }
                MemberPrime::Empty => break,
            }
        }

        return Ok(builder);

        // Handle as member access chain - check if this is JSON member access
    }
}

impl IntoSqlBuilder for MemberPrime {
    fn into_sql_builder(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        match self {
            MemberPrime::MemberAccess { ident } => {
                // For JSON member access, we'll return a JSON field access pattern
                // This will be handled specially in the Member implementation
                let field_name = ident.into_sql_builder()?.to_sql()?;
                Ok(Box::new(LiteralBuilder {
                    value: format!("->'{}'", field_name),
                }))
            }
            MemberPrime::Call { call } => {
                // For now, return a function call builder
                // Type casting will be handled at a higher level in Member
                let args = call
                    .node()
                    .exprs
                    .iter()
                    .map(|expr| {
                        let builder = expr.into_sql_builder()?;
                        builder.to_sql()
                    })
                    .collect::<Result<Vec<String>, ToSqlError>>()?;

                Ok(Box::new(LiteralBuilder {
                    value: format!("({})", args.join(", ")),
                }))
            }
            MemberPrime::ArrayAccess { access } => {
                let access_builder = access.into_sql_builder()?;
                Ok(Box::new(LiteralBuilder {
                    value: format!("[{}]", access_builder.to_sql()?),
                }))
            }
            MemberPrime::Empty => Ok(Box::new(LiteralBuilder {
                value: String::new(),
            })),
        }
    }
}

impl IntoSqlBuilder for Ident {
    fn into_sql_builder(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        Ok(Box::new(IdentifierBuilder {
            name: self.0.clone(),
        }))
    }
}

impl IntoSqlBuilder for Primary {
    fn into_sql_builder(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        match self {
            Primary::Type => Ok(Box::new(UnsupportedBuilder {
                message: "Type primary not implemented yet".to_string(),
            })),
            Primary::Ident(ident) => ident.into_sql_builder(),
            Primary::Parens(ast_node) => {
                let inner_builder = ast_node.into_sql_builder()?;
                Ok(Box::new(ParensBuilder {
                    inner: inner_builder,
                }))
            }
            Primary::ListConstruction(ast_node) => {
                let expr_list = ast_node.node();
                let elements = expr_list
                    .exprs
                    .iter()
                    .map(|expr| expr.into_sql_builder())
                    .collect::<ToSqlResult<Vec<_>>>()?;

                Ok(Box::new(ArrayBuilder { elements: elements }))
            }
            Primary::ObjectInit(ast_node) => ast_node.node().into_sql_builder(),
            Primary::Literal(literals_and_keywords) => literals_and_keywords.into_sql_builder(),
        }
    }
}

impl IntoSqlBuilder for LiteralsAndKeywords {
    fn into_sql_builder(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        let value = match self {
            LiteralsAndKeywords::Type => {
                return Ok(Box::new(UnsupportedBuilder {
                    message: "Type literal not implemented yet".to_string(),
                }))
            }
            LiteralsAndKeywords::NullType => {
                return Ok(Box::new(UnsupportedBuilder {
                    message: "NullType literal not implemented yet".to_string(),
                }))
            }
            LiteralsAndKeywords::Int => {
                return Ok(Box::new(UnsupportedBuilder {
                    message: "Int literal not implemented yet".to_string(),
                }))
            }
            LiteralsAndKeywords::Uint => {
                return Ok(Box::new(UnsupportedBuilder {
                    message: "Uint literal not implemented yet".to_string(),
                }))
            }
            LiteralsAndKeywords::Float => {
                return Ok(Box::new(UnsupportedBuilder {
                    message: "Float literal not implemented yet".to_string(),
                }))
            }
            LiteralsAndKeywords::Bool => {
                return Ok(Box::new(UnsupportedBuilder {
                    message: "Bool literal not implemented yet".to_string(),
                }))
            }
            LiteralsAndKeywords::String => {
                return Ok(Box::new(UnsupportedBuilder {
                    message: "String literal not implemented yet".to_string(),
                }))
            }
            LiteralsAndKeywords::Bytes => {
                return Ok(Box::new(UnsupportedBuilder {
                    message: "Bytes literal not implemented yet".to_string(),
                }))
            }
            LiteralsAndKeywords::Timestamp => {
                return Ok(Box::new(UnsupportedBuilder {
                    message: "Timestamp literal not implemented yet".to_string(),
                }))
            }
            LiteralsAndKeywords::Duration => {
                return Ok(Box::new(UnsupportedBuilder {
                    message: "Duration literal not implemented yet".to_string(),
                }))
            }
            LiteralsAndKeywords::NullLit => "NULL".to_owned(),
            LiteralsAndKeywords::IntegerLit(val) => format!("{}", val),
            LiteralsAndKeywords::UnsignedLit(val) => format!("{}", val),
            LiteralsAndKeywords::FloatingLit(val) => format!("{}", val),
            LiteralsAndKeywords::FStringList(_) => {
                return Ok(Box::new(UnsupportedBuilder {
                    message: "FStringList not implemented yet".to_string(),
                }))
            }
            LiteralsAndKeywords::StringLit(val) => format!("'{}'", val),
            LiteralsAndKeywords::ByteStringLit(_) => {
                return Ok(Box::new(UnsupportedBuilder {
                    message: "ByteStringLit not implemented yet".to_string(),
                }))
            }
            LiteralsAndKeywords::BooleanLit(val) => match val {
                true => "TRUE".to_owned(),
                false => "FALSE".to_owned(),
            },
        };

        Ok(Box::new(LiteralBuilder { value }))
    }
}

impl IntoSqlBuilder for ExprList {
    fn into_sql_builder(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        let args: Result<Vec<String>, ToSqlError> = self
            .exprs
            .iter()
            .map(|expr| {
                let builder = expr.into_sql_builder()?;
                builder.to_sql()
            })
            .collect();

        Ok(Box::new(LiteralBuilder {
            value: args?.join(", "),
        }))
    }
}

impl IntoSqlBuilder for ObjInit {
    fn into_sql_builder(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        // This is handled at the ObjInits level, but we need this for completeness
        let key_builder = self.key.into_sql_builder()?;
        let key_sql = key_builder.to_sql()?;
        let value_builder = self.value.into_sql_builder()?;
        let value_sql = value_builder.to_sql()?;

        Ok(Box::new(LiteralBuilder {
            value: format!("'{}': {}", key_sql, value_sql),
        }))
    }
}

impl IntoSqlBuilder for ObjInits {
    fn into_sql_builder(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        let fields = self
            .inits
            .iter()
            .map(|init| {
                let key_builder = init.node().key.into_sql_builder()?;
                let value_builder = init.node().value.into_sql_builder()?;
                Ok((key_builder, value_builder))
            })
            .collect::<ToSqlResult<Vec<_>>>()?;

        Ok(Box::new(JsonObjectBuilder { fields }))
    }
}
