use crate::error::ToSqlResult;

pub trait ToSql {
    fn to_sql(&self) -> ToSqlResult<String>;
}
