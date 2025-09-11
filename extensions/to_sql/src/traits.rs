use crate::error::{ToSqlError, ToSqlResult};

pub trait SqlBuilder {
    fn to_sql(self: Box<Self>) -> ToSqlResult<String>;
}

pub trait IntoSqlBuilder {
    fn into_sql_builder(&self) -> ToSqlResult<Box<dyn SqlBuilder>>;
}

// Concrete SqlBuilder implementations

pub struct TurnaryExpressionBuilder {
    pub condition: Box<dyn SqlBuilder>,
    pub true_clause: Box<dyn SqlBuilder>,
    pub false_clause: Box<dyn SqlBuilder>,
}

impl SqlBuilder for TurnaryExpressionBuilder {
    fn to_sql(self: Box<Self>) -> ToSqlResult<String> {
        Ok(format!(
            "case ({})::bool when true then ({}) else ({}) end",
            self.condition.to_sql()?,
            self.true_clause.to_sql()?,
            self.false_clause.to_sql()?
        ))
    }
}

pub struct StaticSqlBuilder(&'static str);

impl StaticSqlBuilder {
    pub fn boxed(content: &'static str) -> Box<Self> {
        Box::new(StaticSqlBuilder(content))
    }
}

impl SqlBuilder for StaticSqlBuilder {
    fn to_sql(self: Box<Self>) -> ToSqlResult<String> {
        Ok(self.0.to_owned())
    }
}

pub struct BinaryOperationBuilder {
    pub lhs: Box<dyn SqlBuilder>,
    pub operator: Box<dyn SqlBuilder>,
    pub rhs: Box<dyn SqlBuilder>,
}

impl SqlBuilder for BinaryOperationBuilder {
    fn to_sql(self: Box<Self>) -> ToSqlResult<String> {
        Ok(format!(
            "({}) {} ({})",
            self.lhs.to_sql()?,
            self.operator.to_sql()?,
            self.rhs.to_sql()?
        ))
    }
}

pub struct UnaryOperationBuilder {
    pub operator: Box<dyn SqlBuilder>,
    pub operand: Box<dyn SqlBuilder>,
}

impl SqlBuilder for UnaryOperationBuilder {
    fn to_sql(self: Box<Self>) -> ToSqlResult<String> {
        Ok(format!(
            "({}{})",
            self.operator.to_sql()?,
            self.operand.to_sql()?
        ))
    }
}

pub struct IdentifierBuilder {
    pub name: String,
}

impl SqlBuilder for IdentifierBuilder {
    fn to_sql(self: Box<Self>) -> ToSqlResult<String> {
        Ok(self.name)
    }
}

pub struct LiteralBuilder {
    pub value: String,
}

impl SqlBuilder for LiteralBuilder {
    fn to_sql(self: Box<Self>) -> ToSqlResult<String> {
        Ok(self.value)
    }
}

pub struct ParensBuilder {
    pub inner: Box<dyn SqlBuilder>,
}

impl SqlBuilder for ParensBuilder {
    fn to_sql(self: Box<Self>) -> ToSqlResult<String> {
        Ok(format!("({})", self.inner.to_sql()?))
    }
}

pub struct UnsupportedBuilder {
    pub message: String,
}

impl SqlBuilder for UnsupportedBuilder {
    fn to_sql(self: Box<Self>) -> ToSqlResult<String> {
        Err(ToSqlError::Unsupported(self.message))
    }
}

pub struct FunctionCallBuilder {
    pub primary: Box<dyn SqlBuilder>,
    pub args: Vec<Box<dyn SqlBuilder>>,
}

impl SqlBuilder for FunctionCallBuilder {
    fn to_sql(self: Box<Self>) -> ToSqlResult<String> {
        Ok(format!(
            "{}({})",
            self.primary.to_sql()?,
            self.args
                .into_iter()
                .map(|a| a.to_sql())
                .collect::<ToSqlResult<Vec<_>>>()?
                .join(", ")
        ))
    }
}

pub struct CastBuilder {
    pub value: Box<dyn SqlBuilder>,
    pub cast_type: Box<dyn SqlBuilder>,
}

impl SqlBuilder for CastBuilder {
    fn to_sql(self: Box<Self>) -> ToSqlResult<String> {
        Ok(format!(
            "{}::{}",
            self.value.to_sql()?,
            self.cast_type.to_sql()?
        ))
    }
}

pub struct JsonObjectBuilder {
    pub fields: Vec<(Box<dyn SqlBuilder>, Box<dyn SqlBuilder>)>,
}

impl SqlBuilder for JsonObjectBuilder {
    fn to_sql(self: Box<Self>) -> ToSqlResult<String> {
        if self.fields.is_empty() {
            Ok("'{}'::json".to_string())
        } else {
            let field_pairs: Vec<String> = self
                .fields
                .into_iter()
                .map(|(key, value)| Ok(format!("{}, {}", key.to_sql()?, value.to_sql()?)))
                .collect::<ToSqlResult<Vec<_>>>()?;
            Ok(format!("json_build_object({})", field_pairs.join(", ")))
        }
    }
}

pub struct JsonMemberAccessBuilder {
    pub object: Box<dyn SqlBuilder>,
    pub field: Box<dyn SqlBuilder>,
    pub extract_text: bool,
}

impl SqlBuilder for JsonMemberAccessBuilder {
    fn to_sql(self: Box<Self>) -> ToSqlResult<String> {
        if self.extract_text {
            Ok(format!(
                "({})->>'{}'",
                self.object.to_sql()?,
                self.field.to_sql()?
            ))
        } else {
            Ok(format!(
                "({})->'{}'",
                self.object.to_sql()?,
                self.field.to_sql()?
            ))
        }
    }
}

pub struct ArrayBuilder {
    pub elements: Vec<Box<dyn SqlBuilder>>,
}

impl SqlBuilder for ArrayBuilder {
    fn to_sql(self: Box<Self>) -> ToSqlResult<String> {
        if self.elements.is_empty() {
            Ok("ARRAY[]".to_string())
        } else {
            Ok(format!(
                "ARRAY[{}]",
                self.elements
                    .into_iter()
                    .map(|x| x.to_sql())
                    .collect::<ToSqlResult<Vec<String>>>()?
                    .join(", ")
            ))
        }
    }
}

pub struct ArrayAccessBuilder {
    pub array: Box<dyn SqlBuilder>,
    pub member: Box<dyn SqlBuilder>,
}

impl SqlBuilder for ArrayAccessBuilder {
    fn to_sql(self: Box<Self>) -> ToSqlResult<String> {
        Ok(format!(
            "({}[{}])",
            self.array.to_sql()?,
            self.member.to_sql()?
        ))
    }
}
