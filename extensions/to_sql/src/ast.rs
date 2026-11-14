use rscel::AstNode;

use crate::error::ToSqlError;
use crate::traits::{SqlBuilder, ToSqlBuilder};

impl<T: ToSqlBuilder> ToSqlBuilder for AstNode<T> {
    fn to_sql_builder(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        self.node().to_sql_builder()
    }
}
