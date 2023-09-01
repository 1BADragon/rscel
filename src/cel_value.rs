use chrono::{offset::Utc, DateTime, Duration, FixedOffset};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt,
    iter::zip,
    ops::{Add, Div, Mul, Neg, Not, Rem, Sub},
};

use serde_json::{value::Value, Map};

use crate::{interp::ByteCode, CelError, CelResult};

#[serde_as]
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum CelValue {
    Int(i64),
    UInt(u64),
    Float(f64),
    Bool(bool),
    String(String),
    Bytes(Vec<u8>),
    List(Vec<CelValue>),
    Map(HashMap<String, CelValue>),
    Null,
    Ident(String),
    Type(String),
    #[serde(skip_serializing, skip_deserializing)]
    TimeStamp(DateTime<Utc>),
    #[serde(skip_serializing, skip_deserializing)]
    Duration(Duration),
    ByteCode(Vec<ByteCode>),
}

impl CelValue {
    pub fn from_int(val: i64) -> CelValue {
        CelValue::Int(val)
    }

    pub fn from_uint(val: u64) -> CelValue {
        CelValue::UInt(val)
    }

    pub fn from_float(val: f64) -> CelValue {
        CelValue::Float(val)
    }

    pub fn from_bool(val: bool) -> CelValue {
        CelValue::Bool(val)
    }

    pub fn from_string(val: String) -> CelValue {
        CelValue::String(val)
    }

    pub fn from_bytes(val: Vec<u8>) -> CelValue {
        CelValue::Bytes(val)
    }

    pub fn from_list(val: Vec<CelValue>) -> CelValue {
        CelValue::List(val)
    }

    pub fn from_map(val: HashMap<String, CelValue>) -> CelValue {
        CelValue::Map(val)
    }

    pub fn from_null() -> CelValue {
        CelValue::Null
    }

    pub fn from_ident(val: &str) -> CelValue {
        CelValue::Ident(val.to_owned())
    }

    pub fn from_type(val: &str) -> CelValue {
        CelValue::Type(val.to_owned())
    }

    pub fn from_timestamp(val: &DateTime<Utc>) -> CelValue {
        CelValue::TimeStamp(val.clone())
    }

    pub fn from_duration(val: &Duration) -> CelValue {
        CelValue::Duration(val.clone())
    }

    pub(crate) fn from_bytecode(val: &[ByteCode]) -> CelValue {
        CelValue::ByteCode(val.to_owned())
    }

    pub fn is_true(&self) -> bool {
        if let CelValue::Bool(val) = self {
            *val
        } else {
            false
        }
    }

    pub fn is_null(&self) -> bool {
        if let CelValue::Null = self {
            true
        } else {
            false
        }
    }

    pub fn eq(&self, rhs: &CelValue) -> CelResult<CelValue> {
        let type1 = rhs.as_type();
        let type2 = rhs.as_type();

        match self {
            CelValue::Int(val1) => {
                match rhs {
                    CelValue::Int(val2) => return Ok(CelValue::from_bool(val1 == val2)),
                    CelValue::UInt(val2) => return Ok(CelValue::from_bool(*val1 == *val2 as i64)),
                    CelValue::Float(val2) => return Ok(CelValue::from_bool(*val1 as f64 == *val2)),
                    _ => {}
                };
            }
            CelValue::UInt(val1) => {
                match rhs {
                    CelValue::Int(val2) => return Ok(CelValue::from_bool(*val1 as i64 == *val2)),
                    CelValue::UInt(val2) => return Ok(CelValue::from_bool(val1 == val2)),
                    CelValue::Float(val2) => return Ok(CelValue::from_bool(*val1 as f64 == *val2)),
                    _ => {}
                };
            }
            CelValue::Float(val1) => {
                match rhs {
                    CelValue::Int(val2) => return Ok(CelValue::from_bool(*val1 == *val2 as f64)),
                    CelValue::UInt(val2) => return Ok(CelValue::from_bool(*val1 == *val2 as f64)),
                    CelValue::Float(val2) => return Ok(CelValue::from_bool(val1 == val2)),
                    _ => {}
                };
            }
            CelValue::Bool(val1) => {
                if let CelValue::Bool(val2) = rhs {
                    return Ok(CelValue::from_bool(val1 == val2));
                }
            }
            CelValue::String(val1) => {
                if let CelValue::String(val2) = rhs {
                    return Ok(CelValue::from_bool(val1 == val2));
                }
            }
            CelValue::Bytes(val1) => {
                if let CelValue::Bytes(val2) = rhs {
                    return Ok(CelValue::from_bool(val1 == val2));
                }
            }
            CelValue::List(val1) => {
                if let CelValue::List(val2) = rhs {
                    if val1.len() != val2.len() {
                        return Ok(CelValue::from_bool(false));
                    }

                    for (v1, v2) in zip(val1, val2) {
                        match v1.eq(v2) {
                            Ok(res_cell) => {
                                if let CelValue::Bool(res) = res_cell {
                                    if !res {
                                        return Ok(CelValue::from_bool(false));
                                    }
                                }
                            }
                            Err(_) => return Ok(CelValue::from_bool(false)),
                        }
                    }
                    return Ok(CelValue::from_bool(true));
                }
            }
            CelValue::Null => {
                if let CelValue::Null = rhs {
                    return Ok(CelValue::from_bool(true));
                } else {
                    return Ok(CelValue::from_bool(false));
                }
            }
            CelValue::TimeStamp(v1) => {
                if let CelValue::TimeStamp(v2) = rhs {
                    return Ok(CelValue::from_bool(*v1 == *v2));
                }
            }
            CelValue::Duration(v1) => {
                if let CelValue::Duration(v2) = rhs {
                    return Ok(CelValue::from_bool(*v1 == *v2));
                }
            }
            _ => {}
        }

        return Err(CelError::invalid_op(&format!(
            "Invalid op '==' between {:?} and {:?}",
            type1, type2
        )));
    }

    pub fn neq(&self, rhs: &CelValue) -> CelResult<CelValue> {
        if let CelValue::Bool(res) = self.eq(rhs)? {
            return Ok(CelValue::from_bool(!res));
        }

        unreachable!();
    }

    fn ord(&self, rhs: &CelValue) -> CelResult<Option<Ordering>> {
        let type1 = rhs.as_type();
        let type2 = rhs.as_type();

        match self {
            CelValue::Int(v1) => match rhs {
                CelValue::Int(v2) => return Ok(Some(v1.cmp(v2))),
                CelValue::UInt(v2) => return Ok(Some(v1.cmp(&(*v2 as i64)))),
                CelValue::Float(v2) => return Ok((*v1 as f64).partial_cmp(v2)),
                _ => {}
            },
            CelValue::UInt(v1) => match rhs {
                CelValue::Int(v2) => return Ok(Some((*v1 as i64).cmp(v2))),
                CelValue::UInt(v2) => return Ok(Some(v1.cmp(v2))),
                CelValue::Float(v2) => return Ok((*v1 as f64).partial_cmp(v2)),
                _ => {}
            },
            CelValue::Float(v1) => match rhs {
                CelValue::Int(v2) => return Ok(v1.partial_cmp(&(*v2 as f64))),
                CelValue::UInt(v2) => return Ok(v1.partial_cmp(&(*v2 as f64))),
                CelValue::Float(v2) => return Ok(v1.partial_cmp(v2)),
                _ => {}
            },
            CelValue::TimeStamp(v1) => {
                if let CelValue::TimeStamp(v2) = rhs {
                    return Ok(v1.partial_cmp(v2));
                }
            }
            CelValue::Duration(v1) => {
                if let CelValue::Duration(v2) = rhs {
                    return Ok(v1.partial_cmp(v2));
                }
            }
            _ => {}
        }

        return Err(CelError::invalid_op(&format!(
            "Invalid op 'ord' between {:?} and {:?}",
            type1, type2
        )));
    }

    pub fn lt(&self, rhs: &CelValue) -> CelResult<CelValue> {
        Ok(CelValue::from_bool(self.ord(rhs)? == Some(Ordering::Less)))
    }

    pub fn gt(&self, rhs: &CelValue) -> CelResult<CelValue> {
        Ok(CelValue::from_bool(
            self.ord(rhs)? == Some(Ordering::Greater),
        ))
    }

    pub fn le(&self, rhs: &CelValue) -> CelResult<CelValue> {
        let res = self.ord(rhs)?;

        Ok(CelValue::from_bool(
            res == Some(Ordering::Less) || res == Some(Ordering::Equal),
        ))
    }

    pub fn ge(&self, rhs: &CelValue) -> CelResult<CelValue> {
        let res = self.ord(rhs)?;

        Ok(CelValue::from_bool(
            res == Some(Ordering::Greater) || res == Some(Ordering::Equal),
        ))
    }

    pub fn or(&self, rhs: &CelValue) -> CelResult<CelValue> {
        if let CelValue::Bool(lhs) = self {
            if let CelValue::Bool(rhs) = rhs {
                return Ok((*lhs || *rhs).into());
            }
        }
        return Err(CelError::invalid_op(&format!(
            "|| operator invalid for {:?} and {:?}",
            self.as_type(),
            rhs.as_type(),
        )));
    }

    pub fn and(&self, rhs: &CelValue) -> CelResult<CelValue> {
        if let CelValue::Bool(lhs) = self {
            if let CelValue::Bool(rhs) = rhs {
                return Ok((*lhs && *rhs).into());
            }
        }
        return Err(CelError::invalid_op(&format!(
            "&& operator invalid for {:?} and {:?}",
            self.as_type(),
            rhs.as_type(),
        )));
    }

    pub fn into_json_value(self) -> Value {
        match self {
            CelValue::Int(val) => Value::from(val),
            CelValue::UInt(val) => Value::from(val),
            CelValue::Float(val) => Value::from(val),
            CelValue::Bool(val) => Value::from(val),
            CelValue::String(val) => Value::from(val),
            CelValue::Bytes(val) => Value::from(val),
            CelValue::List(val) => {
                let mut partial: Vec<Value> = Vec::new();

                for v in val.into_iter() {
                    partial.push(v.into_json_value());
                }

                Value::Array(partial)
            }
            CelValue::Map(val) => {
                let mut partial: Map<String, Value> = Map::new();

                for (key, value) in val.into_iter() {
                    partial.insert(key, value.into_json_value());
                }

                Value::Object(partial)
            }
            CelValue::TimeStamp(val) => Value::from(val.to_rfc3339()),
            CelValue::Duration(val) => Value::from(val.to_string()),
            _ => Value::Null,
        }
    }

    pub fn as_type(&self) -> CelValue {
        match self {
            CelValue::Int(_) => CelValue::from_type("int"),
            CelValue::UInt(_) => CelValue::from_type("uint"),
            CelValue::Float(_) => CelValue::from_type("float"),
            CelValue::Bool(_) => CelValue::from_type("bool"),
            CelValue::String(_) => CelValue::from_type("string"),
            CelValue::Bytes(_) => CelValue::from_type("bytes"),
            CelValue::List(_) => CelValue::from_type("list"),
            CelValue::Map(_) => CelValue::from_type("map"),
            CelValue::Null => CelValue::from_type("null_type"),
            CelValue::Ident(_) => CelValue::from_type("ident"),
            CelValue::Type(_) => CelValue::from_type("type"),
            CelValue::TimeStamp(_) => CelValue::from_type("timestamp"),
            CelValue::Duration(_) => CelValue::from_type("duration"),
            CelValue::ByteCode(_) => CelValue::from_type("bytecode"),
        }
    }
}

impl From<&Value> for CelValue {
    fn from(value: &Value) -> CelValue {
        match value {
            Value::Number(val) => {
                if let Some(val) = val.as_i64() {
                    return CelValue::from_int(val);
                } else if let Some(val) = val.as_u64() {
                    return CelValue::from_uint(val);
                } else if let Some(val) = val.as_f64() {
                    return CelValue::from_float(val);
                }

                unreachable!()
            }
            Value::String(val) => CelValue::from_string(val.clone()),
            Value::Bool(val) => CelValue::from_bool(*val),
            Value::Array(val) => {
                let list: Vec<CelValue> = val.iter().map(|x| CelValue::from(x)).collect();
                CelValue::from_list(list)
            }
            Value::Null => CelValue::from_null(),
            Value::Object(val) => {
                let mut map: HashMap<String, CelValue> = HashMap::new();

                for key in val.keys() {
                    map.insert(key.clone(), CelValue::from(&val[key]));
                }

                CelValue::from_map(map)
            }
        }
    }
}

impl From<Value> for CelValue {
    fn from(value: Value) -> CelValue {
        match value {
            Value::Number(val) => {
                if let Some(val) = val.as_i64() {
                    return CelValue::from_int(val);
                } else if let Some(val) = val.as_u64() {
                    return CelValue::from_uint(val);
                } else if let Some(val) = val.as_f64() {
                    return CelValue::from_float(val);
                }

                unreachable!()
            }
            Value::String(val) => CelValue::from_string(val),
            Value::Bool(val) => CelValue::from_bool(val),
            Value::Array(val) => {
                let list: Vec<CelValue> = val.iter().map(|x| CelValue::from(x)).collect();
                CelValue::from_list(list)
            }
            Value::Null => CelValue::from_null(),
            Value::Object(val) => {
                let mut map: HashMap<String, CelValue> = HashMap::new();

                for key in val.keys() {
                    map.insert(key.clone(), CelValue::from(&val[key]));
                }

                CelValue::from_map(map)
            }
        }
    }
}

impl From<i64> for CelValue {
    fn from(val: i64) -> CelValue {
        CelValue::from_int(val)
    }
}

impl From<i32> for CelValue {
    fn from(val: i32) -> CelValue {
        CelValue::from_int(val as i64)
    }
}

impl From<i16> for CelValue {
    fn from(val: i16) -> CelValue {
        CelValue::from_int(val as i64)
    }
}

impl From<i8> for CelValue {
    fn from(val: i8) -> CelValue {
        CelValue::from_int(val as i64)
    }
}

impl TryInto<i64> for CelValue {
    type Error = CelError;

    fn try_into(self) -> CelResult<i64> {
        if let CelValue::Int(val) = self {
            return Ok(val);
        }

        return Err(CelError::internal("Convertion Error"));
    }
}

impl From<u64> for CelValue {
    fn from(val: u64) -> CelValue {
        CelValue::from_uint(val)
    }
}

impl From<u32> for CelValue {
    fn from(val: u32) -> CelValue {
        CelValue::from_uint(val as u64)
    }
}

impl From<u16> for CelValue {
    fn from(val: u16) -> CelValue {
        CelValue::from_uint(val as u64)
    }
}

impl From<u8> for CelValue {
    fn from(val: u8) -> CelValue {
        CelValue::from_uint(val as u64)
    }
}

impl TryInto<u64> for CelValue {
    type Error = CelError;

    fn try_into(self) -> CelResult<u64> {
        if let CelValue::UInt(val) = self {
            return Ok(val);
        }

        return Err(CelError::internal("Convertion Error"));
    }
}

impl From<f64> for CelValue {
    fn from(val: f64) -> CelValue {
        CelValue::from_float(val)
    }
}

impl TryInto<f64> for CelValue {
    type Error = CelError;

    fn try_into(self) -> CelResult<f64> {
        if let CelValue::Float(val) = self {
            return Ok(val);
        }

        return Err(CelError::internal("Convertion Error"));
    }
}

impl From<bool> for CelValue {
    fn from(val: bool) -> CelValue {
        CelValue::from_bool(val)
    }
}

impl TryInto<bool> for CelValue {
    type Error = CelError;

    fn try_into(self) -> CelResult<bool> {
        if let CelValue::Bool(val) = self {
            return Ok(val);
        }

        return Err(CelError::internal("Convertion Error"));
    }
}

impl From<&str> for CelValue {
    fn from(val: &str) -> CelValue {
        CelValue::from_string(val.to_owned())
    }
}

impl From<String> for CelValue {
    fn from(val: String) -> CelValue {
        CelValue::from_string(val)
    }
}

impl TryInto<String> for CelValue {
    type Error = CelError;

    fn try_into(self) -> CelResult<String> {
        if let CelValue::String(val) = self {
            return Ok(val);
        }

        return Err(CelError::internal("Convertion Error"));
    }
}

impl From<&[u8]> for CelValue {
    fn from(val: &[u8]) -> CelValue {
        CelValue::from_bytes(val.to_owned())
    }
}

impl From<Vec<u8>> for CelValue {
    fn from(val: Vec<u8>) -> CelValue {
        CelValue::Bytes(val).into()
    }
}

impl TryInto<Vec<u8>> for CelValue {
    type Error = CelError;

    fn try_into(self) -> CelResult<Vec<u8>> {
        if let CelValue::Bytes(val) = self {
            return Ok(val);
        }

        return Err(CelError::internal("Convertion Error"));
    }
}

impl From<&[CelValue]> for CelValue {
    fn from(val: &[CelValue]) -> CelValue {
        CelValue::from_list(val.to_vec())
    }
}

impl From<Vec<CelValue>> for CelValue {
    fn from(val: Vec<CelValue>) -> CelValue {
        CelValue::List(val).into()
    }
}

impl TryInto<Vec<CelValue>> for CelValue {
    type Error = CelError;

    fn try_into(self) -> CelResult<Vec<CelValue>> {
        if let CelValue::List(val) = self {
            return Ok(val);
        }

        return Err(CelError::internal("Convertion Error"));
    }
}

impl From<HashMap<String, CelValue>> for CelValue {
    fn from(val: HashMap<String, CelValue>) -> CelValue {
        CelValue::from_map(val)
    }
}

impl TryInto<HashMap<String, CelValue>> for CelValue {
    type Error = CelError;

    fn try_into(self) -> CelResult<HashMap<String, CelValue>> {
        if let CelValue::Map(val) = self {
            return Ok(val);
        }

        return Err(CelError::internal("Convertion Error"));
    }
}

impl From<DateTime<Utc>> for CelValue {
    fn from(val: DateTime<Utc>) -> CelValue {
        CelValue::from_timestamp(&val)
    }
}

impl From<DateTime<FixedOffset>> for CelValue {
    fn from(val: DateTime<FixedOffset>) -> CelValue {
        CelValue::from_timestamp(&(val.into()))
    }
}

impl TryInto<DateTime<Utc>> for CelValue {
    type Error = CelError;

    fn try_into(self) -> CelResult<DateTime<Utc>> {
        if let CelValue::TimeStamp(val) = self {
            return Ok(val);
        }

        return Err(CelError::internal("Convertion Error"));
    }
}

impl From<Duration> for CelValue {
    fn from(val: Duration) -> CelValue {
        CelValue::from_duration(&val)
    }
}

impl TryInto<Duration> for CelValue {
    type Error = CelError;

    fn try_into(self) -> CelResult<Duration> {
        if let CelValue::Duration(val) = self {
            return Ok(val);
        }

        return Err(CelError::internal("Convertion Error"));
    }
}

impl From<Vec<ByteCode>> for CelValue {
    fn from(val: Vec<ByteCode>) -> CelValue {
        CelValue::from_bytecode(&val)
    }
}

impl TryInto<Vec<ByteCode>> for CelValue {
    type Error = CelError;

    fn try_into(self) -> CelResult<Vec<ByteCode>> {
        if let CelValue::ByteCode(val) = self {
            return Ok(val);
        }

        return Err(CelError::internal("Convertion Error"));
    }
}

impl Add for CelValue {
    type Output = CelResult<CelValue>;

    fn add(self, rhs: Self) -> Self::Output {
        let type1 = self.as_type();
        let type2 = rhs.as_type();

        match self {
            CelValue::Int(val1) => {
                if let CelValue::Int(val2) = rhs {
                    return Ok(CelValue::from(val1 + val2));
                }
            }
            CelValue::UInt(val1) => {
                if let CelValue::UInt(val2) = rhs {
                    return Ok(CelValue::from(val1 + val2));
                }
            }
            CelValue::Float(val1) => {
                if let CelValue::Float(val2) = rhs {
                    return Ok(CelValue::from(val1 + val2));
                }
            }
            CelValue::String(val1) => {
                if let CelValue::String(val2) = rhs {
                    let mut res = val1;
                    res.push_str(&val2);
                    return Ok(CelValue::from(res));
                }
            }
            CelValue::Bytes(val1) => {
                if let CelValue::Bytes(val2) = rhs {
                    let mut res = val1;
                    res.extend_from_slice(&val2);
                    return Ok(CelValue::from(res));
                }
            }
            CelValue::List(val1) => {
                if let CelValue::List(val2) = rhs {
                    let mut res = val1;
                    res.extend_from_slice(&val2);
                    return Ok(CelValue::from(res));
                }
            }
            CelValue::TimeStamp(v1) => {
                if let CelValue::Duration(v2) = rhs {
                    return Ok(CelValue::from_timestamp(&(v1 + v2)));
                }
            }
            CelValue::Duration(v1) => {
                if let CelValue::TimeStamp(v2) = rhs {
                    return Ok(CelValue::from_timestamp(&(v2 + v1)));
                } else if let CelValue::Duration(v2) = rhs {
                    return Ok(CelValue::from_duration(&(v1 + v2)));
                }
            }
            _ => {}
        }

        Err(CelError::invalid_op(&format!(
            "Invalid op '+' between {:?} and {:?}",
            type1, type2
        )))
    }
}

impl Sub for CelValue {
    type Output = CelResult<CelValue>;

    fn sub(self, rhs: Self) -> Self::Output {
        let type1 = self.as_type();
        let type2 = rhs.as_type();

        match self {
            CelValue::Int(val1) => {
                if let CelValue::Int(val2) = rhs {
                    return Ok(CelValue::from(val1 - val2));
                }
            }
            CelValue::UInt(val1) => {
                if let CelValue::UInt(val2) = rhs {
                    return Ok(CelValue::from(val1 - val2));
                }
            }
            CelValue::Float(val1) => {
                if let CelValue::Float(val2) = rhs {
                    return Ok(CelValue::from(val1 - val2));
                }
            }
            CelValue::TimeStamp(v1) => {
                if let CelValue::Duration(v2) = rhs {
                    return Ok(CelValue::from_timestamp(&(v1 - v2)));
                }
            }
            CelValue::Duration(v1) => {
                if let CelValue::TimeStamp(v2) = rhs {
                    return Ok(CelValue::from_timestamp(&(v2 - v1)));
                } else if let CelValue::Duration(v2) = rhs {
                    return Ok(CelValue::from_duration(&(v1 - v2)));
                }
            }
            _ => {}
        }

        Err(CelError::invalid_op(&format!(
            "Invalid op '-' between {:?} and {:?}",
            type1, type2
        )))
    }
}

impl Mul for CelValue {
    type Output = CelResult<CelValue>;

    fn mul(self, rhs: Self) -> Self::Output {
        let type1 = self.as_type();
        let type2 = rhs.as_type();

        match self {
            CelValue::Int(val1) => {
                if let CelValue::Int(val2) = rhs {
                    return Ok(CelValue::from(val1 * val2));
                }
            }
            CelValue::UInt(val1) => {
                if let CelValue::UInt(val2) = rhs {
                    return Ok(CelValue::from(val1 * val2));
                }
            }
            CelValue::Float(val1) => {
                if let CelValue::Float(val2) = rhs {
                    return Ok(CelValue::from(val1 * val2));
                }
            }
            _ => {}
        }

        Err(CelError::invalid_op(&format!(
            "Invalid op '*' between {:?} and {:?}",
            type1, type2
        )))
    }
}

impl Div for CelValue {
    type Output = CelResult<CelValue>;

    fn div(self, rhs: Self) -> Self::Output {
        let type1 = self.as_type();
        let type2 = rhs.as_type();

        match self {
            CelValue::Int(val1) => {
                if let CelValue::Int(val2) = rhs {
                    return Ok(CelValue::from(val1 / val2));
                }
            }
            CelValue::UInt(val1) => {
                if let CelValue::UInt(val2) = rhs {
                    return Ok(CelValue::from(val1 / val2));
                }
            }
            CelValue::Float(val1) => {
                if let CelValue::Float(val2) = rhs {
                    return Ok(CelValue::from(val1 / val2));
                }
            }
            _ => {}
        }

        Err(CelError::invalid_op(&format!(
            "Invalid op '/' between {:?} and {:?}",
            type1, type2
        )))
    }
}

impl Rem for CelValue {
    type Output = CelResult<CelValue>;

    fn rem(self, rhs: Self) -> Self::Output {
        let type1 = self.as_type();
        let type2 = rhs.as_type();

        match self {
            CelValue::Int(val1) => {
                if let CelValue::Int(val2) = rhs {
                    return Ok(CelValue::from(val1 % val2));
                }
            }
            CelValue::UInt(val1) => {
                if let CelValue::UInt(val2) = rhs {
                    return Ok(CelValue::from(val1 % val2));
                }
            }
            _ => {}
        }

        Err(CelError::invalid_op(&format!(
            "Invalid op '/' between {:?} and {:?}",
            type1, type2
        )))
    }
}

impl Neg for CelValue {
    type Output = CelResult<CelValue>;

    fn neg(self) -> Self::Output {
        let type1 = self.as_type();

        match self {
            CelValue::Int(val1) => {
                return Ok(CelValue::from(-val1));
            }
            CelValue::Float(val1) => {
                return Ok(CelValue::from(-val1));
            }
            _ => {}
        }

        Err(CelError::invalid_op(&format!(
            "Invalid op '-' on {:?}",
            type1
        )))
    }
}

impl Not for CelValue {
    type Output = CelResult<CelValue>;

    fn not(self) -> Self::Output {
        let type1 = self.as_type();

        match self {
            CelValue::Bool(val1) => {
                return Ok(CelValue::from(!val1));
            }
            _ => {}
        }

        Err(CelError::invalid_op(&format!(
            "Invalid op '-' on {:?}",
            type1
        )))
    }
}

impl fmt::Display for CelValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CelValue::*;
        match self {
            Int(val) => write!(f, "{}", val),
            UInt(val) => write!(f, "{}", val),
            Float(val) => write!(f, "{}", val),
            Bool(val) => write!(f, "{}", val),
            String(val) => write!(f, "\"{}\"", val),
            Bytes(val) => write!(f, "{:?}", val),
            List(val) => write!(f, "{:?}", val),
            Map(val) => write!(f, "{:?}", val),
            Null => write!(f, "NULL"),
            Ident(val) => write!(f, "{}", val),
            Type(val) => write!(f, "{}", val),
            TimeStamp(val) => write!(f, "{}", val),
            Duration(val) => write!(f, "{}", val),
            ByteCode(val) => write!(f, "{:?}", val),
        }
    }
}

#[cfg(test)]
mod test {
    use super::CelValue;

    #[test]
    fn test_add() {
        let res = CelValue::from(4i64) + CelValue::from(5i64);

        assert!(res.is_ok());
        let val: i64 = res.ok().unwrap().try_into().unwrap();
        assert!(val == 9);
    }

    #[test]
    fn test_bad_op() {
        let res = CelValue::from(3i64) + CelValue::from(4.2);

        assert!(res.is_err());
    }
}
