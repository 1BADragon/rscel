use chrono::{offset::Utc, DateTime, Duration, FixedOffset};
use std::{
    cmp::Ordering,
    collections::HashMap,
    iter::zip,
    ops::{Add, Div, Mul, Neg, Not, Rem, Sub},
    rc::Rc,
};

use serde_json::{value::Value, Map};

#[derive(Debug, Clone, PartialEq)]
pub enum ValueCellInner {
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
    TimeStamp(DateTime<Utc>),
    Duration(Duration),
}

#[derive(Debug)]
pub struct ValueCell {
    inner: Rc<ValueCellInner>,
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

impl From<ValueCellInner> for ValueCell {
    fn from(inner: ValueCellInner) -> ValueCell {
        return ValueCell {
            inner: Rc::new(inner),
        };
    }
}

impl Clone for ValueCell {
    fn clone(&self) -> ValueCell {
        ValueCell {
            inner: self.inner.clone(),
        }
    }
}

impl PartialEq for ValueCell {
    fn eq(&self, rhs: &ValueCell) -> bool {
        self.inner().eq(rhs.inner())
    }
}

impl AsRef<ValueCellInner> for ValueCell {
    fn as_ref(&self) -> &ValueCellInner {
        self.inner()
    }
}

impl ValueCell {
    pub fn from_int(val: i64) -> ValueCell {
        ValueCellInner::Int(val).into()
    }

    pub fn from_uint(val: u64) -> ValueCell {
        ValueCellInner::UInt(val).into()
    }

    pub fn from_float(val: f64) -> ValueCell {
        ValueCellInner::Float(val).into()
    }

    pub fn from_bool(val: bool) -> ValueCell {
        ValueCellInner::Bool(val).into()
    }

    pub fn from_string(val: &str) -> ValueCell {
        ValueCellInner::String(val.to_owned()).into()
    }

    pub fn from_bytes(val: &[u8]) -> ValueCell {
        ValueCellInner::Bytes(val.to_vec()).into()
    }

    pub fn from_list(val: &[ValueCell]) -> ValueCell {
        ValueCellInner::List(val.to_vec()).into()
    }

    pub fn from_map(val: &HashMap<String, ValueCell>) -> ValueCell {
        ValueCellInner::Map(val.clone()).into()
    }

    pub fn from_null() -> ValueCell {
        ValueCellInner::Null.into()
    }

    pub fn from_ident(val: &str) -> ValueCell {
        ValueCellInner::Ident(val.to_owned()).into()
    }

    pub fn from_type(val: &str) -> ValueCell {
        ValueCellInner::Type(val.to_owned()).into()
    }

    pub fn from_timestamp(val: &DateTime<Utc>) -> ValueCell {
        ValueCellInner::TimeStamp(val.clone()).into()
    }

    pub fn from_duration(val: &Duration) -> ValueCell {
        ValueCellInner::Duration(val.clone()).into()
    }

    pub fn into_inner(self) -> ValueCellInner {
        match Rc::try_unwrap(self.inner) {
            Ok(inner) => inner,
            Err(rc) => (*rc).clone(),
        }
    }

    pub fn inner<'l>(&'l self) -> &'l ValueCellInner {
        &self.inner
    }

    pub fn eq(&self, rhs: &ValueCell) -> ValueCellResult<ValueCell> {
        let type1 = rhs.as_type();
        let type2 = rhs.as_type();

        match self.inner() {
            ValueCellInner::Int(val1) => {
                match rhs.inner() {
                    ValueCellInner::Int(val2) => return Ok(ValueCell::from_bool(val1 == val2)),
                    ValueCellInner::UInt(val2) => {
                        return Ok(ValueCell::from_bool(*val1 == *val2 as i64))
                    }
                    ValueCellInner::Float(val2) => {
                        return Ok(ValueCell::from_bool(*val1 as f64 == *val2))
                    }
                    _ => {}
                };
            }
            ValueCellInner::UInt(val1) => {
                match rhs.inner() {
                    ValueCellInner::Int(val2) => {
                        return Ok(ValueCell::from_bool(*val1 as i64 == *val2))
                    }
                    ValueCellInner::UInt(val2) => return Ok(ValueCell::from_bool(val1 == val2)),
                    ValueCellInner::Float(val2) => {
                        return Ok(ValueCell::from_bool(*val1 as f64 == *val2))
                    }
                    _ => {}
                };
            }
            ValueCellInner::Float(val1) => {
                match rhs.inner() {
                    ValueCellInner::Int(val2) => {
                        return Ok(ValueCell::from_bool(*val1 == *val2 as f64))
                    }
                    ValueCellInner::UInt(val2) => {
                        return Ok(ValueCell::from_bool(*val1 == *val2 as f64))
                    }
                    ValueCellInner::Float(val2) => return Ok(ValueCell::from_bool(val1 == val2)),
                    _ => {}
                };
            }
            ValueCellInner::Bool(val1) => {
                if let ValueCellInner::Bool(val2) = rhs.inner() {
                    return Ok(ValueCell::from_bool(val1 == val2));
                }
            }
            ValueCellInner::String(val1) => {
                if let ValueCellInner::String(val2) = rhs.inner() {
                    return Ok(ValueCell::from_bool(val1 == val2));
                }
            }
            ValueCellInner::Bytes(val1) => {
                if let ValueCellInner::Bytes(val2) = rhs.inner() {
                    return Ok(ValueCell::from_bool(val1 == val2));
                }
            }
            ValueCellInner::List(val1) => {
                if let ValueCellInner::List(val2) = rhs.inner() {
                    if val1.len() != val2.len() {
                        return Ok(ValueCell::from_bool(false));
                    }

                    for (v1, v2) in zip(val1, val2) {
                        match v1.eq(v2) {
                            Ok(res_cell) => {
                                if let ValueCellInner::Bool(res) = res_cell.inner() {
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
            ValueCellInner::Null => {
                if let ValueCellInner::Null = rhs.inner() {
                    return Ok(ValueCell::from_bool(true));
                } else {
                    return Ok(ValueCell::from_bool(false));
                }
            }
            ValueCellInner::TimeStamp(v1) => {
                if let ValueCellInner::TimeStamp(v2) = rhs.inner() {
                    return Ok(ValueCell::from_bool(*v1 == *v2));
                }
            }
            ValueCellInner::Duration(v1) => {
                if let ValueCellInner::Duration(v2) = rhs.inner() {
                    return Ok(ValueCell::from_bool(*v1 == *v2));
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
        if let ValueCellInner::Bool(res) = self.eq(rhs)?.inner() {
            return Ok(ValueCell::from_bool(!res));
        }

        unreachable!();
    }

    fn ord(&self, rhs: &ValueCell) -> ValueCellResult<Option<Ordering>> {
        let type1 = rhs.as_type();
        let type2 = rhs.as_type();

        match self.inner() {
            ValueCellInner::Int(v1) => match rhs.inner() {
                ValueCellInner::Int(v2) => return Ok(Some(v1.cmp(v2))),
                ValueCellInner::UInt(v2) => return Ok(Some(v1.cmp(&(*v2 as i64)))),
                ValueCellInner::Float(v2) => return Ok((*v1 as f64).partial_cmp(v2)),
                _ => {}
            },
            ValueCellInner::UInt(v1) => match rhs.inner() {
                ValueCellInner::Int(v2) => return Ok(Some((*v1 as i64).cmp(v2))),
                ValueCellInner::UInt(v2) => return Ok(Some(v1.cmp(v2))),
                ValueCellInner::Float(v2) => return Ok((*v1 as f64).partial_cmp(v2)),
                _ => {}
            },
            ValueCellInner::Float(v1) => match rhs.inner() {
                ValueCellInner::Int(v2) => return Ok(v1.partial_cmp(&(*v2 as f64))),
                ValueCellInner::UInt(v2) => return Ok(v1.partial_cmp(&(*v2 as f64))),
                ValueCellInner::Float(v2) => return Ok(v1.partial_cmp(v2)),
                _ => {}
            },
            ValueCellInner::TimeStamp(v1) => {
                if let ValueCellInner::TimeStamp(v2) = rhs.inner() {
                    return Ok(v1.partial_cmp(v2));
                }
            }
            ValueCellInner::Duration(v1) => {
                if let ValueCellInner::Duration(v2) = rhs.inner() {
                    return Ok(v1.partial_cmp(v2));
                }
            }
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
        match self.into_inner() {
            ValueCellInner::Int(val) => Value::from(val),
            ValueCellInner::UInt(val) => Value::from(val),
            ValueCellInner::Float(val) => Value::from(val),
            ValueCellInner::Bool(val) => Value::from(val),
            ValueCellInner::String(val) => Value::from(val),
            ValueCellInner::Bytes(val) => Value::from(val),
            ValueCellInner::List(val) => {
                let mut partial: Vec<Value> = Vec::new();

                for v in val.into_iter() {
                    partial.push(v.into_json_value());
                }

                Value::Array(partial)
            }
            ValueCellInner::Map(val) => {
                let mut partial: Map<String, Value> = Map::new();

                for (key, value) in val.into_iter() {
                    partial.insert(key, value.into_json_value());
                }

                Value::Object(partial)
            }
            ValueCellInner::TimeStamp(val) => Value::from(val.to_rfc3339()),
            ValueCellInner::Duration(val) => Value::from(val.to_string()),
            _ => Value::Null,
        }
    }

    pub fn as_type(&self) -> ValueCell {
        match self.inner() {
            ValueCellInner::Int(_) => ValueCell::from_type("int"),
            ValueCellInner::UInt(_) => ValueCell::from_type("uint"),
            ValueCellInner::Float(_) => ValueCell::from_type("float"),
            ValueCellInner::Bool(_) => ValueCell::from_type("bool"),
            ValueCellInner::String(_) => ValueCell::from_type("string"),
            ValueCellInner::Bytes(_) => ValueCell::from_type("bytes"),
            ValueCellInner::List(_) => ValueCell::from_type("list"),
            ValueCellInner::Map(_) => ValueCell::from_type("map"),
            ValueCellInner::Null => ValueCell::from_type("null_type"),
            ValueCellInner::Ident(_) => ValueCell::from_type("ident"),
            ValueCellInner::Type(_) => ValueCell::from_type("type"),
            ValueCellInner::TimeStamp(_) => ValueCell::from_type("timestamp"),
            ValueCellInner::Duration(_) => ValueCell::from_type("duration"),
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
        if let ValueCellInner::Int(val) = self.inner() {
            return Ok(*val);
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
        if let ValueCellInner::UInt(val) = self.inner() {
            return Ok(*val);
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
        if let ValueCellInner::Float(val) = self.inner() {
            return Ok(*val);
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
        if let ValueCellInner::Bool(val) = self.inner() {
            return Ok(*val);
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
        ValueCellInner::String(val).into()
    }
}

impl TryInto<String> for ValueCell {
    type Error = ValueCellError;

    fn try_into(self) -> ValueCellResult<String> {
        if let ValueCellInner::String(val) = self.into_inner() {
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
        ValueCellInner::Bytes(val).into()
    }
}

impl TryInto<Vec<u8>> for ValueCell {
    type Error = ValueCellError;

    fn try_into(self) -> ValueCellResult<Vec<u8>> {
        if let ValueCellInner::Bytes(val) = self.into_inner() {
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
        ValueCellInner::List(val).into()
    }
}

impl TryInto<Vec<ValueCell>> for ValueCell {
    type Error = ValueCellError;

    fn try_into(self) -> ValueCellResult<Vec<ValueCell>> {
        if let ValueCellInner::List(val) = self.into_inner() {
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
        if let ValueCellInner::Map(val) = self.into_inner() {
            return Ok(val);
        }

        return Err(ValueCellError::with_msg("Convertion Error"));
    }
}

impl From<DateTime<Utc>> for ValueCell {
    fn from(val: DateTime<Utc>) -> ValueCell {
        ValueCell::from_timestamp(&val)
    }
}

impl From<DateTime<FixedOffset>> for ValueCell {
    fn from(val: DateTime<FixedOffset>) -> ValueCell {
        ValueCell::from_timestamp(&(val.into()))
    }
}

impl TryInto<DateTime<Utc>> for ValueCell {
    type Error = ValueCellError;

    fn try_into(self) -> ValueCellResult<DateTime<Utc>> {
        if let ValueCellInner::TimeStamp(val) = self.into_inner() {
            return Ok(val);
        }

        return Err(ValueCellError::with_msg("Convertion Error"));
    }
}

impl From<Duration> for ValueCell {
    fn from(val: Duration) -> ValueCell {
        ValueCell::from_duration(&val)
    }
}

impl TryInto<Duration> for ValueCell {
    type Error = ValueCellError;

    fn try_into(self) -> ValueCellResult<Duration> {
        if let ValueCellInner::Duration(val) = self.into_inner() {
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

        match self.into_inner() {
            ValueCellInner::Int(val1) => {
                if let ValueCellInner::Int(val2) = rhs.inner() {
                    return Ok(ValueCell::from(val1 + val2));
                }
            }
            ValueCellInner::UInt(val1) => {
                if let ValueCellInner::UInt(val2) = rhs.inner() {
                    return Ok(ValueCell::from(val1 + val2));
                }
            }
            ValueCellInner::Float(val1) => {
                if let ValueCellInner::Float(val2) = rhs.inner() {
                    return Ok(ValueCell::from(val1 + val2));
                }
            }
            ValueCellInner::String(val1) => {
                if let ValueCellInner::String(val2) = rhs.into_inner() {
                    let mut res = val1;
                    res.push_str(&val2);
                    return Ok(ValueCell::from(res));
                }
            }
            ValueCellInner::Bytes(val1) => {
                if let ValueCellInner::Bytes(val2) = rhs.inner() {
                    let mut res = val1;
                    res.extend_from_slice(&val2);
                    return Ok(ValueCell::from(res));
                }
            }
            ValueCellInner::List(val1) => {
                if let ValueCellInner::List(val2) = rhs.inner() {
                    let mut res = val1;
                    res.extend_from_slice(&val2);
                    return Ok(ValueCell::from(res));
                }
            }
            ValueCellInner::TimeStamp(v1) => {
                if let ValueCellInner::Duration(v2) = rhs.inner() {
                    return Ok(ValueCell::from_timestamp(&(v1 + *v2)));
                }
            }
            ValueCellInner::Duration(v1) => {
                if let ValueCellInner::TimeStamp(v2) = rhs.inner() {
                    return Ok(ValueCell::from_timestamp(&(*v2 + v1)));
                } else if let ValueCellInner::Duration(v2) = rhs.inner() {
                    return Ok(ValueCell::from_duration(&(v1 + *v2)));
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

        match self.inner() {
            ValueCellInner::Int(val1) => {
                if let ValueCellInner::Int(val2) = rhs.inner() {
                    return Ok(ValueCell::from(val1 - val2));
                }
            }
            ValueCellInner::UInt(val1) => {
                if let ValueCellInner::UInt(val2) = rhs.inner() {
                    return Ok(ValueCell::from(val1 - val2));
                }
            }
            ValueCellInner::Float(val1) => {
                if let ValueCellInner::Float(val2) = rhs.inner() {
                    return Ok(ValueCell::from(val1 - val2));
                }
            }
            ValueCellInner::TimeStamp(v1) => {
                if let ValueCellInner::Duration(v2) = rhs.inner() {
                    return Ok(ValueCell::from_timestamp(&(*v1 - *v2)));
                }
            }
            ValueCellInner::Duration(v1) => {
                if let ValueCellInner::TimeStamp(v2) = rhs.inner() {
                    return Ok(ValueCell::from_timestamp(&(*v2 - *v1)));
                } else if let ValueCellInner::Duration(v2) = rhs.inner() {
                    return Ok(ValueCell::from_duration(&(*v1 - *v2)));
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

        match self.inner() {
            ValueCellInner::Int(val1) => {
                if let ValueCellInner::Int(val2) = rhs.inner() {
                    return Ok(ValueCell::from(val1 * val2));
                }
            }
            ValueCellInner::UInt(val1) => {
                if let ValueCellInner::UInt(val2) = rhs.inner() {
                    return Ok(ValueCell::from(val1 * val2));
                }
            }
            ValueCellInner::Float(val1) => {
                if let ValueCellInner::Float(val2) = rhs.inner() {
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

        match self.inner() {
            ValueCellInner::Int(val1) => {
                if let ValueCellInner::Int(val2) = rhs.inner() {
                    return Ok(ValueCell::from(val1 / val2));
                }
            }
            ValueCellInner::UInt(val1) => {
                if let ValueCellInner::UInt(val2) = rhs.inner() {
                    return Ok(ValueCell::from(val1 / val2));
                }
            }
            ValueCellInner::Float(val1) => {
                if let ValueCellInner::Float(val2) = rhs.inner() {
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

        match self.inner() {
            ValueCellInner::Int(val1) => {
                if let ValueCellInner::Int(val2) = rhs.inner() {
                    return Ok(ValueCell::from(val1 % val2));
                }
            }
            ValueCellInner::UInt(val1) => {
                if let ValueCellInner::UInt(val2) = rhs.inner() {
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

        match self.inner() {
            ValueCellInner::Int(val1) => {
                return Ok(ValueCell::from(-val1));
            }
            ValueCellInner::Float(val1) => {
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

        match self.inner() {
            ValueCellInner::Bool(val1) => {
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
