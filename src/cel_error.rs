use std::fmt;

use parsel::Span;

#[derive(Debug)]
pub enum CelError {
    Misc(String),
    Syntax {
        start_line: usize,
        start_column: usize,
        end_line: usize,
        end_column: usize,
    },
    Value(String),
    Argument(String),
    InvalidOp(String),
    Runtime(String),
    Binding {
        symbol: String,
    },

    Internal(String),
}
pub type CelResult<T> = Result<T, CelError>;

impl CelError {
    pub fn misc(msg: &str) -> CelError {
        CelError::Misc(msg.to_owned())
    }

    pub fn syntax(span: &Span) -> CelError {
        CelError::Syntax {
            start_line: span.start().line,
            start_column: span.start().column,
            end_line: span.end().line,
            end_column: span.end().column,
        }
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
        }
    }
}

impl fmt::Display for CelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CelError::*;

        match self {
            Misc(msg) => write!(f, "{}", msg),
            Syntax {
                start_line,
                start_column,
                end_line,
                end_column,
            } => {
                if start_line == end_line {
                    write!(
                        f,
                        "Syntax error on line {} {}:{}",
                        start_line, start_column, end_column
                    )
                } else {
                    write!(
                        f,
                        "Syntax error from {}:{} to {}:{}",
                        start_line, start_column, end_line, end_column
                    )
                }
            }
            Value(msg) => write!(f, "{}", msg),
            Argument(msg) => write!(f, "{}", msg),
            InvalidOp(msg) => write!(f, "{}", msg),

            Internal(msg) => write!(f, "{}", msg),
            Runtime(msg) => write!(f, "{}", msg),
            Binding { symbol } => write!(f, "Symbol not bound: {}", symbol),
        }
    }
}
