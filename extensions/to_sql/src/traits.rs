use crate::error::{ToSqlError, ToSqlResult};

pub trait SqlBuilder {
    fn to_sql(&self) -> ToSqlResult<String>;
}

pub trait ToSql {
    fn to_sql(&self) -> ToSqlResult<Box<dyn SqlBuilder>>;
}

// Concrete SqlBuilder implementations

pub struct CaseExpressionBuilder {
    pub condition: String,
    pub true_clause: String,
    pub false_clause: String,
}

impl SqlBuilder for CaseExpressionBuilder {
    fn to_sql(&self) -> ToSqlResult<String> {
        Ok(format!(
            "case ({})::bool when true then ({}) else ({})",
            self.condition, self.true_clause, self.false_clause
        ))
    }
}

pub struct BinaryOperationBuilder {
    pub lhs: String,
    pub operator: String,
    pub rhs: String,
}

impl SqlBuilder for BinaryOperationBuilder {
    fn to_sql(&self) -> ToSqlResult<String> {
        Ok(format!("({}) {} ({})", self.lhs, self.operator, self.rhs))
    }
}

pub struct UnaryOperationBuilder {
    pub operator: String,
    pub operand: String,
}

impl SqlBuilder for UnaryOperationBuilder {
    fn to_sql(&self) -> ToSqlResult<String> {
        Ok(format!("({}{})", self.operator, self.operand))
    }
}

pub struct IdentifierBuilder {
    pub name: String,
}

impl SqlBuilder for IdentifierBuilder {
    fn to_sql(&self) -> ToSqlResult<String> {
        Ok(self.name.clone())
    }
}

pub struct LiteralBuilder {
    pub value: String,
}

impl SqlBuilder for LiteralBuilder {
    fn to_sql(&self) -> ToSqlResult<String> {
        Ok(self.value.clone())
    }
}

pub struct ParensBuilder {
    pub inner: String,
}

impl SqlBuilder for ParensBuilder {
    fn to_sql(&self) -> ToSqlResult<String> {
        Ok(format!("({})", self.inner))
    }
}

pub struct MemberAccessBuilder {
    pub base: String,
    pub members: Vec<String>,
}

impl SqlBuilder for MemberAccessBuilder {
    fn to_sql(&self) -> ToSqlResult<String> {
        let mut result = self.base.clone();
        for member in &self.members {
            result += member;
        }
        Ok(result)
    }
}

pub struct UnsupportedBuilder {
    pub message: String,
}

impl SqlBuilder for UnsupportedBuilder {
    fn to_sql(&self) -> ToSqlResult<String> {
        Err(ToSqlError::unsupported(&self.message))
    }
}

pub struct CastBuilder {
    pub value: String,
    pub cast_type: String,
}

impl SqlBuilder for CastBuilder {
    fn to_sql(&self) -> ToSqlResult<String> {
        Ok(format!("{}::{}", self.value, self.cast_type))
    }
}

pub struct JsonObjectBuilder {
    pub fields: Vec<(String, String)>,
}

impl SqlBuilder for JsonObjectBuilder {
    fn to_sql(&self) -> ToSqlResult<String> {
        if self.fields.is_empty() {
            Ok("'{}'::json".to_string())
        } else {
            let field_pairs: Vec<String> = self
                .fields
                .iter()
                .map(|(key, value)| format!("'{}', {}", key, value))
                .collect();
            Ok(format!("json_build_object({})", field_pairs.join(", ")))
        }
    }
}

pub struct JsonMemberAccessBuilder {
    pub object: String,
    pub field: String,
    pub extract_text: bool,
}

impl SqlBuilder for JsonMemberAccessBuilder {
    fn to_sql(&self) -> ToSqlResult<String> {
        if self.extract_text {
            Ok(format!("({})->>'{}'", self.object, self.field))
        } else {
            Ok(format!("({})->'{}'", self.object, self.field))
        }
    }
}

pub struct ArrayBuilder {
    pub elements: Vec<String>,
}

impl SqlBuilder for ArrayBuilder {
    fn to_sql(&self) -> ToSqlResult<String> {
        if self.elements.is_empty() {
            Ok("ARRAY[]".to_string())
        } else {
            Ok(format!("ARRAY[{}]", self.elements.join(", ")))
        }
    }
}
