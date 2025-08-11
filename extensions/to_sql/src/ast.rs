use rscel::AstNode;

use crate::traits::ToSql;

impl<T: ToSql> ToSql for AstNode<T> {
    fn to_sql(&self) -> crate::error::ToSqlResult<String> {
        self.node().to_sql()
    }
}
