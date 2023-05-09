use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum ValueCell {
    Int(i64),
    UInt(u64),
    Float(f64),
    Bool(bool),
    String(String),
    Bytes(Vec<u8>),
    List(Vec<ValueCell>),
    Map(HashMap<String, ValueCell>),
    Null,
    Ident(String),
    Type(String),
}

impl ValueCell {
    pub fn from_int(val: i64) -> ValueCell {
        ValueCell::Int(val)
    }

    pub fn from_uint(val: u64) -> ValueCell {
        ValueCell::UInt(val)
    }

    pub fn from_float(val: f64) -> ValueCell {
        ValueCell::Float(val)
    }

    pub fn from_bool(val: bool) -> ValueCell {
        ValueCell::Bool(val)
    }

    pub fn from_string(val: &str) -> ValueCell {
        ValueCell::String(val.to_owned())
    }

    pub fn from_bytes(val: &[u8]) -> ValueCell {
        ValueCell::Bytes(val.to_vec())
    }

    pub fn from_list(val: &[ValueCell]) -> ValueCell {
        ValueCell::List(val.to_vec())
    }

    pub fn from_map(val: &HashMap<String, ValueCell>) -> ValueCell {
        ValueCell::Map(val.clone())
    }

    pub fn from_null() -> ValueCell {
        ValueCell::Null
    }

    pub fn from_msg_name(val: &str) -> ValueCell {
        ValueCell::Ident(val.to_owned())
    }

    pub fn from_type(val: &str) -> ValueCell {
        ValueCell::Type(val.to_owned())
    }
}
