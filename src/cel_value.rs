use chrono::{offset::Utc, DateTime, Duration, FixedOffset};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt,
    iter::zip,
    ops::{Add, Div, Mul, Neg, Not, Rem, Sub},
    sync::Arc,
};

use serde_json::{value::Value, Map};

use crate::{context::RsCallable, interp::ByteCode, CelError, CelResult};

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CelValueInner {
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
    #[serde(skip_serializing, skip_deserializing)]
    BoundCall {
        callable: RsCallable,
        value: CelValue,
    },
}

#[derive(Serialize, Deserialize)]
pub struct CelValue {
    inner: Arc<CelValueInner>,
}

impl From<CelValueInner> for CelValue {
    fn from(inner: CelValueInner) -> CelValue {
        return CelValue {
            inner: Arc::new(inner),
        };
    }
}

impl Clone for CelValue {
    fn clone(&self) -> CelValue {
        CelValue {
            inner: self.inner.clone(),
        }
    }
}

impl PartialEq for CelValue {
    fn eq(&self, rhs: &CelValue) -> bool {
        self.inner().eq(rhs.inner())
    }
}

impl AsRef<CelValueInner> for CelValue {
    fn as_ref(&self) -> &CelValueInner {
        self.inner()
    }
}

impl CelValue {
    pub fn from_int(val: i64) -> CelValue {
        CelValueInner::Int(val).into()
    }

    pub fn from_uint(val: u64) -> CelValue {
        CelValueInner::UInt(val).into()
    }

    pub fn from_float(val: f64) -> CelValue {
        CelValueInner::Float(val).into()
    }

    pub fn from_bool(val: bool) -> CelValue {
        CelValueInner::Bool(val).into()
    }

    pub fn from_string(val: String) -> CelValue {
        CelValueInner::String(val).into()
    }

    pub fn from_bytes(val: Vec<u8>) -> CelValue {
        CelValueInner::Bytes(val).into()
    }

    pub fn from_list(val: Vec<CelValue>) -> CelValue {
        CelValueInner::List(val.to_vec()).into()
    }

    pub fn from_map(val: HashMap<String, CelValue>) -> CelValue {
        CelValueInner::Map(val.clone()).into()
    }

    pub fn from_null() -> CelValue {
        CelValueInner::Null.into()
    }

    pub fn from_ident(val: &str) -> CelValue {
        CelValueInner::Ident(val.to_owned()).into()
    }

    pub fn from_type(val: &str) -> CelValue {
        CelValueInner::Type(val.to_owned()).into()
    }

    pub fn from_timestamp(val: &DateTime<Utc>) -> CelValue {
        CelValueInner::TimeStamp(val.clone()).into()
    }

    pub fn from_duration(val: &Duration) -> CelValue {
        CelValueInner::Duration(val.clone()).into()
    }

    pub(crate) fn from_bytecode(val: &[ByteCode]) -> CelValue {
        CelValueInner::ByteCode(val.to_owned()).into()
    }

    pub(crate) fn from_binding(callable: RsCallable, value: &CelValue) -> CelValue {
        CelValueInner::BoundCall {
            callable,
            value: value.clone(),
        }
        .into()
    }

    pub fn into_inner(self) -> CelValueInner {
        match Arc::try_unwrap(self.inner) {
            Ok(inner) => inner,
            Err(rc) => (*rc).clone(),
        }
    }

    pub fn inner<'l>(&'l self) -> &'l CelValueInner {
        &self.inner
    }

    pub fn eq(&self, rhs: &CelValue) -> CelResult<CelValue> {
        let type1 = rhs.as_type();
        let type2 = rhs.as_type();

        match self.inner() {
            CelValueInner::Int(val1) => {
                match rhs.inner() {
                    CelValueInner::Int(val2) => return Ok(CelValue::from_bool(val1 == val2)),
                    CelValueInner::UInt(val2) => {
                        return Ok(CelValue::from_bool(*val1 == *val2 as i64))
                    }
                    CelValueInner::Float(val2) => {
                        return Ok(CelValue::from_bool(*val1 as f64 == *val2))
                    }
                    _ => {}
                };
            }
            CelValueInner::UInt(val1) => {
                match rhs.inner() {
                    CelValueInner::Int(val2) => {
                        return Ok(CelValue::from_bool(*val1 as i64 == *val2))
                    }
                    CelValueInner::UInt(val2) => return Ok(CelValue::from_bool(val1 == val2)),
                    CelValueInner::Float(val2) => {
                        return Ok(CelValue::from_bool(*val1 as f64 == *val2))
                    }
                    _ => {}
                };
            }
            CelValueInner::Float(val1) => {
                match rhs.inner() {
                    CelValueInner::Int(val2) => {
                        return Ok(CelValue::from_bool(*val1 == *val2 as f64))
                    }
                    CelValueInner::UInt(val2) => {
                        return Ok(CelValue::from_bool(*val1 == *val2 as f64))
                    }
                    CelValueInner::Float(val2) => return Ok(CelValue::from_bool(val1 == val2)),
                    _ => {}
                };
            }
            CelValueInner::Bool(val1) => {
                if let CelValueInner::Bool(val2) = rhs.inner() {
                    return Ok(CelValue::from_bool(val1 == val2));
                }
            }
            CelValueInner::String(val1) => {
                if let CelValueInner::String(val2) = rhs.inner() {
                    return Ok(CelValue::from_bool(val1 == val2));
                }
            }
            CelValueInner::Bytes(val1) => {
                if let CelValueInner::Bytes(val2) = rhs.inner() {
                    return Ok(CelValue::from_bool(val1 == val2));
                }
            }
            CelValueInner::List(val1) => {
                if let CelValueInner::List(val2) = rhs.inner() {
                    if val1.len() != val2.len() {
                        return Ok(CelValue::from_bool(false));
                    }

                    for (v1, v2) in zip(val1, val2) {
                        match v1.eq(v2) {
                            Ok(res_cell) => {
                                if let CelValueInner::Bool(res) = res_cell.inner() {
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
            CelValueInner::Null => {
                if let CelValueInner::Null = rhs.inner() {
                    return Ok(CelValue::from_bool(true));
                } else {
                    return Ok(CelValue::from_bool(false));
                }
            }
            CelValueInner::TimeStamp(v1) => {
                if let CelValueInner::TimeStamp(v2) = rhs.inner() {
                    return Ok(CelValue::from_bool(*v1 == *v2));
                }
            }
            CelValueInner::Duration(v1) => {
                if let CelValueInner::Duration(v2) = rhs.inner() {
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
        if let CelValueInner::Bool(res) = self.eq(rhs)?.inner() {
            return Ok(CelValue::from_bool(!res));
        }

        unreachable!();
    }

    fn ord(&self, rhs: &CelValue) -> CelResult<Option<Ordering>> {
        let type1 = rhs.as_type();
        let type2 = rhs.as_type();

        match self.inner() {
            CelValueInner::Int(v1) => match rhs.inner() {
                CelValueInner::Int(v2) => return Ok(Some(v1.cmp(v2))),
                CelValueInner::UInt(v2) => return Ok(Some(v1.cmp(&(*v2 as i64)))),
                CelValueInner::Float(v2) => return Ok((*v1 as f64).partial_cmp(v2)),
                _ => {}
            },
            CelValueInner::UInt(v1) => match rhs.inner() {
                CelValueInner::Int(v2) => return Ok(Some((*v1 as i64).cmp(v2))),
                CelValueInner::UInt(v2) => return Ok(Some(v1.cmp(v2))),
                CelValueInner::Float(v2) => return Ok((*v1 as f64).partial_cmp(v2)),
                _ => {}
            },
            CelValueInner::Float(v1) => match rhs.inner() {
                CelValueInner::Int(v2) => return Ok(v1.partial_cmp(&(*v2 as f64))),
                CelValueInner::UInt(v2) => return Ok(v1.partial_cmp(&(*v2 as f64))),
                CelValueInner::Float(v2) => return Ok(v1.partial_cmp(v2)),
                _ => {}
            },
            CelValueInner::TimeStamp(v1) => {
                if let CelValueInner::TimeStamp(v2) = rhs.inner() {
                    return Ok(v1.partial_cmp(v2));
                }
            }
            CelValueInner::Duration(v1) => {
                if let CelValueInner::Duration(v2) = rhs.inner() {
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
        if let CelValueInner::Bool(lhs) = self.inner() {
            if let CelValueInner::Bool(rhs) = rhs.inner() {
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
        if let CelValueInner::Bool(lhs) = self.inner() {
            if let CelValueInner::Bool(rhs) = rhs.inner() {
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
        match self.into_inner() {
            CelValueInner::Int(val) => Value::from(val),
            CelValueInner::UInt(val) => Value::from(val),
            CelValueInner::Float(val) => Value::from(val),
            CelValueInner::Bool(val) => Value::from(val),
            CelValueInner::String(val) => Value::from(val),
            CelValueInner::Bytes(val) => Value::from(val),
            CelValueInner::List(val) => {
                let mut partial: Vec<Value> = Vec::new();

                for v in val.into_iter() {
                    partial.push(v.into_json_value());
                }

                Value::Array(partial)
            }
            CelValueInner::Map(val) => {
                let mut partial: Map<String, Value> = Map::new();

                for (key, value) in val.into_iter() {
                    partial.insert(key, value.into_json_value());
                }

                Value::Object(partial)
            }
            CelValueInner::TimeStamp(val) => Value::from(val.to_rfc3339()),
            CelValueInner::Duration(val) => Value::from(val.to_string()),
            _ => Value::Null,
        }
    }

    pub fn as_type(&self) -> CelValue {
        match self.inner() {
            CelValueInner::Int(_) => CelValue::from_type("int"),
            CelValueInner::UInt(_) => CelValue::from_type("uint"),
            CelValueInner::Float(_) => CelValue::from_type("float"),
            CelValueInner::Bool(_) => CelValue::from_type("bool"),
            CelValueInner::String(_) => CelValue::from_type("string"),
            CelValueInner::Bytes(_) => CelValue::from_type("bytes"),
            CelValueInner::List(_) => CelValue::from_type("list"),
            CelValueInner::Map(_) => CelValue::from_type("map"),
            CelValueInner::Null => CelValue::from_type("null_type"),
            CelValueInner::Ident(_) => CelValue::from_type("ident"),
            CelValueInner::Type(_) => CelValue::from_type("type"),
            CelValueInner::TimeStamp(_) => CelValue::from_type("timestamp"),
            CelValueInner::Duration(_) => CelValue::from_type("duration"),
            CelValueInner::ByteCode(_) => CelValue::from_type("bytecode"),
            CelValueInner::BoundCall {
                callable: _,
                value: _,
            } => CelValue::from_type("bound call"),
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
        if let CelValueInner::Int(val) = self.inner() {
            return Ok(*val);
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
        if let CelValueInner::UInt(val) = self.inner() {
            return Ok(*val);
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
        if let CelValueInner::Float(val) = self.inner() {
            return Ok(*val);
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
        if let CelValueInner::Bool(val) = self.inner() {
            return Ok(*val);
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
        if let CelValueInner::String(val) = self.into_inner() {
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
        CelValueInner::Bytes(val).into()
    }
}

impl TryInto<Vec<u8>> for CelValue {
    type Error = CelError;

    fn try_into(self) -> CelResult<Vec<u8>> {
        if let CelValueInner::Bytes(val) = self.into_inner() {
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
        CelValueInner::List(val).into()
    }
}

impl TryInto<Vec<CelValue>> for CelValue {
    type Error = CelError;

    fn try_into(self) -> CelResult<Vec<CelValue>> {
        if let CelValueInner::List(val) = self.into_inner() {
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
        if let CelValueInner::Map(val) = self.into_inner() {
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
        if let CelValueInner::TimeStamp(val) = self.into_inner() {
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
        if let CelValueInner::Duration(val) = self.into_inner() {
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
        if let CelValueInner::ByteCode(val) = self.into_inner() {
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

        match self.into_inner() {
            CelValueInner::Int(val1) => {
                if let CelValueInner::Int(val2) = rhs.inner() {
                    return Ok(CelValue::from(val1 + val2));
                }
            }
            CelValueInner::UInt(val1) => {
                if let CelValueInner::UInt(val2) = rhs.inner() {
                    return Ok(CelValue::from(val1 + val2));
                }
            }
            CelValueInner::Float(val1) => {
                if let CelValueInner::Float(val2) = rhs.inner() {
                    return Ok(CelValue::from(val1 + val2));
                }
            }
            CelValueInner::String(val1) => {
                if let CelValueInner::String(val2) = rhs.into_inner() {
                    let mut res = val1;
                    res.push_str(&val2);
                    return Ok(CelValue::from(res));
                }
            }
            CelValueInner::Bytes(val1) => {
                if let CelValueInner::Bytes(val2) = rhs.inner() {
                    let mut res = val1;
                    res.extend_from_slice(&val2);
                    return Ok(CelValue::from(res));
                }
            }
            CelValueInner::List(val1) => {
                if let CelValueInner::List(val2) = rhs.inner() {
                    let mut res = val1;
                    res.extend_from_slice(&val2);
                    return Ok(CelValue::from(res));
                }
            }
            CelValueInner::TimeStamp(v1) => {
                if let CelValueInner::Duration(v2) = rhs.inner() {
                    return Ok(CelValue::from_timestamp(&(v1 + *v2)));
                }
            }
            CelValueInner::Duration(v1) => {
                if let CelValueInner::TimeStamp(v2) = rhs.inner() {
                    return Ok(CelValue::from_timestamp(&(*v2 + v1)));
                } else if let CelValueInner::Duration(v2) = rhs.inner() {
                    return Ok(CelValue::from_duration(&(v1 + *v2)));
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

        match self.inner() {
            CelValueInner::Int(val1) => {
                if let CelValueInner::Int(val2) = rhs.inner() {
                    return Ok(CelValue::from(val1 - val2));
                }
            }
            CelValueInner::UInt(val1) => {
                if let CelValueInner::UInt(val2) = rhs.inner() {
                    return Ok(CelValue::from(val1 - val2));
                }
            }
            CelValueInner::Float(val1) => {
                if let CelValueInner::Float(val2) = rhs.inner() {
                    return Ok(CelValue::from(val1 - val2));
                }
            }
            CelValueInner::TimeStamp(v1) => {
                if let CelValueInner::Duration(v2) = rhs.inner() {
                    return Ok(CelValue::from_timestamp(&(*v1 - *v2)));
                }
            }
            CelValueInner::Duration(v1) => {
                if let CelValueInner::TimeStamp(v2) = rhs.inner() {
                    return Ok(CelValue::from_timestamp(&(*v2 - *v1)));
                } else if let CelValueInner::Duration(v2) = rhs.inner() {
                    return Ok(CelValue::from_duration(&(*v1 - *v2)));
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

        match self.inner() {
            CelValueInner::Int(val1) => {
                if let CelValueInner::Int(val2) = rhs.inner() {
                    return Ok(CelValue::from(val1 * val2));
                }
            }
            CelValueInner::UInt(val1) => {
                if let CelValueInner::UInt(val2) = rhs.inner() {
                    return Ok(CelValue::from(val1 * val2));
                }
            }
            CelValueInner::Float(val1) => {
                if let CelValueInner::Float(val2) = rhs.inner() {
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

        match self.inner() {
            CelValueInner::Int(val1) => {
                if let CelValueInner::Int(val2) = rhs.inner() {
                    return Ok(CelValue::from(val1 / val2));
                }
            }
            CelValueInner::UInt(val1) => {
                if let CelValueInner::UInt(val2) = rhs.inner() {
                    return Ok(CelValue::from(val1 / val2));
                }
            }
            CelValueInner::Float(val1) => {
                if let CelValueInner::Float(val2) = rhs.inner() {
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

        match self.inner() {
            CelValueInner::Int(val1) => {
                if let CelValueInner::Int(val2) = rhs.inner() {
                    return Ok(CelValue::from(val1 % val2));
                }
            }
            CelValueInner::UInt(val1) => {
                if let CelValueInner::UInt(val2) = rhs.inner() {
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

        match self.inner() {
            CelValueInner::Int(val1) => {
                return Ok(CelValue::from(-val1));
            }
            CelValueInner::Float(val1) => {
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

        match self.inner() {
            CelValueInner::Bool(val1) => {
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

impl fmt::Debug for CelValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.inner())
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
