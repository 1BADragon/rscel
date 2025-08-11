use crate::{error::ToSqlError, traits::ToSql};
use rscel::{
    Addition, ConditionalAnd, ConditionalOr, Expr, ExprList, Ident, LiteralsAndKeywords, MatchCase,
    MatchPattern, Member, MemberPrime, Multiplication, NegList, NotList, ObjInit, Primary,
    Relation, Unary,
};

impl ToSql for Expr {
    fn to_sql(&self) -> Result<String, ToSqlError> {
        match self {
            Expr::Ternary {
                condition,
                true_clause,
                false_clause,
            } => Ok(format!(
                "case ({})::bool when true then ({}) else ({})",
                condition.to_sql()?,
                true_clause.to_sql()?,
                false_clause.to_sql()?
            )),
            Expr::Match {
                condition: _,
                cases: _,
            } => Err(ToSqlError::unsupported(
                "Match expressions not supported currently",
            )),
            Expr::Unary(ast_node) => ast_node.to_sql(),
        }
    }
}

impl ToSql for MatchCase {
    fn to_sql(&self) -> crate::error::ToSqlResult<String> {
        todo!()
    }
}

impl ToSql for MatchPattern {
    fn to_sql(&self) -> crate::error::ToSqlResult<String> {
        todo!()
    }
}

impl ToSql for ConditionalOr {
    fn to_sql(&self) -> Result<String, ToSqlError> {
        match self {
            ConditionalOr::Binary { lhs, rhs } => {
                Ok(format!("({}) OR ({})", lhs.to_sql()?, rhs.to_sql()?))
            }
            ConditionalOr::Unary(ast_node) => ast_node.to_sql(),
        }
    }
}

impl ToSql for ConditionalAnd {
    fn to_sql(&self) -> Result<String, ToSqlError> {
        match self {
            ConditionalAnd::Binary { lhs, rhs } => {
                Ok(format!("({}) AND ({})", lhs.to_sql()?, rhs.to_sql()?))
            }
            ConditionalAnd::Unary(ast_node) => ast_node.to_sql(),
        }
    }
}

impl ToSql for Relation {
    fn to_sql(&self) -> Result<String, ToSqlError> {
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

                Ok(format!(
                    "({}) {} ({})",
                    lhs.to_sql()?,
                    op_str,
                    rhs.to_sql()?
                ))
            }
            Relation::Unary(ast_node) => ast_node.to_sql(),
        }
    }
}

impl ToSql for Addition {
    fn to_sql(&self) -> Result<String, ToSqlError> {
        match self {
            Addition::Binary { lhs, op, rhs } => {
                let op_str = match op {
                    rscel::AddOp::Add => "+",
                    rscel::AddOp::Sub => "-",
                };

                Ok(format!(
                    "({}) {} ({})",
                    lhs.to_sql()?,
                    op_str,
                    rhs.to_sql()?
                ))
            }
            Addition::Unary(ast_node) => ast_node.to_sql(),
        }
    }
}

impl ToSql for Multiplication {
    fn to_sql(&self) -> Result<String, ToSqlError> {
        match self {
            Multiplication::Binary { lhs, op, rhs } => {
                let op_str = match op {
                    rscel::MultOp::Mult => "*",
                    rscel::MultOp::Div => "/",
                    rscel::MultOp::Mod => "%",
                };

                Ok(format!(
                    "({}) {} ({})",
                    lhs.to_sql()?,
                    op_str,
                    rhs.to_sql()?
                ))
            }
            Multiplication::Unary(ast_node) => ast_node.to_sql(),
        }
    }
}

impl ToSql for Unary {
    fn to_sql(&self) -> Result<String, ToSqlError> {
        match self {
            Unary::Member(ast_node) => ast_node.to_sql(),
            Unary::NotMember { nots, member } => {
                Ok(format!("({}{})", nots.to_sql()?, member.to_sql()?))
            }
            Unary::NegMember { negs, member } => {
                Ok(format!("({}{})", negs.to_sql()?, member.to_sql()?))
            }
        }
    }
}

impl ToSql for NotList {
    fn to_sql(&self) -> Result<String, ToSqlError> {
        match self {
            NotList::List { tail } => Ok(format!("!{}", tail.to_sql()?)),
            NotList::EmptyList => Ok(String::new()),
        }
    }
}

impl ToSql for NegList {
    fn to_sql(&self) -> Result<String, ToSqlError> {
        match self {
            NegList::List { tail } => Ok(format!("-{}", tail.to_sql()?)),
            NegList::EmptyList => Ok(String::new()),
        }
    }
}

impl ToSql for Member {
    fn to_sql(&self) -> Result<String, ToSqlError> {
        let mut primary_sql = self.primary.to_sql()?;

        self.member
            .iter()
            .map(|m| m.to_sql())
            .collect::<Result<Vec<String>, ToSqlError>>()?
            .into_iter()
            .for_each(|piece| primary_sql += &piece);

        Ok(primary_sql)
    }
}

impl ToSql for MemberPrime {
    fn to_sql(&self) -> crate::error::ToSqlResult<String> {
        todo!()
    }
}

impl ToSql for Ident {
    fn to_sql(&self) -> Result<String, ToSqlError> {
        Ok(self.0.clone())
    }
}

impl ToSql for Primary {
    fn to_sql(&self) -> Result<String, ToSqlError> {
        match self {
            Primary::Type => todo!(),
            Primary::Ident(ident) => ident.to_sql(),
            Primary::Parens(ast_node) => Ok(format!("({})", ast_node.to_sql()?)),
            Primary::ListConstruction(_ast_node) => todo!(),
            Primary::ObjectInit(_ast_node) => todo!(),
            Primary::Literal(literals_and_keywords) => literals_and_keywords.to_sql(),
        }
    }
}

impl ToSql for LiteralsAndKeywords {
    fn to_sql(&self) -> crate::error::ToSqlResult<String> {
        match self {
            LiteralsAndKeywords::Type => todo!(),
            LiteralsAndKeywords::NullType => todo!(),
            LiteralsAndKeywords::Int => todo!(),
            LiteralsAndKeywords::Uint => todo!(),
            LiteralsAndKeywords::Float => todo!(),
            LiteralsAndKeywords::Bool => todo!(),
            LiteralsAndKeywords::String => todo!(),
            LiteralsAndKeywords::Bytes => todo!(),
            LiteralsAndKeywords::Timestamp => todo!(),
            LiteralsAndKeywords::Duration => todo!(),
            LiteralsAndKeywords::NullLit => Ok("NULL".to_owned()),
            LiteralsAndKeywords::IntegerLit(val) => Ok(format!("{}", val)),
            LiteralsAndKeywords::UnsignedLit(val) => Ok(format!("{}", val)),
            LiteralsAndKeywords::FloatingLit(val) => Ok(format!("{}", val)),
            LiteralsAndKeywords::FStringList(_) => todo!(),
            LiteralsAndKeywords::StringLit(val) => Ok(val.clone()),
            LiteralsAndKeywords::ByteStringLit(_) => todo!(),
            LiteralsAndKeywords::BooleanLit(val) => match val {
                true => Ok("TRUE".to_owned()),
                false => Ok("FALSE".to_owned()),
            },
        }
    }
}

impl ToSql for ExprList {
    fn to_sql(&self) -> Result<String, ToSqlError> {
        todo!()
    }
}

impl ToSql for ObjInit {
    fn to_sql(&self) -> crate::error::ToSqlResult<String> {
        todo!()
    }
}
