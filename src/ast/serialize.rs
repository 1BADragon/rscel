use serde::ser::{Serialize, SerializeStruct, Serializer};

use crate::ast::grammar::*;

impl Serialize for Expr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Expr", 2)?;
        state.serialize_field("cond_or", &self.cond_or)?;
        match self.ternary.as_prefix() {
            Some(v) => state.serialize_some("ternary", &v)?,
            None => state.serialize_none()?,
        }
        state.end()
    }
}
