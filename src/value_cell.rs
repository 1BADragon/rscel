use std::{
    cmp::Ordering,
    collections::HashMap,
    iter::zip,
    ops::{Add, Div, Mul, Neg, Not, Rem, Sub},
};

use serde_json::{value::Value, Map};

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug)]
pub struct ValueCellError {
    msg: String,
}

impl ValueCellError {
    pub fn with_msg(msg: &str) -> ValueCellError {
        ValueCellError {
            msg: msg.to_owned(),
        }
    }

    pub fn msg<'a>(&'a self) -> &'a str {
        return &self.msg;
    }
}

pub type ValueCellResult<T> = Result<T, ValueCellError>;

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

    pub fn from_ident(val: &str) -> ValueCell {
        ValueCell::Ident(val.to_owned())
    }

    pub fn from_type(val: &str) -> ValueCell {
        ValueCell::Type(val.to_owned())
    }

    pub fn eq(&self, rhs: &ValueCell) -> ValueCellResult<ValueCell> {
        let type1 = rhs.as_type();
        let type2 = rhs.as_type();

        match self {
            ValueCell::Int(val1) => {
                match rhs {
                    ValueCell::Int(val2) => return Ok(ValueCell::from_bool(val1 == val2)),
                    ValueCell::UInt(val2) => {
                        return Ok(ValueCell::from_bool(*val1 == *val2 as i64))
                    }
                    ValueCell::Float(val2) => {
                        return Ok(ValueCell::from_bool(*val1 as f64 == *val2))
                    }
                    _ => {}
                };
            }
            ValueCell::UInt(val1) => {
                match rhs {
                    ValueCell::Int(val2) => return Ok(ValueCell::from_bool(*val1 as i64 == *val2)),
                    ValueCell::UInt(val2) => return Ok(ValueCell::from_bool(val1 == val2)),
                    ValueCell::Float(val2) => {
                        return Ok(ValueCell::from_bool(*val1 as f64 == *val2))
                    }
                    _ => {}
                };
            }
            ValueCell::Float(val1) => {
                match rhs {
                    ValueCell::Int(val2) => return Ok(ValueCell::from_bool(*val1 == *val2 as f64)),
                    ValueCell::UInt(val2) => {
                        return Ok(ValueCell::from_bool(*val1 == *val2 as f64))
                    }
                    ValueCell::Float(val2) => return Ok(ValueCell::from_bool(val1 == val2)),
                    _ => {}
                };
            }
            ValueCell::Bool(val1) => {
                if let ValueCell::Bool(val2) = rhs {
                    return Ok(ValueCell::from_bool(val1 == val2));
                }
            }
            ValueCell::String(val1) => {
                if let ValueCell::String(val2) = rhs {
                    return Ok(ValueCell::from_bool(val1 == val2));
                }
            }
            ValueCell::Bytes(val1) => {
                if let ValueCell::Bytes(val2) = rhs {
                    return Ok(ValueCell::from_bool(val1 == val2));
                }
            }
            ValueCell::List(val1) => {
                if let ValueCell::List(val2) = rhs {
                    if val1.len() != val2.len() {
                        return Ok(ValueCell::from_bool(false));
                    }

                    for (v1, v2) in zip(val1, val2) {
                        match v1.eq(v2) {
                            Ok(res_cell) => {
                                if let ValueCell::Bool(res) = res_cell {
                                    if !res {
                                        return Ok(ValueCell::from_bool(false));
                                    }
                                }
                            }
                            Err(_) => return Ok(ValueCell::from_bool(false)),
                        }
                    }
                    return Ok(ValueCell::from_bool(true));
                }
            }
            ValueCell::Null => {
                if let ValueCell::Null = rhs {
                    return Ok(ValueCell::from_bool(true));
                } else {
                    return Ok(ValueCell::from_bool(false));
                }
            }
            _ => {}
        }

        return Err(ValueCellError::with_msg(&format!(
            "Invalid op '==' between {:?} and {:?}",
            type1, type2
        )));
    }

    pub fn neq(&self, rhs: &ValueCell) -> ValueCellResult<ValueCell> {
        if let ValueCell::Bool(res) = self.eq(rhs)? {
            return Ok(ValueCell::from_bool(!res));
        }

        unreachable!();
    }

    fn ord(&self, rhs: &ValueCell) -> ValueCellResult<Option<Ordering>> {
        let type1 = rhs.as_type();
        let type2 = rhs.as_type();

        match self {
            ValueCell::Int(v1) => match rhs {
                ValueCell::Int(v2) => return Ok(Some(v1.cmp(v2))),
                ValueCell::UInt(v2) => return Ok(Some(v1.cmp(&(*v2 as i64)))),
                ValueCell::Float(v2) => return Ok((*v1 as f64).partial_cmp(v2)),
                _ => {}
            },
            ValueCell::UInt(v1) => match rhs {
                ValueCell::Int(v2) => return Ok(Some((*v1 as i64).cmp(v2))),
                ValueCell::UInt(v2) => return Ok(Some(v1.cmp(v2))),
                ValueCell::Float(v2) => return Ok((*v1 as f64).partial_cmp(v2)),
                _ => {}
            },
            ValueCell::Float(v1) => match rhs {
                ValueCell::Int(v2) => return Ok(v1.partial_cmp(&(*v2 as f64))),
                ValueCell::UInt(v2) => return Ok(v1.partial_cmp(&(*v2 as f64))),
                ValueCell::Float(v2) => return Ok(v1.partial_cmp(v2)),
                _ => {}
            },
            _ => {}
        }

        return Err(ValueCellError::with_msg(&format!(
            "Invalid op 'ord' between {:?} and {:?}",
            type1, type2
        )));
    }

    pub fn lt(&self, rhs: &ValueCell) -> ValueCellResult<ValueCell> {
        Ok(ValueCell::from_bool(self.ord(rhs)? == Some(Ordering::Less)))
    }

    pub fn gt(&self, rhs: &ValueCell) -> ValueCellResult<ValueCell> {
        Ok(ValueCell::from_bool(
            self.ord(rhs)? == Some(Ordering::Greater),
        ))
    }

    pub fn le(&self, rhs: &ValueCell) -> ValueCellResult<ValueCell> {
        let res = self.ord(rhs)?;

        Ok(ValueCell::from_bool(
            res == Some(Ordering::Less) || res == Some(Ordering::Equal),
        ))
    }

    pub fn ge(&self, rhs: &ValueCell) -> ValueCellResult<ValueCell> {
        let res = self.ord(rhs)?;

        Ok(ValueCell::from_bool(
            res == Some(Ordering::Greater) || res == Some(Ordering::Equal),
        ))
    }

    pub fn into_json_value(self) -> Value {
        match self {
            ValueCell::Int(val) => Value::from(val),
            ValueCell::UInt(val) => Value::from(val),
            ValueCell::Float(val) => Value::from(val),
            ValueCell::Bool(val) => Value::from(val),
            ValueCell::String(val) => Value::from(val),
            ValueCell::Bytes(val) => Value::from(val),
            ValueCell::List(val) => {
                let mut partial: Vec<Value> = Vec::new();

                for v in val.into_iter() {
                    partial.push(v.into_json_value());
                }

                Value::Array(partial)
            }
            ValueCell::Map(val) => {
                let mut partial: Map<String, Value> = Map::new();

                for (key, value) in val.into_iter() {
                    partial.insert(key, value.into_json_value());
                }

                Value::Object(partial)
            }
            _ => Value::Null,
        }
    }

    pub fn as_type(&self) -> ValueCell {
        match self {
            ValueCell::Int(_) => ValueCell::from_type("int"),
            ValueCell::UInt(_) => ValueCell::from_type("uint"),
            ValueCell::Float(_) => ValueCell::from_type("float"),
            ValueCell::Bool(_) => ValueCell::from_type("bool"),
            ValueCell::String(_) => ValueCell::from_type("string"),
            ValueCell::Bytes(_) => ValueCell::from_type("bytes"),
            ValueCell::List(_) => ValueCell::from_type("list"),
            ValueCell::Map(_) => ValueCell::from_type("map"),
            ValueCell::Null => ValueCell::from_type("null_type"),
            ValueCell::Ident(_) => ValueCell::from_type("ident"),
            ValueCell::Type(_) => ValueCell::from_type("type"),
        }
    }
}

impl From<&Value> for ValueCell {
    fn from(value: &Value) -> ValueCell {
        match value {
            Value::Number(val) => {
                if let Some(val) = val.as_i64() {
                    return ValueCell::from_int(val);
                } else if let Some(val) = val.as_u64() {
                    return ValueCell::from_uint(val);
                } else if let Some(val) = val.as_f64() {
                    return ValueCell::from_float(val);
                }

                unreachable!()
            }
            Value::String(val) => ValueCell::from_string(val),
            Value::Bool(val) => ValueCell::from_bool(*val),
            Value::Array(val) => {
                let list: Vec<ValueCell> = val.iter().map(|x| ValueCell::from(x)).collect();
                ValueCell::from_list(&list)
            }
            Value::Null => ValueCell::from_null(),
            Value::Object(val) => {
                let mut map: HashMap<String, ValueCell> = HashMap::new();

                for key in val.keys() {
                    map.insert(key.clone(), ValueCell::from(&val[key]));
                }

                ValueCell::from_map(&map)
            }
        }
    }
}

impl From<Value> for ValueCell {
    fn from(value: Value) -> ValueCell {
        match value {
            Value::Number(val) => {
                if let Some(val) = val.as_i64() {
                    return ValueCell::from_int(val);
                } else if let Some(val) = val.as_u64() {
                    return ValueCell::from_uint(val);
                } else if let Some(val) = val.as_f64() {
                    return ValueCell::from_float(val);
                }

                unreachable!()
            }
            Value::String(val) => ValueCell::from_string(&val),
            Value::Bool(val) => ValueCell::from_bool(val),
            Value::Array(val) => {
                let list: Vec<ValueCell> = val.iter().map(|x| ValueCell::from(x)).collect();
                ValueCell::from_list(&list)
            }
            Value::Null => ValueCell::from_null(),
            Value::Object(val) => {
                let mut map: HashMap<String, ValueCell> = HashMap::new();

                for key in val.keys() {
                    map.insert(key.clone(), ValueCell::from(&val[key]));
                }

                ValueCell::from_map(&map)
            }
        }
    }
}

impl From<i64> for ValueCell {
    fn from(val: i64) -> ValueCell {
        ValueCell::from_int(val)
    }
}

impl From<i32> for ValueCell {
    fn from(val: i32) -> ValueCell {
        ValueCell::from_int(val as i64)
    }
}

impl From<i16> for ValueCell {
    fn from(val: i16) -> ValueCell {
        ValueCell::from_int(val as i64)
    }
}

impl From<i8> for ValueCell {
    fn from(val: i8) -> ValueCell {
        ValueCell::from_int(val as i64)
    }
}

impl TryInto<i64> for ValueCell {
    type Error = ValueCellError;

    fn try_into(self) -> ValueCellResult<i64> {
        if let ValueCell::Int(val) = self {
            return Ok(val);
        }

        return Err(ValueCellError::with_msg("Convertion Error"));
    }
}

impl From<u64> for ValueCell {
    fn from(val: u64) -> ValueCell {
        ValueCell::from_uint(val)
    }
}

impl From<u32> for ValueCell {
    fn from(val: u32) -> ValueCell {
        ValueCell::from_uint(val as u64)
    }
}

impl From<u16> for ValueCell {
    fn from(val: u16) -> ValueCell {
        ValueCell::from_uint(val as u64)
    }
}

impl From<u8> for ValueCell {
    fn from(val: u8) -> ValueCell {
        ValueCell::from_uint(val as u64)
    }
}

impl TryInto<u64> for ValueCell {
    type Error = ValueCellError;

    fn try_into(self) -> ValueCellResult<u64> {
        if let ValueCell::UInt(val) = self {
            return Ok(val);
        }

        return Err(ValueCellError::with_msg("Convertion Error"));
    }
}

impl From<f64> for ValueCell {
    fn from(val: f64) -> ValueCell {
        ValueCell::from_float(val)
    }
}

impl TryInto<f64> for ValueCell {
    type Error = ValueCellError;

    fn try_into(self) -> ValueCellResult<f64> {
        if let ValueCell::Float(val) = self {
            return Ok(val);
        }

        return Err(ValueCellError::with_msg("Convertion Error"));
    }
}

impl From<bool> for ValueCell {
    fn from(val: bool) -> ValueCell {
        ValueCell::from_bool(val)
    }
}

impl TryInto<bool> for ValueCell {
    type Error = ValueCellError;

    fn try_into(self) -> ValueCellResult<bool> {
        if let ValueCell::Bool(val) = self {
            return Ok(val);
        }

        return Err(ValueCellError::with_msg("Convertion Error"));
    }
}

impl From<&str> for ValueCell {
    fn from(val: &str) -> ValueCell {
        ValueCell::from_string(val)
    }
}

impl From<String> for ValueCell {
    fn from(val: String) -> ValueCell {
        ValueCell::String(val)
    }
}

impl TryInto<String> for ValueCell {
    type Error = ValueCellError;

    fn try_into(self) -> ValueCellResult<String> {
        if let ValueCell::String(val) = self {
            return Ok(val);
        }

        return Err(ValueCellError::with_msg("Convertion Error"));
    }
}

impl From<&[u8]> for ValueCell {
    fn from(val: &[u8]) -> ValueCell {
        ValueCell::from_bytes(val)
    }
}
impl From<Vec<u8>> for ValueCell {
    fn from(val: Vec<u8>) -> ValueCell {
        ValueCell::Bytes(val)
    }
}

impl TryInto<Vec<u8>> for ValueCell {
    type Error = ValueCellError;

    fn try_into(self) -> ValueCellResult<Vec<u8>> {
        if let ValueCell::Bytes(val) = self {
            return Ok(val);
        }

        return Err(ValueCellError::with_msg("Convertion Error"));
    }
}

impl From<&[ValueCell]> for ValueCell {
    fn from(val: &[ValueCell]) -> ValueCell {
        ValueCell::from_list(val)
    }
}

impl From<Vec<ValueCell>> for ValueCell {
    fn from(val: Vec<ValueCell>) -> ValueCell {
        ValueCell::List(val)
    }
}

impl TryInto<Vec<ValueCell>> for ValueCell {
    type Error = ValueCellError;

    fn try_into(self) -> ValueCellResult<Vec<ValueCell>> {
        if let ValueCell::List(val) = self {
            return Ok(val);
        }

        return Err(ValueCellError::with_msg("Convertion Error"));
    }
}

impl From<HashMap<String, ValueCell>> for ValueCell {
    fn from(val: HashMap<String, ValueCell>) -> ValueCell {
        ValueCell::from_map(&val)
    }
}

impl TryInto<HashMap<String, ValueCell>> for ValueCell {
    type Error = ValueCellError;

    fn try_into(self) -> ValueCellResult<HashMap<String, ValueCell>> {
        if let ValueCell::Map(val) = self {
            return Ok(val);
        }

        return Err(ValueCellError::with_msg("Convertion Error"));
    }
}

impl Add for ValueCell {
    type Output = ValueCellResult<ValueCell>;

    fn add(self, rhs: Self) -> Self::Output {
        let type1 = self.as_type();
        let type2 = rhs.as_type();

        match self {
            ValueCell::Int(val1) => {
                if let ValueCell::Int(val2) = rhs {
                    return Ok(ValueCell::from(val1 + val2));
                }
            }
            ValueCell::UInt(val1) => {
                if let ValueCell::UInt(val2) = rhs {
                    return Ok(ValueCell::from(val1 + val2));
                }
            }
            ValueCell::Float(val1) => {
                if let ValueCell::Float(val2) = rhs {
                    return Ok(ValueCell::from(val1 + val2));
                }
            }
            ValueCell::String(val1) => {
                if let ValueCell::String(val2) = rhs {
                    let mut res = val1;
                    res.push_str(&val2);
                    return Ok(ValueCell::from(res));
                }
            }
            ValueCell::Bytes(val1) => {
                if let ValueCell::Bytes(val2) = rhs {
                    let mut res = val1;
                    res.extend_from_slice(&val2);
                    return Ok(ValueCell::from(res));
                }
            }
            ValueCell::List(val1) => {
                if let ValueCell::List(val2) = rhs {
                    let mut res = val1;
                    res.extend_from_slice(&val2);
                    return Ok(res.into());
                }
            }
            _ => {}
        }

        Err(ValueCellError::with_msg(&format!(
            "Invalid op '+' between {:?} and {:?}",
            type1, type2
        )))
    }
}

impl Sub for ValueCell {
    type Output = ValueCellResult<ValueCell>;

    fn sub(self, rhs: Self) -> Self::Output {
        let type1 = self.as_type();
        let type2 = rhs.as_type();

        match self {
            ValueCell::Int(val1) => {
                if let ValueCell::Int(val2) = rhs {
                    return Ok(ValueCell::from(val1 - val2));
                }
            }
            ValueCell::UInt(val1) => {
                if let ValueCell::UInt(val2) = rhs {
                    return Ok(ValueCell::from(val1 - val2));
                }
            }
            ValueCell::Float(val1) => {
                if let ValueCell::Float(val2) = rhs {
                    return Ok(ValueCell::from(val1 - val2));
                }
            }
            _ => {}
        }

        Err(ValueCellError::with_msg(&format!(
            "Invalid op '-' between {:?} and {:?}",
            type1, type2
        )))
    }
}

impl Mul for ValueCell {
    type Output = ValueCellResult<ValueCell>;

    fn mul(self, rhs: Self) -> Self::Output {
        let type1 = self.as_type();
        let type2 = rhs.as_type();

        match self {
            ValueCell::Int(val1) => {
                if let ValueCell::Int(val2) = rhs {
                    return Ok(ValueCell::from(val1 * val2));
                }
            }
            ValueCell::UInt(val1) => {
                if let ValueCell::UInt(val2) = rhs {
                    return Ok(ValueCell::from(val1 * val2));
                }
            }
            ValueCell::Float(val1) => {
                if let ValueCell::Float(val2) = rhs {
                    return Ok(ValueCell::from(val1 * val2));
                }
            }
            _ => {}
        }

        Err(ValueCellError::with_msg(&format!(
            "Invalid op '*' between {:?} and {:?}",
            type1, type2
        )))
    }
}

impl Div for ValueCell {
    type Output = ValueCellResult<ValueCell>;

    fn div(self, rhs: Self) -> Self::Output {
        let type1 = self.as_type();
        let type2 = rhs.as_type();

        match self {
            ValueCell::Int(val1) => {
                if let ValueCell::Int(val2) = rhs {
                    return Ok(ValueCell::from(val1 / val2));
                }
            }
            ValueCell::UInt(val1) => {
                if let ValueCell::UInt(val2) = rhs {
                    return Ok(ValueCell::from(val1 / val2));
                }
            }
            ValueCell::Float(val1) => {
                if let ValueCell::Float(val2) = rhs {
                    return Ok(ValueCell::from(val1 / val2));
                }
            }
            _ => {}
        }

        Err(ValueCellError::with_msg(&format!(
            "Invalid op '/' between {:?} and {:?}",
            type1, type2
        )))
    }
}

impl Rem for ValueCell {
    type Output = ValueCellResult<ValueCell>;

    fn rem(self, rhs: Self) -> Self::Output {
        let type1 = self.as_type();
        let type2 = rhs.as_type();

        match self {
            ValueCell::Int(val1) => {
                if let ValueCell::Int(val2) = rhs {
                    return Ok(ValueCell::from(val1 % val2));
                }
            }
            ValueCell::UInt(val1) => {
                if let ValueCell::UInt(val2) = rhs {
                    return Ok(ValueCell::from(val1 % val2));
                }
            }
            _ => {}
        }

        Err(ValueCellError::with_msg(&format!(
            "Invalid op '/' between {:?} and {:?}",
            type1, type2
        )))
    }
}

impl Neg for ValueCell {
    type Output = ValueCellResult<ValueCell>;

    fn neg(self) -> Self::Output {
        let type1 = self.as_type();

        match self {
            ValueCell::Int(val1) => {
                return Ok(ValueCell::from(-val1));
            }
            ValueCell::Float(val1) => {
                return Ok(ValueCell::from(-val1));
            }
            _ => {}
        }

        Err(ValueCellError::with_msg(&format!(
            "Invalid op '-' on {:?}",
            type1
        )))
    }
}

impl Not for ValueCell {
    type Output = ValueCellResult<ValueCell>;

    fn not(self) -> Self::Output {
        let type1 = self.as_type();

        match self {
            ValueCell::Bool(val1) => {
                return Ok(ValueCell::from(!val1));
            }
            _ => {}
        }

        Err(ValueCellError::with_msg(&format!(
            "Invalid op '-' on {:?}",
            type1
        )))
    }
}

#[cfg(test)]
mod test {
    use super::ValueCell;

    #[test]
    fn test_add() {
        let res = ValueCell::from(4i64) + ValueCell::from(5i64);

        assert!(res.is_ok());
        let val: i64 = res.ok().unwrap().try_into().unwrap();
        assert!(val == 9);
    }

    #[test]
    fn test_bad_op() {
        let res = ValueCell::from(3i64) + ValueCell::from(4.2);

        assert!(res.is_err());
        println!("Failure: {}", res.err().unwrap().msg());
    }
}
