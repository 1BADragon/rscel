use chrono::{offset::Utc, DateTime, Duration, FixedOffset};
use protobuf::{
    reflect::{EnumDescriptor, MessageDescriptor, ReflectValueRef},
    MessageDyn,
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::{
    borrow::{Cow, ToOwned},
    cmp::Ordering,
    collections::HashMap,
    fmt,
    iter::zip,
    ops::{Add, Div, Mul, Neg, Not, Rem, Sub},
};

use serde_json::value::Value;

use crate::{interp::ByteCode, CelError, CelResult};

/// The basic value of the CEL interpreter.
///
/// Houses all possible types and implements most of the valid operations within the
/// interpreter
#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
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
    #[serde(skip_serializing, skip_deserializing)]
    Message(Box<dyn MessageDyn>),
    #[serde(skip_serializing, skip_deserializing)]
    Enum {
        descriptor: EnumDescriptor,
        value: i32,
    },
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

    pub fn true_() -> CelValue {
        CelValue::Bool(true)
    }

    pub fn false_() -> CelValue {
        CelValue::Bool(false)
    }

    pub fn from_string(val: String) -> CelValue {
        CelValue::String(val)
    }

    pub fn from_str(val: &str) -> CelValue {
        CelValue::String(val.to_owned())
    }

    pub fn from_bytes(val: Vec<u8>) -> CelValue {
        CelValue::Bytes(val)
    }

    pub fn from_byte_slice(val: &[u8]) -> CelValue {
        CelValue::Bytes(val.to_owned())
    }

    pub fn from_list(val: Vec<CelValue>) -> CelValue {
        CelValue::List(val)
    }

    pub fn from_val_slice(val: &[CelValue]) -> CelValue {
        CelValue::List(val.to_owned())
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

    pub fn from_proto_msg(val: Box<dyn MessageDyn>) -> CelValue {
        CelValue::Message(val)
    }

    pub fn from_proto_enum(descriptor: EnumDescriptor, value: i32) -> CelValue {
        CelValue::Enum { descriptor, value }
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

    pub fn int_type() -> CelValue {
        CelValue::from_type("int")
    }

    pub fn uint_type() -> CelValue {
        CelValue::from_type("uint")
    }

    pub fn float_type() -> CelValue {
        CelValue::from_type("float")
    }

    pub fn bool_type() -> CelValue {
        CelValue::from_type("bool")
    }

    pub fn string_type() -> CelValue {
        CelValue::from_type("string")
    }

    pub fn bytes_type() -> CelValue {
        CelValue::from_type("bytes")
    }

    pub fn list_type() -> CelValue {
        CelValue::from_type("list")
    }

    pub fn map_type() -> CelValue {
        CelValue::from_type("map")
    }

    pub fn null_type() -> CelValue {
        CelValue::from_type("null")
    }

    pub fn ident_type() -> CelValue {
        CelValue::from_type("ident")
    }

    pub fn type_type() -> CelValue {
        CelValue::from_type("type")
    }

    pub fn timestamp_type() -> CelValue {
        CelValue::from_type("timestamp")
    }

    pub fn duration_type() -> CelValue {
        CelValue::from_type("duration")
    }

    pub fn bytecode_type() -> CelValue {
        CelValue::from_type("bytecode")
    }

    pub fn message_type(desc: &MessageDescriptor) -> CelValue {
        CelValue::Type(format!("message-{}", desc.full_name()))
    }

    pub fn enum_type(desc: &EnumDescriptor) -> CelValue {
        CelValue::Type(format!("enum-{}", desc.full_name()))
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            CelValue::Int(i) => *i != 0,
            CelValue::UInt(u) => *u != 0,
            CelValue::Float(f) => *f != 0.0,
            CelValue::Bool(b) => *b,
            CelValue::String(s) => s.len() != 0,
            CelValue::Bytes(b) => b.len() != 0,
            CelValue::List(l) => l.len() != 0,
            CelValue::Map(m) => m.len() != 0,
            CelValue::Null => false,
            CelValue::Type(_) => true,
            CelValue::TimeStamp(_) => true,
            CelValue::Duration(_) => true,
            CelValue::Enum {
                descriptor: _,
                value,
            } => *value != 0,
            CelValue::Message(_) => true,
            _ => false,
        }
    }

    pub fn is_null(&self) -> bool {
        if let CelValue::Null = self {
            true
        } else {
            false
        }
    }

    fn type_prop<'a>(
        lhs: Cow<'a, CelValue>,
        rhs: Cow<'a, CelValue>,
    ) -> (Cow<'a, CelValue>, Cow<'a, CelValue>) {
        if let CelValue::Int(l) = lhs.as_ref() {
            match rhs.as_ref() {
                CelValue::Int(_) => (lhs, rhs),
                CelValue::UInt(u) => (lhs, Cow::Owned((*u as i64).into())),
                CelValue::Float(_) => (Cow::Owned((*l as f64).into()), rhs),
                CelValue::Bool(b) => (lhs, Cow::Owned((*b as i64).into())),
                _ => (lhs, rhs),
            }
        } else if let CelValue::UInt(l) = lhs.as_ref() {
            match rhs.as_ref() {
                CelValue::Int(_) => (Cow::Owned((*l as i64).into()), rhs),
                CelValue::UInt(_) => (lhs, rhs),
                CelValue::Float(_) => (Cow::Owned((*l as f64).into()), rhs),
                CelValue::Bool(b) => (lhs, Cow::Owned((*b as u64).into())),
                _ => (lhs, rhs),
            }
        } else if let CelValue::Float(_) = lhs.as_ref() {
            match rhs.as_ref() {
                CelValue::Int(i) => (lhs, Cow::Owned((*i as f64).into())),
                CelValue::UInt(u) => (lhs, Cow::Owned((*u as f64).into())),
                CelValue::Float(_) => (lhs, rhs),
                CelValue::Bool(b) => (lhs, Cow::Owned((if *b { 1.0 } else { 0.0 }).into())),
                _ => (lhs, rhs),
            }
        } else if let CelValue::Bool(l) = lhs.as_ref() {
            match rhs.as_ref() {
                CelValue::Int(_) => (Cow::Owned((*l as i64).into()), rhs),
                CelValue::UInt(_) => (Cow::Owned((*l as u64).into()), rhs),
                CelValue::Float(_) => (Cow::Owned((if *l { 1.0 } else { 0.0 }).into()), rhs),
                CelValue::Bool(_) => (lhs, rhs),
                _ => (lhs, rhs),
            }
        } else {
            (lhs, rhs)
        }
    }

    pub fn eq(&self, rhs_val: &CelValue) -> CelResult<CelValue> {
        let type1 = self.as_type();
        let type2 = rhs_val.as_type();

        let (lhs, rhs) = CelValue::type_prop(Cow::Borrowed(self), Cow::Borrowed(rhs_val));

        if let (CelValue::Int(l), CelValue::Int(r)) = (lhs.as_ref(), rhs.as_ref()) {
            Ok(CelValue::from_bool(l == r))
        } else if let (CelValue::UInt(l), CelValue::UInt(r)) = (lhs.as_ref(), rhs.as_ref()) {
            Ok(CelValue::from_bool(l == r))
        } else if let (CelValue::Float(l), CelValue::Float(r)) = (lhs.as_ref(), rhs.as_ref()) {
            Ok(CelValue::from_bool(l == r))
        } else if let (CelValue::Bool(l), CelValue::Bool(r)) = (lhs.as_ref(), rhs.as_ref()) {
            Ok(CelValue::from_bool(l == r))
        } else if let (CelValue::String(l), CelValue::String(r)) = (lhs.as_ref(), rhs.as_ref()) {
            Ok(CelValue::from_bool(l == r))
        } else if let (CelValue::Bytes(l), CelValue::Bytes(r)) = (lhs.as_ref(), rhs.as_ref()) {
            Ok(CelValue::from_bool(l == r))
        } else if let (CelValue::List(l), CelValue::List(r)) = (lhs.as_ref(), rhs.as_ref()) {
            if l.len() != r.len() {
                Ok(false.into())
            } else {
                for (v1, v2) in zip(l, r) {
                    match v1.eq(&v2) {
                        Ok(res_cell) => {
                            if let CelValue::Bool(res) = res_cell {
                                if !res {
                                    return Ok(CelValue::false_());
                                }
                            }
                        }
                        Err(_) => return Ok(CelValue::false_()),
                    }
                }
                Ok(CelValue::true_())
            }
        } else if let CelValue::Null = lhs.as_ref() {
            if let CelValue::Null = rhs.as_ref() {
                Ok(CelValue::true_())
            } else {
                Ok(CelValue::false_())
            }
        } else if let (CelValue::TimeStamp(l), CelValue::TimeStamp(r)) =
            (lhs.as_ref(), rhs.as_ref())
        {
            Ok(CelValue::from_bool(l == r))
        } else if let (CelValue::Duration(l), CelValue::Duration(r)) = (lhs.as_ref(), rhs.as_ref())
        {
            Ok(CelValue::from_bool(l == r))
        } else if let (CelValue::Type(l), CelValue::Type(r)) = (lhs.as_ref(), rhs.as_ref()) {
            Ok(CelValue::from_bool(l == r))
        } else if let (CelValue::Message(l), CelValue::Message(r)) = (lhs.as_ref(), rhs.as_ref()) {
            Ok(CelValue::from_bool(
                l.descriptor_dyn().eq(l.as_ref(), r.as_ref()),
            ))
        } else if let CelValue::Enum {
            descriptor: l_desc,
            value: l_value,
        } = lhs.as_ref()
        {
            if let CelValue::Enum {
                descriptor: r_desc,
                value: r_value,
            } = rhs.as_ref()
            {
                Ok(CelValue::from_bool(l_value == r_value && l_desc == r_desc))
            } else if let CelValue::Int(intval) = rhs.as_ref() {
                Ok(CelValue::from_bool(*intval == (*l_value as i64)))
            } else if let CelValue::UInt(intval) = rhs.as_ref() {
                Ok(CelValue::from_bool(*intval == (*l_value as u64)))
            } else if let CelValue::String(strval) = rhs.as_ref() {
                if let Some(_) = l_desc.value_by_name(strval) {
                    Ok(CelValue::true_())
                } else {
                    Ok(CelValue::false_())
                }
            } else {
                Err(CelError::invalid_op(&format!(
                    "Invalid op '==' between {:?} and {:?}",
                    type1, type2
                )))
            }
        } else {
            Err(CelError::invalid_op(&format!(
                "Invalid op '==' between {:?} and {:?}",
                type1, type2
            )))
        }
    }

    pub fn neq(&self, rhs: &CelValue) -> CelResult<CelValue> {
        if let CelValue::Bool(res) = self.eq(rhs)? {
            return Ok(CelValue::from_bool(!res));
        }

        unreachable!();
    }

    fn ord(&self, rhs_value: &CelValue) -> CelResult<Option<Ordering>> {
        let type1 = self.as_type();
        let type2 = rhs_value.as_type();

        let (lhs, rhs) = CelValue::type_prop(Cow::Borrowed(self), Cow::Borrowed(rhs_value));

        if let (CelValue::Int(l), CelValue::Int(r)) = (lhs.as_ref(), rhs.as_ref()) {
            Ok(l.partial_cmp(r))
        } else if let (CelValue::UInt(l), CelValue::UInt(r)) = (lhs.as_ref(), rhs.as_ref()) {
            Ok(l.partial_cmp(r))
        } else if let (CelValue::Float(l), CelValue::Float(r)) = (lhs.as_ref(), rhs.as_ref()) {
            Ok(l.partial_cmp(r))
        } else if let (CelValue::TimeStamp(l), CelValue::TimeStamp(r)) =
            (lhs.as_ref(), rhs.as_ref())
        {
            Ok(l.partial_cmp(r))
        } else if let (CelValue::Duration(l), CelValue::Duration(r)) = (lhs.as_ref(), rhs.as_ref())
        {
            Ok(l.partial_cmp(r))
        } else {
            Err(CelError::invalid_op(&format!(
                "Invalid op 'ord' between {:?} and {:?}",
                type1, type2
            )))
        }
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
        if cfg!(feature = "type_prop") {
            return Ok((self.is_truthy() || rhs.is_truthy()).into());
        }

        if let CelValue::Bool(lhs) = self {
            if let CelValue::Bool(rhs) = rhs {
                return Ok((*lhs || *rhs).into());
            }
        }

        Err(CelError::invalid_op(&format!(
            "|| operator invalid for {:?} and {:?}",
            self.as_type(),
            rhs.as_type(),
        )))
    }

    pub fn in_(&self, rhs: &CelValue) -> CelResult<CelValue> {
        let rhs_type = rhs.as_type();
        let lhs_type = self.as_type();

        match rhs {
            CelValue::List(l) => {
                for value in l.iter() {
                    if *self == *value {
                        return Ok(true.into());
                    }
                }

                Ok(false.into())
            }
            CelValue::Map(m) => {
                if let CelValue::String(r) = self {
                    Ok(CelValue::from_bool(m.contains_key(r)))
                } else {
                    Err(CelError::invalid_op(&format!(
                        "Op 'in' invalid between {:?} and {:?}",
                        lhs_type, rhs_type
                    )))
                }
            }
            CelValue::String(s) => {
                if let CelValue::String(r) = self {
                    Ok(CelValue::from_bool(s.contains(r)))
                } else {
                    Err(CelError::invalid_op(&format!(
                        "Op 'in' invalid between {:?} and {:?}",
                        lhs_type, rhs_type
                    )))
                }
            }
            _ => Err(CelError::invalid_op(&format!(
                "Op 'in' invalid between {:?} and {:?}",
                lhs_type, rhs_type
            ))),
        }
    }

    pub fn and(&self, rhs: &CelValue) -> CelResult<CelValue> {
        if cfg!(feature = "type_prop") {
            return Ok((self.is_truthy() && rhs.is_truthy()).into());
        }

        if let CelValue::Bool(lhs) = self {
            if let CelValue::Bool(rhs) = rhs {
                return Ok((*lhs && *rhs).into());
            }
        }

        Err(CelError::invalid_op(&format!(
            "&& operator invalid for {:?} and {:?}",
            self.as_type(),
            rhs.as_type(),
        )))
    }

    pub fn as_type(&self) -> CelValue {
        match self {
            CelValue::Int(_) => CelValue::int_type(),
            CelValue::UInt(_) => CelValue::uint_type(),
            CelValue::Float(_) => CelValue::float_type(),
            CelValue::Bool(_) => CelValue::bool_type(),
            CelValue::String(_) => CelValue::string_type(),
            CelValue::Bytes(_) => CelValue::bytes_type(),
            CelValue::List(_) => CelValue::list_type(),
            CelValue::Map(_) => CelValue::map_type(),
            CelValue::Null => CelValue::null_type(),
            CelValue::Ident(_) => CelValue::ident_type(),
            CelValue::Type(_) => CelValue::type_type(),
            CelValue::TimeStamp(_) => CelValue::timestamp_type(),
            CelValue::Duration(_) => CelValue::duration_type(),
            CelValue::ByteCode(_) => CelValue::bytecode_type(),
            CelValue::Message(msg) => CelValue::message_type(&msg.descriptor_dyn()),
            CelValue::Enum {
                descriptor,
                value: _value,
            } => CelValue::enum_type(&descriptor),
        }
    }
}

impl From<&Value> for CelValue {
    fn from(value: &Value) -> CelValue {
        match value {
            Value::Number(val) => {
                if let Some(val) = val.as_i64() {
                    return CelValue::from_int(val);
                }

                if let Some(val) = val.as_u64() {
                    return CelValue::from_uint(val);
                }

                if let Some(val) = val.as_f64() {
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
                }

                if let Some(val) = val.as_u64() {
                    return CelValue::from_uint(val);
                }

                if let Some(val) = val.as_f64() {
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

impl<'a> From<ReflectValueRef<'a>> for CelValue {
    fn from(value: ReflectValueRef) -> Self {
        match value {
            ReflectValueRef::U32(u) => CelValue::UInt(u as u64),
            ReflectValueRef::U64(u) => CelValue::UInt(u),
            ReflectValueRef::I32(i) => CelValue::Int(i as i64),
            ReflectValueRef::I64(i) => CelValue::Int(i),
            ReflectValueRef::F32(f) => CelValue::Float(f as f64),
            ReflectValueRef::F64(f) => CelValue::Float(f),
            ReflectValueRef::Bool(b) => CelValue::Bool(b),
            ReflectValueRef::String(s) => CelValue::String(s.to_string()),
            ReflectValueRef::Bytes(b) => CelValue::Bytes(b.to_owned()),
            ReflectValueRef::Enum(desc, value) => CelValue::Enum {
                descriptor: desc,
                value,
            },
            ReflectValueRef::Message(msg) => CelValue::Message(msg.clone_box()),
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

        Err(CelError::internal("Convertion Error"))
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

        Err(CelError::internal("Convertion Error"))
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

        Err(CelError::internal("Convertion Error"))
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

        Err(CelError::internal("Convertion Error"))
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

        Err(CelError::internal("Convertion Error"))
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

        Err(CelError::internal("Convertion Error"))
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

        Err(CelError::internal("Convertion Error"))
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

        Err(CelError::internal("Convertion Error"))
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

        Err(CelError::internal("Convertion Error"))
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

        Err(CelError::internal("Convertion Error"))
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

        Err(CelError::internal("Convertion Error"))
    }
}

impl PartialEq for CelValue {
    fn eq(&self, other: &Self) -> bool {
        if let (CelValue::Int(lhs), CelValue::Int(rhs)) = (self, other) {
            lhs == rhs
        } else if let (CelValue::UInt(lhs), CelValue::UInt(rhs)) = (self, other) {
            lhs == rhs
        } else if let (CelValue::Float(lhs), CelValue::Float(rhs)) = (self, other) {
            lhs == rhs
        } else if let (CelValue::Bool(lhs), CelValue::Bool(rhs)) = (self, other) {
            lhs == rhs
        } else if let (CelValue::String(lhs), CelValue::String(rhs)) = (self, other) {
            lhs == rhs
        } else if let (CelValue::Bytes(lhs), CelValue::Bytes(rhs)) = (self, other) {
            lhs == rhs
        } else if let (CelValue::List(lhs), CelValue::List(rhs)) = (self, other) {
            lhs == rhs
        } else if let (CelValue::Map(lhs), CelValue::Map(rhs)) = (self, other) {
            lhs == rhs
        } else if let (CelValue::Null, CelValue::Null) = (self, other) {
            true
        } else if let (CelValue::Ident(lhs), CelValue::Ident(rhs)) = (self, other) {
            lhs == rhs
        } else if let (CelValue::Type(lhs), CelValue::Type(rhs)) = (self, other) {
            lhs == rhs
        } else if let (CelValue::TimeStamp(lhs), CelValue::TimeStamp(rhs)) = (self, other) {
            lhs == rhs
        } else if let (CelValue::Duration(lhs), CelValue::Duration(rhs)) = (self, other) {
            lhs == rhs
        } else if let (CelValue::ByteCode(lhs), CelValue::ByteCode(rhs)) = (self, other) {
            lhs == rhs
        } else if let (CelValue::Message(lhs), CelValue::Message(rhs)) = (self, other) {
            lhs.descriptor_dyn().eq(lhs.as_ref(), rhs.as_ref())
        } else if let (
            CelValue::Enum {
                descriptor: lhs_descriptor,
                value: lhs_value,
            },
            CelValue::Enum {
                descriptor: rhs_descriptor,
                value: rhs_value,
            },
        ) = (self, other)
        {
            lhs_value == rhs_value && lhs_descriptor == rhs_descriptor
        } else {
            false
        }
    }
}

impl Add for CelValue {
    type Output = CelResult<CelValue>;

    fn add(self, rhs_val: Self) -> Self::Output {
        let type1 = self.as_type();
        let type2 = rhs_val.as_type();

        let (lhs, rhs) = if cfg!(feature = "type_prop") {
            CelValue::type_prop(Cow::Owned(self), Cow::Owned(rhs_val))
        } else {
            (Cow::Owned(self), Cow::Owned(rhs_val))
        };

        match lhs.into_owned() {
            CelValue::Int(val1) => {
                if let CelValue::Int(val2) = rhs.into_owned() {
                    return Ok(CelValue::from(val1 + val2));
                }
            }
            CelValue::UInt(val1) => {
                if let CelValue::UInt(val2) = rhs.into_owned() {
                    return Ok(CelValue::from(val1 + val2));
                }
            }
            CelValue::Float(val1) => {
                if let CelValue::Float(val2) = rhs.into_owned() {
                    return Ok(CelValue::from(val1 + val2));
                }
            }
            CelValue::String(val1) => {
                if let CelValue::String(val2) = rhs.into_owned() {
                    let mut res = val1;
                    res.push_str(&val2);
                    return Ok(CelValue::from_str(res.as_ref()));
                }
            }
            CelValue::Bytes(val1) => {
                if let CelValue::Bytes(val2) = rhs.into_owned() {
                    let mut res = val1;
                    res.extend_from_slice(&val2);
                    return Ok(CelValue::from_byte_slice(res.as_ref()));
                }
            }
            CelValue::List(val1) => {
                if let CelValue::List(val2) = rhs.into_owned() {
                    let mut res = val1;
                    res.extend_from_slice(&val2);
                    return Ok(CelValue::from_val_slice(res.as_ref()));
                }
            }
            CelValue::TimeStamp(v1) => {
                if let CelValue::Duration(v2) = rhs.into_owned() {
                    return Ok(CelValue::from_timestamp(&(v1 + v2)));
                }
            }
            CelValue::Duration(v1) => match rhs.into_owned() {
                CelValue::TimeStamp(v2) => return Ok(CelValue::from_timestamp(&(v2 + v1))),
                CelValue::Duration(v2) => return Ok(CelValue::Duration(v1 + v2)),
                _ => {}
            },
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

    fn sub(self, rhs_val: Self) -> Self::Output {
        let type1 = self.as_type();
        let type2 = rhs_val.as_type();

        let (lhs, rhs) = if cfg!(feature = "type_prop") {
            CelValue::type_prop(Cow::Owned(self), Cow::Owned(rhs_val))
        } else {
            (Cow::Owned(self), Cow::Owned(rhs_val))
        };

        match lhs.into_owned() {
            CelValue::Int(val1) => {
                if let CelValue::Int(val2) = rhs.into_owned() {
                    return Ok(CelValue::from(val1 - val2));
                }
            }
            CelValue::UInt(val1) => {
                if let CelValue::UInt(val2) = rhs.into_owned() {
                    return Ok(CelValue::from(val1 - val2));
                }
            }
            CelValue::Float(val1) => {
                if let CelValue::Float(val2) = rhs.into_owned() {
                    return Ok(CelValue::from(val1 - val2));
                }
            }
            CelValue::TimeStamp(v1) => {
                if let CelValue::Duration(v2) = rhs.into_owned() {
                    return Ok(CelValue::from_timestamp(&(v1 - v2)));
                }
            }
            CelValue::Duration(v1) => match rhs.into_owned() {
                CelValue::TimeStamp(v2) => return Ok(CelValue::from_timestamp(&(v2 - v1))),
                CelValue::Duration(v2) => return Ok(CelValue::from_duration(&(v1 - v2))),
                _ => {}
            },
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

    fn mul(self, rhs_val: Self) -> Self::Output {
        let type1 = self.as_type();
        let type2 = rhs_val.as_type();

        let (lhs, rhs) = if cfg!(feature = "type_prop") {
            CelValue::type_prop(Cow::Owned(self), Cow::Owned(rhs_val))
        } else {
            (Cow::Owned(self), Cow::Owned(rhs_val))
        };

        match lhs.into_owned() {
            CelValue::Int(val1) => {
                if let CelValue::Int(val2) = rhs.into_owned() {
                    return Ok(CelValue::from(val1 * val2));
                }
            }
            CelValue::UInt(val1) => {
                if let CelValue::UInt(val2) = rhs.into_owned() {
                    return Ok(CelValue::from(val1 * val2));
                }
            }
            CelValue::Float(val1) => {
                if let CelValue::Float(val2) = rhs.into_owned() {
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

    fn div(self, rhs_val: Self) -> Self::Output {
        let type1 = self.as_type();
        let type2 = rhs_val.as_type();

        let (lhs, rhs) = if cfg!(feature = "type_prop") {
            CelValue::type_prop(Cow::Owned(self), Cow::Owned(rhs_val))
        } else {
            (Cow::Owned(self), Cow::Owned(rhs_val))
        };

        match lhs.into_owned() {
            CelValue::Int(val1) => {
                if let CelValue::Int(val2) = rhs.into_owned() {
                    return Ok(CelValue::from(val1 / val2));
                }
            }
            CelValue::UInt(val1) => {
                if let CelValue::UInt(val2) = rhs.into_owned() {
                    return Ok(CelValue::from(val1 / val2));
                }
            }
            CelValue::Float(val1) => {
                if let CelValue::Float(val2) = rhs.into_owned() {
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

    fn rem(self, rhs_val: Self) -> Self::Output {
        let type1 = self.as_type();
        let type2 = rhs_val.as_type();

        let (lhs, rhs) = if cfg!(feature = "type_prop") {
            CelValue::type_prop(Cow::Owned(self), Cow::Owned(rhs_val))
        } else {
            (Cow::Owned(self), Cow::Owned(rhs_val))
        };

        match lhs.into_owned() {
            CelValue::Int(val1) => {
                if let CelValue::Int(val2) = rhs.into_owned() {
                    return Ok(CelValue::from(val1 % val2));
                }
            }
            CelValue::UInt(val1) => {
                if let CelValue::UInt(val2) = rhs.into_owned() {
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

    #[cfg(not(feature = "type_prop"))]
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

    #[cfg(feature = "type_prop")]
    fn not(self) -> Self::Output {
        Ok((!self.is_truthy()).into())
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
            Message(msg) => write!(f, "{}", msg.as_ref()),
            Enum { descriptor, value } => {
                if let Some(v) = descriptor.value_by_number(*value) {
                    write!(f, "{}::{} ({})", descriptor.full_name(), v.name(), value)
                } else {
                    write!(f, "{}::({})", descriptor.full_name(), value)
                }
            }
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

    #[cfg(not(feature = "type_prop"))]
    #[test]
    fn test_bad_op() {
        let res = CelValue::from(3i64) + CelValue::from(4.2);

        assert!(res.is_err());
    }
}
