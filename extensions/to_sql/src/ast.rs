use rscel::AstNode;

use crate::error::ToSqlError;
use crate::traits::{IntoSqlBuilder, SqlBuilder};

impl<T: IntoSqlBuilder> IntoSqlBuilder for AstNode<T> {
    fn into_sql_builder(&self) -> Result<Box<dyn SqlBuilder>, ToSqlError> {
        self.node().into_sql_builder()
    }
}
