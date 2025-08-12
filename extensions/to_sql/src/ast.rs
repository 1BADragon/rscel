use rscel::AstNode;

use crate::error::ToSqlError;
use crate::traits::{SqlBuilder, ToSql};

impl<T: ToSql> ToSql for AstNode<T> {
    fn to_sql(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        self.node().to_sql()
    }
}
