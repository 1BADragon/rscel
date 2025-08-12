use crate::{error::ToSqlError, traits::*};
use rscel::{
    Addition, ConditionalAnd, ConditionalOr, Expr, ExprList, Ident, LiteralsAndKeywords, MatchCase,
    MatchPattern, Member, MemberPrime, Multiplication, NegList, NotList, ObjInit, ObjInits,
    Primary, Relation, Unary,
};

impl ToSql for Expr {
    fn to_sql(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        match self {
            Expr::Ternary {
                condition,
                true_clause,
                false_clause,
            } => {
                let condition_builder = condition.to_sql()?;
                let true_builder = true_clause.to_sql()?;
                let false_builder = false_clause.to_sql()?;

                Ok(Box::new(CaseExpressionBuilder {
                    condition: condition_builder.to_sql()?,
                    true_clause: true_builder.to_sql()?,
                    false_clause: false_builder.to_sql()?,
                }))
            }
            Expr::Match {
                condition: _,
                cases: _,
            } => Ok(Box::new(UnsupportedBuilder {
                message: "Match expressions not supported currently".to_string(),
            })),
            Expr::Unary(ast_node) => ast_node.to_sql(),
        }
    }
}

impl ToSql for MatchCase {
    fn to_sql(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        Ok(Box::new(UnsupportedBuilder {
            message: "MatchCase not implemented yet".to_string(),
        }))
    }
}

impl ToSql for MatchPattern {
    fn to_sql(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        Ok(Box::new(UnsupportedBuilder {
            message: "MatchPattern not implemented yet".to_string(),
        }))
    }
}

impl ToSql for ConditionalOr {
    fn to_sql(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        match self {
            ConditionalOr::Binary { lhs, rhs } => {
                let lhs_builder = lhs.to_sql()?;
                let rhs_builder = rhs.to_sql()?;

                Ok(Box::new(BinaryOperationBuilder {
                    lhs: lhs_builder.to_sql()?,
                    operator: "OR".to_string(),
                    rhs: rhs_builder.to_sql()?,
                }))
            }
            ConditionalOr::Unary(ast_node) => ast_node.to_sql(),
        }
    }
}

impl ToSql for ConditionalAnd {
    fn to_sql(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        match self {
            ConditionalAnd::Binary { lhs, rhs } => {
                let lhs_builder = lhs.to_sql()?;
                let rhs_builder = rhs.to_sql()?;

                Ok(Box::new(BinaryOperationBuilder {
                    lhs: lhs_builder.to_sql()?,
                    operator: "AND".to_string(),
                    rhs: rhs_builder.to_sql()?,
                }))
            }
            ConditionalAnd::Unary(ast_node) => ast_node.to_sql(),
        }
    }
}

impl ToSql for Relation {
    fn to_sql(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
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

                let lhs_builder = lhs.to_sql()?;
                let rhs_builder = rhs.to_sql()?;

                Ok(Box::new(BinaryOperationBuilder {
                    lhs: lhs_builder.to_sql()?,
                    operator: op_str.to_string(),
                    rhs: rhs_builder.to_sql()?,
                }))
            }
            Relation::Unary(ast_node) => ast_node.to_sql(),
        }
    }
}

impl ToSql for Addition {
    fn to_sql(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        match self {
            Addition::Binary { lhs, op, rhs } => {
                let op_str = match op {
                    rscel::AddOp::Add => "+",
                    rscel::AddOp::Sub => "-",
                };

                let lhs_builder = lhs.to_sql()?;
                let rhs_builder = rhs.to_sql()?;

                Ok(Box::new(BinaryOperationBuilder {
                    lhs: lhs_builder.to_sql()?,
                    operator: op_str.to_string(),
                    rhs: rhs_builder.to_sql()?,
                }))
            }
            Addition::Unary(ast_node) => ast_node.to_sql(),
        }
    }
}

impl ToSql for Multiplication {
    fn to_sql(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        match self {
            Multiplication::Binary { lhs, op, rhs } => {
                let op_str = match op {
                    rscel::MultOp::Mult => "*",
                    rscel::MultOp::Div => "/",
                    rscel::MultOp::Mod => "%",
                };

                let lhs_builder = lhs.to_sql()?;
                let rhs_builder = rhs.to_sql()?;

                Ok(Box::new(BinaryOperationBuilder {
                    lhs: lhs_builder.to_sql()?,
                    operator: op_str.to_string(),
                    rhs: rhs_builder.to_sql()?,
                }))
            }
            Multiplication::Unary(ast_node) => ast_node.to_sql(),
        }
    }
}

impl ToSql for Unary {
    fn to_sql(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        match self {
            Unary::Member(ast_node) => ast_node.to_sql(),
            Unary::NotMember { nots, member } => {
                let nots_builder = nots.to_sql()?;
                let member_builder = member.to_sql()?;

                Ok(Box::new(UnaryOperationBuilder {
                    operator: nots_builder.to_sql()?,
                    operand: member_builder.to_sql()?,
                }))
            }
            Unary::NegMember { negs, member } => {
                let negs_builder = negs.to_sql()?;
                let member_builder = member.to_sql()?;

                Ok(Box::new(UnaryOperationBuilder {
                    operator: negs_builder.to_sql()?,
                    operand: member_builder.to_sql()?,
                }))
            }
        }
    }
}

impl ToSql for NotList {
    fn to_sql(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        match self {
            NotList::List { tail } => {
                let tail_builder = tail.to_sql()?;
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

impl ToSql for NegList {
    fn to_sql(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        match self {
            NegList::List { tail } => {
                let tail_builder = tail.to_sql()?;
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

impl ToSql for Member {
    fn to_sql(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        let primary_builder = self.primary.to_sql()?;
        let base_sql = primary_builder.to_sql()?;

        // Check if this is a single function call
        if self.member.len() == 1 {
            if let MemberPrime::Call { call } = self.member[0].node() {
                // Get the function arguments (reversed to fix parser ordering)
                let mut args = call
                    .node()
                    .exprs
                    .iter()
                    .map(|expr| {
                        let builder = expr.to_sql()?;
                        builder.to_sql()
                    })
                    .collect::<Result<Vec<String>, ToSqlError>>()?;
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
                                value: args[0].clone(),
                                cast_type: cast_type.to_string(),
                            }));
                        } else if args.is_empty() {
                            // Handle cases like int() - cast null to type
                            return Ok(Box::new(CastBuilder {
                                value: "NULL".to_string(),
                                cast_type: cast_type.to_string(),
                            }));
                        }
                    }
                }

                // Not a type cast, this is a regular function call
                return Ok(Box::new(LiteralBuilder {
                    value: if args.is_empty() {
                        format!("{}()", base_sql)
                    } else {
                        format!("{}({})", base_sql, args.join(", "))
                    },
                }));
            }
        }

        // Handle as member access chain - check if this is JSON member access
        if self
            .member
            .iter()
            .all(|m| matches!(m.node(), MemberPrime::MemberAccess { .. }))
        {
            // This is a chain of member accesses, likely JSON field access
            let mut result = base_sql;
            for member_prime in &self.member {
                if let MemberPrime::MemberAccess { ident } = member_prime.node() {
                    let field_name = ident.to_sql()?.to_sql()?;
                    result = format!("({})->'{}'", result, field_name);
                }
            }
            return Ok(Box::new(LiteralBuilder { value: result }));
        }

        // Handle other member access patterns (mixed access types)
        let member_sqls: Result<Vec<String>, ToSqlError> = self
            .member
            .iter()
            .map(|m| {
                let builder = m.to_sql()?;
                builder.to_sql()
            })
            .collect();

        Ok(Box::new(MemberAccessBuilder {
            base: base_sql,
            members: member_sqls?,
        }))
    }
}

impl ToSql for MemberPrime {
    fn to_sql(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        match self {
            MemberPrime::MemberAccess { ident } => {
                // For JSON member access, we'll return a JSON field access pattern
                // This will be handled specially in the Member implementation
                let field_name = ident.to_sql()?.to_sql()?;
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
                        let builder = expr.to_sql()?;
                        builder.to_sql()
                    })
                    .collect::<Result<Vec<String>, ToSqlError>>()?;

                Ok(Box::new(LiteralBuilder {
                    value: format!("({})", args.join(", ")),
                }))
            }
            MemberPrime::ArrayAccess { access } => {
                let access_builder = access.to_sql()?;
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

impl ToSql for Ident {
    fn to_sql(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        Ok(Box::new(IdentifierBuilder {
            name: self.0.clone(),
        }))
    }
}

impl ToSql for Primary {
    fn to_sql(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        match self {
            Primary::Type => Ok(Box::new(UnsupportedBuilder {
                message: "Type primary not implemented yet".to_string(),
            })),
            Primary::Ident(ident) => ident.to_sql(),
            Primary::Parens(ast_node) => {
                let inner_builder = ast_node.to_sql()?;
                Ok(Box::new(ParensBuilder {
                    inner: inner_builder.to_sql()?,
                }))
            }
            Primary::ListConstruction(ast_node) => {
                let expr_list = ast_node.node();
                let elements: Result<Vec<String>, ToSqlError> = expr_list
                    .exprs
                    .iter()
                    .map(|expr| {
                        let builder = expr.to_sql()?;
                        builder.to_sql()
                    })
                    .collect();

                Ok(Box::new(ArrayBuilder {
                    elements: elements?,
                }))
            }
            Primary::ObjectInit(ast_node) => {
                let obj_inits = ast_node.node();
                let mut fields: Vec<(String, String)> = obj_inits
                    .inits
                    .iter()
                    .map(|init| {
                        // Swap key and value to fix parser order issue
                        let value_builder = init.node().key.to_sql()?;
                        let value_sql = value_builder.to_sql()?;

                        let key_builder = init.node().value.to_sql()?;
                        let key_sql = key_builder.to_sql()?;
                        // Remove quotes from string keys for JSON field names
                        let clean_key = if key_sql.starts_with('\'') && key_sql.ends_with('\'') {
                            key_sql[1..key_sql.len() - 1].to_string()
                        } else {
                            key_sql
                        };

                        Ok((clean_key, value_sql))
                    })
                    .collect::<Result<Vec<(String, String)>, ToSqlError>>()?;

                // Reverse the order to fix parser ordering issue
                fields.reverse();

                Ok(Box::new(JsonObjectBuilder { fields }))
            }
            Primary::Literal(literals_and_keywords) => literals_and_keywords.to_sql(),
        }
    }
}

impl ToSql for LiteralsAndKeywords {
    fn to_sql(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
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
            LiteralsAndKeywords::StringLit(val) => val.clone(),
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

impl ToSql for ExprList {
    fn to_sql(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        let args: Result<Vec<String>, ToSqlError> = self
            .exprs
            .iter()
            .map(|expr| {
                let builder = expr.to_sql()?;
                builder.to_sql()
            })
            .collect();

        Ok(Box::new(LiteralBuilder {
            value: args?.join(", "),
        }))
    }
}

impl ToSql for ObjInit {
    fn to_sql(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        // This is handled at the ObjInits level, but we need this for completeness
        let key_builder = self.key.to_sql()?;
        let key_sql = key_builder.to_sql()?;
        let value_builder = self.value.to_sql()?;
        let value_sql = value_builder.to_sql()?;

        Ok(Box::new(LiteralBuilder {
            value: format!("'{}': {}", key_sql, value_sql),
        }))
    }
}

impl ToSql for ObjInits {
    fn to_sql(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        let fields: Result<Vec<(String, String)>, ToSqlError> = self
            .inits
            .iter()
            .map(|init| {
                let key_builder = init.node().key.to_sql()?;
                let key_sql = key_builder.to_sql()?;
                let value_builder = init.node().value.to_sql()?;
                let value_sql = value_builder.to_sql()?;
                Ok((key_sql, value_sql))
            })
            .collect();

        Ok(Box::new(JsonObjectBuilder { fields: fields? }))
    }
}
