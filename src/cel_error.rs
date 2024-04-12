use std::{error, fmt};

use serde::Serialize;

use crate::compiler::syntax_error::SyntaxError;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CelError {
    Misc(String),
    Syntax(SyntaxError),
    Value(String),
    Argument(String),
    InvalidOp(String),
    Runtime(String),
    Binding { symbol: String },
    Attribute { parent: String, field: String },

    Internal(String),
}
pub type CelResult<T> = Result<T, CelError>;

impl CelError {
    pub fn misc(msg: &str) -> CelError {
        CelError::Misc(msg.to_owned())
    }

    pub fn syntax(err: SyntaxError) -> CelError {
        CelError::Syntax(err)
    }

    pub fn value(msg: &str) -> CelError {
        CelError::Value(msg.to_owned())
    }

    pub fn argument(msg: &str) -> CelError {
        CelError::Argument(msg.to_owned())
    }

    pub fn internal(msg: &str) -> CelError {
        CelError::Internal(msg.to_owned())
    }

    pub fn invalid_op(msg: &str) -> CelError {
        CelError::InvalidOp(msg.to_owned())
    }

    pub fn runtime(msg: &str) -> CelError {
        CelError::Runtime(msg.to_owned())
    }

    pub fn binding(sym_name: &str) -> CelError {
        CelError::Binding {
            symbol: sym_name.to_owned(),
        }
    }

    pub fn attribute(parent_name: &str, field_name: &str) -> CelError {
        CelError::Attribute {
            parent: parent_name.to_string(),
            field: field_name.to_string(),
        }
    }

    pub fn type_string(&self) -> &'static str {
        use CelError::*;

        match self {
            Misc(..) => "MISC",
            Syntax { .. } => "SYNTAX",
            Value(..) => "VALUE",
            Argument(..) => "ARGUMENT",
            InvalidOp(..) => "INVALID OP",

            Internal(..) => "INTERNAL",
            Runtime(_) => "RUNTIME",
            Binding { .. } => "BINDING",
            Attribute { .. } => "ATTRIBUTE",
        }
    }
}

impl From<SyntaxError> for CelError {
    fn from(value: SyntaxError) -> Self {
        CelError::Syntax(value)
    }
}

impl fmt::Display for CelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CelError::*;

        match self {
            Misc(msg) => write!(f, "{}", msg),
            Syntax(err) => write!(f, "Syntax Error: {}", err),

            Value(msg) => write!(f, "{}", msg),
            Argument(msg) => write!(f, "{}", msg),
            InvalidOp(msg) => write!(f, "{}", msg),

            Internal(msg) => write!(f, "{}", msg),
            Runtime(msg) => write!(f, "{}", msg),
            Binding { symbol } => write!(f, "Symbol not bound: {}", symbol),
            Attribute { parent, field } => {
                write!(f, "Field {} does not exist on {}", field, parent)
            }
        }
    }
}

impl error::Error for CelError {}
