use chrono::{offset::Utc, serde::ts_milliseconds, DateTime, Duration, FixedOffset};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DeserializeAs, DurationMilliSeconds, SerializeAs};
use std::{
    any::Any,
    cmp::Ordering,
    collections::HashMap,
    fmt,
    iter::zip,
    ops::{Add, Div, Mul, Neg, Not, Rem, Sub},
    sync::Arc,
};

use serde_json::value::Value;

#[cfg(feature = "protobuf")]
use protobuf::{
    reflect::{EnumDescriptor, MessageDescriptor, ReflectValueRef},
    MessageDyn,
};

use crate::{interp::ByteCode, CelError, CelResult, CelValueDyn};

use super::{cel_byte_code::CelByteCode, CelBytes};

pub type CelTimeStamp = DateTime<Utc>;
pub type CelValueVec = Vec<CelValue>;
pub type CelValueMap = HashMap<String, CelValue>;

/// The basic value of the CEL interpreter.
///
/// Houses all possible types and implements most of the valid operations within the
/// interpreter
// Only the enum values that are also part of the language are serializable, aka int
// because int literals can exist. If you can't represent it as part of the language
// it doesn't need to be serialized. The time types will be serialized to
// milliseconds resolution.
#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CelValue {
    Int(i64),
    UInt(u64),
    Float(f64),
    Bool(bool),
    String(String),
    Bytes(CelBytes),
    List(CelValueVec),
    Map(CelValueMap),
    Null,
    Ident(String),
    Type(String),
    #[serde(with = "ts_milliseconds")]
    TimeStamp(CelTimeStamp),
    #[serde(
        serialize_with = "DurationMilliSeconds::<i64>::serialize_as",
        deserialize_with = "DurationMilliSeconds::<i64>::deserialize_as"
    )]
    Duration(Duration),
    ByteCode(CelByteCode),
    #[cfg(feature = "protobuf")]
    #[serde(skip_serializing, skip_deserializing)]
    Message(Box<dyn MessageDyn>),
    #[cfg(feature = "protobuf")]
    #[serde(skip_serializing, skip_deserializing)]
    Enum {
        descriptor: EnumDescriptor,
        value: i32,
    },
    #[serde(skip_serializing, skip_deserializing)]
    Dyn(Arc<dyn CelValueDyn>),
    // Error values are now encapsulated.
    Err(CelError),
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
        CelValue::Bytes(val.into())
    }

    pub fn from_byte_slice(val: &[u8]) -> CelValue {
        CelValue::Bytes(val.to_owned().into())
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

    #[cfg(feature = "protobuf")]
    pub fn from_proto_msg(val: Box<dyn MessageDyn>) -> CelValue {
        CelValue::Message(val)
    }

    #[cfg(feature = "protobuf")]
    pub fn from_proto_enum(descriptor: EnumDescriptor, value: i32) -> CelValue {
        CelValue::Enum { descriptor, value }
    }

    pub fn from_type(val: &str) -> CelValue {
        CelValue::Type(val.to_owned())
    }

    pub fn from_timestamp(val: DateTime<Utc>) -> CelValue {
        CelValue::TimeStamp(val)
    }

    pub fn from_duration(val: Duration) -> CelValue {
        CelValue::Duration(val)
    }

    pub(crate) fn from_bytecode(val: Vec<ByteCode>) -> CelValue {
        CelValue::ByteCode(val.into())
    }

    pub fn from_dyn(val: Arc<dyn CelValueDyn>) -> CelValue {
        CelValue::Dyn(val)
    }

    pub fn from_err(val: CelError) -> CelValue {
        CelValue::Err(val)
    }

    pub fn value_error(msg: &str) -> CelValue {
        CelError::Value(msg.to_owned()).into()
    }

    pub fn argument_error(msg: &str) -> CelValue {
        CelError::Argument(msg.to_owned()).into()
    }

    pub fn internal_error(msg: &str) -> CelValue {
        CelError::Internal(msg.to_owned()).into()
    }

    pub fn invalid_op_error(msg: &str) -> CelValue {
        CelError::InvalidOp(msg.to_owned()).into()
    }

    pub fn runtime_error(msg: &str) -> CelValue {
        CelError::Runtime(msg.to_owned()).into()
    }

    pub fn binding_error(sym_name: &str) -> CelValue {
        CelError::Binding {
            symbol: sym_name.to_owned(),
        }
        .into()
    }

    pub fn attribute(parent_name: &str, field_name: &str) -> CelError {
        CelError::Attribute {
            parent: parent_name.to_string(),
            field: field_name.to_string(),
        }
    }

    pub fn into_result(self) -> CelResult<CelValue> {
        match self {
            CelValue::Err(e) => Err(e),
            _ => Ok(self),
        }
    }

    pub fn is_true(&self) -> bool {
        if let CelValue::Bool(val) = self {
            *val
        } else {
            false
        }
    }

    pub fn is_err(&self) -> bool {
        if let CelValue::Err(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_obj(&self) -> bool {
        match self {
            CelValue::Map(_) => true,
            CelValue::Dyn(_) => true,
            _ => false,
        }
    }

    pub fn is_zero(&self) -> bool {
        match self {
            CelValue::Int(i) => *i == 0,
            CelValue::UInt(u) => *u == 0,
            CelValue::Float(f) => *f == 0.0,
            _ => false,
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

    pub fn dyn_type() -> CelValue {
        CelValue::from_type("dyn")
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

    pub fn err_type() -> CelValue {
        CelValue::from_type("err")
    }

    #[cfg(feature = "protobuf")]
    pub fn message_type(desc: &MessageDescriptor) -> CelValue {
        CelValue::Type(format!("message-{}", desc.full_name()))
    }

    #[cfg(feature = "protobuf")]
    pub fn enum_type(desc: &EnumDescriptor) -> CelValue {
        CelValue::Type(format!("enum-{}", desc.full_name()))
    }

    pub fn is_null(&self) -> bool {
        if let CelValue::Null = self {
            true
        } else {
            false
        }
    }

    fn type_prop(lhs: CelValue, rhs: CelValue) -> (CelValue, CelValue) {
        if let CelValue::Int(l) = lhs {
            match rhs {
                CelValue::Int(_) => (lhs, rhs),
                CelValue::UInt(u) => (lhs, (u as i64).into()),
                CelValue::Float(_) => ((l as f64).into(), rhs),
                CelValue::Bool(b) => (lhs, (b as i64).into()),
                _ => (lhs, rhs),
            }
        } else if let CelValue::UInt(l) = lhs {
            match rhs {
                CelValue::Int(_) => ((l as i64).into(), rhs),
                CelValue::UInt(_) => (lhs, rhs),
                CelValue::Float(_) => ((l as f64).into(), rhs),
                CelValue::Bool(b) => (lhs, (b as u64).into()),
                _ => (lhs, rhs),
            }
        } else if let CelValue::Float(_) = lhs {
            match rhs {
                CelValue::Int(i) => (lhs, (i as f64).into()),
                CelValue::UInt(u) => (lhs, (u as f64).into()),
                CelValue::Float(_) => (lhs, rhs),
                CelValue::Bool(b) => (lhs, (if b { 1.0 } else { 0.0 }).into()),
                _ => (lhs, rhs),
            }
        } else if let CelValue::Bool(l) = lhs {
            match rhs {
                CelValue::Int(_) => ((l as i64).into(), rhs),
                CelValue::UInt(_) => ((l as u64).into(), rhs),
                CelValue::Float(_) => ((if l { 1.0 } else { 0.0 }).into(), rhs),
                CelValue::Bool(_) => (lhs, rhs),
                _ => (lhs, rhs),
            }
        } else {
            (lhs, rhs)
        }
    }

    pub fn neq(self, rhs: CelValue) -> CelValue {
        self.error_prop_or(rhs, |lhs, rhs| {
            if let CelValue::Bool(res) = CelValueDyn::eq(&lhs, &rhs) {
                return CelValue::from_bool(!res);
            }

            unreachable!();
        })
    }

    fn ord(self, rhs_value: CelValue) -> CelResult<Option<Ordering>> {
        let type1 = self.as_type();
        let type2 = rhs_value.as_type();

        let (lhs, rhs) = CelValue::type_prop(self, rhs_value);

        match (lhs, rhs) {
            (CelValue::Int(l), CelValue::Int(r)) => Ok(l.partial_cmp(&r)),
            (CelValue::UInt(l), CelValue::UInt(r)) => Ok(l.partial_cmp(&r)),
            (CelValue::Float(l), CelValue::Float(r)) => Ok(l.partial_cmp(&r)),
            (CelValue::Bool(l), CelValue::Bool(r)) => Ok(l.partial_cmp(&r)),
            (CelValue::String(l), CelValue::String(r)) => Ok(l.partial_cmp(&r)),
            (CelValue::Bytes(l), CelValue::Bytes(r)) => Ok(l.partial_cmp(&r)),
            (CelValue::TimeStamp(l), CelValue::TimeStamp(r)) => Ok(l.partial_cmp(&r)),
            (CelValue::Duration(l), CelValue::Duration(r)) => Ok(l.partial_cmp(&r)),
            _ => Err(CelError::invalid_op(&format!(
                "Invalid op 'ord' between {:?} and {:?}",
                type1, type2
            ))),
        }
    }

    pub fn lt(self, rhs: CelValue) -> CelValue {
        self.error_prop_or(rhs, |lhs, rhs| match lhs.ord(rhs) {
            Ok(val) => (val == Some(Ordering::Less)).into(),
            Err(e) => e.into(),
        })
    }

    pub fn gt(self, rhs: CelValue) -> CelValue {
        self.error_prop_or(rhs, |lhs, rhs| match lhs.ord(rhs) {
            Ok(val) => (val == Some(Ordering::Greater)).into(),
            Err(e) => e.into(),
        })
    }

    pub fn le(self, rhs: CelValue) -> CelValue {
        self.error_prop_or(rhs, |lhs, rhs| match lhs.ord(rhs) {
            Ok(val) => (val == Some(Ordering::Less) || val == Some(Ordering::Equal)).into(),
            Err(e) => e.into(),
        })
    }

    pub fn ge(self, rhs: CelValue) -> CelValue {
        self.error_prop_or(rhs, |lhs, rhs| match lhs.ord(rhs) {
            Ok(val) => (val == Some(Ordering::Greater) || val == Some(Ordering::Equal)).into(),
            Err(e) => e.into(),
        })
    }

    pub fn or(&self, rhs: &CelValue) -> CelValue {
        if cfg!(feature = "type_prop") {
            if self.is_err() {
                if rhs.is_truthy() {
                    return CelValue::true_();
                } else {
                    return self.clone();
                }
            }

            if rhs.is_err() {
                if self.is_truthy() {
                    return CelValue::true_();
                } else {
                    return rhs.clone();
                }
            }

            return (self.is_truthy() || rhs.is_truthy()).into();
        } else {
            if self.is_err() {
                if rhs.is_true() {
                    return CelValue::true_();
                } else {
                    return self.clone();
                }
            }

            if rhs.is_err() {
                if self.is_true() {
                    return CelValue::true_();
                } else {
                    return rhs.clone();
                }
            }
        }

        if let CelValue::Bool(lhs) = self {
            if let CelValue::Bool(rhs) = rhs {
                return (*lhs || *rhs).into();
            }
        }

        CelValue::from_err(CelError::invalid_op(&format!(
            "|| operator invalid for {:?} and {:?}",
            self.as_type(),
            rhs.as_type(),
        )))
    }

    pub fn in_(self, rhs: CelValue) -> CelValue {
        self.error_prop_or(rhs, |lhs, rhs| {
            let rhs_type = rhs.as_type();
            let lhs_type = lhs.as_type();

            match rhs {
                CelValue::List(l) => {
                    for value in l.iter() {
                        if lhs == *value {
                            return true.into();
                        }
                    }

                    false.into()
                }
                CelValue::Map(m) => {
                    if let CelValue::String(r) = lhs {
                        CelValue::from_bool(m.contains_key(&r))
                    } else {
                        CelValue::from_err(CelError::invalid_op(&format!(
                            "Op 'in' invalid between {:?} and {:?}",
                            lhs_type, rhs_type
                        )))
                    }
                }
                CelValue::String(s) => {
                    if let CelValue::String(r) = lhs {
                        CelValue::from_bool(s.contains(&r))
                    } else {
                        CelValue::from_err(CelError::invalid_op(&format!(
                            "Op 'in' invalid between {:?} and {:?}",
                            lhs_type, rhs_type
                        )))
                    }
                }
                _ => CelValue::from_err(CelError::invalid_op(&format!(
                    "Op 'in' invalid between {:?} and {:?}",
                    lhs_type, rhs_type
                ))),
            }
        })
    }

    pub fn and(self, rhs: CelValue) -> CelValue {
        self.error_prop_or(rhs, |lhs, rhs| {
            if cfg!(feature = "type_prop") {
                return (lhs.is_truthy() && rhs.is_truthy()).into();
            }

            if let CelValue::Bool(lhs) = lhs {
                if let CelValue::Bool(rhs) = rhs {
                    return (lhs && rhs).into();
                }
            }

            CelValue::from_err(CelError::invalid_op(&format!(
                "&& operator invalid for {:?} and {:?}",
                lhs.as_type(),
                rhs.as_type(),
            )))
        })
    }

    #[inline]
    fn error_prop_or<F>(self, rhs: CelValue, f: F) -> CelValue
    where
        F: Fn(CelValue, CelValue) -> CelValue,
    {
        if self.is_err() {
            self.clone()
        } else if rhs.is_err() {
            rhs
        } else {
            f(self, rhs)
        }
    }

    pub fn index(self, ival: CelValue) -> CelValue {
        self.error_prop_or(ival, |obj, index| match obj {
            CelValue::List(list) => {
                if let CelValue::UInt(index) = index {
                    if index as usize >= list.len() {
                        return CelValue::from_err(CelError::value("List access out of bounds"));
                    }

                    return list[index as usize].clone();
                } else if let CelValue::Int(index) = index {
                    if index < 0 {
                        if cfg!(feature = "neg_index") {
                            let adjusted_index: isize = match TryInto::<isize>::try_into(list.len())
                            {
                                Ok(v) => v,
                                Err(_) => {
                                    return CelValue::from_err(CelError::value(
                                        "List access out of bounds",
                                    ))
                                }
                            } + (index as isize);

                            if adjusted_index < 0
                                || TryInto::<usize>::try_into(adjusted_index).unwrap() >= list.len()
                            {
                                return CelValue::from_err(CelError::value(
                                    "List access out of bounds 3",
                                ));
                            }

                            list[adjusted_index as usize].clone()
                        } else {
                            return CelValue::from_err(CelError::value(
                                "Negative index is not allowed",
                            ));
                        }
                    } else {
                        if index as usize >= list.len() {
                            return CelValue::from_err(CelError::value(
                                "List access out of bounds",
                            ));
                        }

                        list[index as usize].clone()
                    }
                } else {
                    return CelValue::from_err(CelError::value(
                        "List index can only be int or uint",
                    ));
                }
            }
            CelValue::Map(map) => {
                if let CelValue::String(index) = index {
                    match map.get(index.as_str()) {
                        Some(val) => return val.clone(),
                        None => {
                            return CelValue::from_err(CelError::attribute("obj", &index));
                        }
                    }
                } else {
                    CelValue::from_err(CelError::value(&format!(
                        "Map index operator mush be a string, found {:?}",
                        index.as_type()
                    )))
                }
            }
            CelValue::Dyn(d) => {
                if let CelValue::String(index) = index {
                    return d.access(&index);
                } else {
                    CelValue::from_err(CelError::value(&format!(
                        "Dyn index operator mush be a string, found {:?}",
                        index.as_type()
                    )))
                }
            }
            _ => CelValue::from_err(CelError::value(&format!(
                "Index operator invalid between {:?} and {:?}",
                index.as_type(),
                obj.as_type()
            ))),
        })
    }
}

impl CelValueDyn for CelValue {
    fn as_type(&self) -> CelValue {
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
            #[cfg(feature = "protobuf")]
            CelValue::Message(msg) => CelValue::message_type(&msg.descriptor_dyn()),
            #[cfg(feature = "protobuf")]
            CelValue::Enum {
                descriptor,
                value: _value,
            } => CelValue::enum_type(&descriptor),
            CelValue::Dyn(obj) => obj.as_type(),
            CelValue::Err(_) => CelValue::err_type(),
        }
    }

    fn access(&self, key: &str) -> CelValue {
        if self.is_err() {
            return self.clone();
        }

        let self_type = self.as_type();

        match self {
            CelValue::Map(ref map) => match map.get(key) {
                Some(val) => val.clone(),
                None => CelValue::from_err(CelError::attribute("obj", key)),
            },
            CelValue::Dyn(ref d) => d.access(key),
            #[cfg(feature = "protobuf")]
            CelValue::Message(msg) => {
                let desc = msg.descriptor_dyn();

                if let Some(field) = desc.field_by_name(key) {
                    field.get_singular_field_or_default(msg.as_ref()).into()
                } else {
                    CelValue::from_err(CelError::attribute("msg", key))
                }
            }
            _ => CelValue::from_err(CelError::invalid_op(&format!(
                "Access invalid on type {}",
                self_type
            ))),
        }
    }

    fn eq(&self, rhs_val: &CelValue) -> CelValue {
        self.clone()
            .error_prop_or(rhs_val.clone(), |lhs_val, rhs_val| {
                let rhs = if let CelValue::Dyn(d) = rhs_val {
                    match d.any_ref().downcast_ref::<CelValue>() {
                        Some(v) => v.clone(),
                        None => CelValue::Dyn(d),
                    }
                } else {
                    rhs_val
                };

                let (lhs, rhs) = CelValue::type_prop(lhs_val, rhs);

                match (lhs, rhs) {
                    (CelValue::Int(l), CelValue::Int(r)) => CelValue::from_bool(l == r),
                    (CelValue::UInt(l), CelValue::UInt(r)) => CelValue::from_bool(l == r),
                    (CelValue::Float(l), CelValue::Float(r)) => CelValue::from_bool(l == r),
                    (CelValue::Bool(l), CelValue::Bool(r)) => CelValue::from_bool(l == r),
                    (CelValue::String(l), CelValue::String(r)) => CelValue::from_bool(l == r),
                    (CelValue::Bytes(l), CelValue::Bytes(r)) => CelValue::from_bool(l == r),
                    (CelValue::List(l), CelValue::List(r)) => {
                        if l.len() != r.len() {
                            CelValue::false_()
                        } else {
                            for (v1, v2) in zip(l, r) {
                                match CelValueDyn::eq(&v1, &v2) {
                                    CelValue::Err(err) => return CelValue::from_err(err),
                                    other => {
                                        if !other.is_true() {
                                            return CelValue::false_();
                                        }
                                    }
                                }
                            }
                            CelValue::true_()
                        }
                    }
                    (CelValue::Map(l), CelValue::Map(r)) => {
                        let mut r = r.clone();

                        for (k, v1) in l.into_iter() {
                            if let Some(v2) = r.remove(&k) {
                                if !CelValueDyn::eq(&v1, &v2).is_true() {
                                    return CelValue::false_();
                                }
                            } else {
                                return CelValue::false_();
                            }
                        }

                        if !r.is_empty() {
                            return CelValue::false_();
                        }

                        CelValue::true_()
                    }
                    (CelValue::Null, CelValue::Null) => CelValue::true_(),
                    (CelValue::Null, _) => CelValue::false_(),
                    (CelValue::TimeStamp(l), CelValue::TimeStamp(r)) => CelValue::from_bool(l == r),
                    (CelValue::Duration(l), CelValue::Duration(r)) => CelValue::from_bool(l == r),
                    (CelValue::Type(l), CelValue::Type(r)) => CelValue::from_bool(l == r),
                    #[cfg(feature = "protobuf")]
                    (CelValue::Message(l), CelValue::Message(r)) => {
                        CelValue::from_bool(l.descriptor_dyn().eq(l.as_ref(), r.as_ref()))
                    }
                    #[cfg(feature = "protobuf")]
                    (
                        CelValue::Enum {
                            descriptor: l_desc,
                            value: l_value,
                        },
                        CelValue::Enum {
                            descriptor: r_desc,
                            value: r_value,
                        },
                    ) => CelValue::from_bool(l_value == r_value && l_desc == r_desc),
                    #[cfg(feature = "protobuf")]
                    (
                        CelValue::Enum {
                            descriptor: _l_desc,
                            value: l_value,
                        },
                        CelValue::Int(intval),
                    ) => CelValue::from_bool(intval == (l_value as i64)),
                    #[cfg(feature = "protobuf")]
                    (
                        CelValue::Enum {
                            descriptor: _l_desc,
                            value: l_value,
                        },
                        CelValue::UInt(intval),
                    ) => CelValue::from_bool(intval == (l_value as u64)),
                    #[cfg(feature = "protobuf")]
                    (
                        CelValue::Enum {
                            descriptor: l_desc,
                            value: _l_value,
                        },
                        CelValue::String(strval),
                    ) => {
                        if let Some(_) = l_desc.value_by_name(&strval) {
                            CelValue::true_()
                        } else {
                            CelValue::false_()
                        }
                    }
                    (CelValue::Dyn(d), rhs) => d.eq(&rhs),
                    // (_a, _b) => CelValue::from_err(CelError::invalid_op(&format!(
                    //     "Invalid op '==' between {:?} and {:?}",
                    //     type1, type2
                    // ))),
                    (_a, _b) => CelValue::false_(),
                }
            })
    }

    fn is_truthy(&self) -> bool {
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
            #[cfg(feature = "protobuf")]
            CelValue::Enum {
                descriptor: _,
                value,
            } => *value != 0,
            #[cfg(feature = "protobuf")]
            CelValue::Message(_) => true,
            CelValue::Dyn(obj) => obj.is_truthy(),
            CelValue::Err(_) => false,
            _ => false,
        }
    }

    fn any_ref<'a>(&'a self) -> &'a dyn Any {
        self
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

#[cfg(feature = "protobuf")]
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
            ReflectValueRef::Bytes(b) => CelValue::Bytes(b.to_owned().into()),
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

impl From<f64> for CelValue {
    fn from(val: f64) -> CelValue {
        CelValue::from_float(val)
    }
}

impl From<bool> for CelValue {
    fn from(val: bool) -> CelValue {
        CelValue::from_bool(val)
    }
}

impl From<Duration> for CelValue {
    fn from(val: Duration) -> CelValue {
        CelValue::from_duration(val)
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

impl From<&[u8]> for CelValue {
    fn from(val: &[u8]) -> CelValue {
        CelValue::from_bytes(val.to_owned())
    }
}

impl From<&[CelValue]> for CelValue {
    fn from(val: &[CelValue]) -> CelValue {
        CelValue::from_list(val.to_vec())
    }
}

impl From<HashMap<String, CelValue>> for CelValue {
    fn from(val: HashMap<String, CelValue>) -> CelValue {
        CelValue::from_map(val)
    }
}

impl From<DateTime<Utc>> for CelValue {
    fn from(val: DateTime<Utc>) -> CelValue {
        CelValue::from_timestamp(val)
    }
}

impl From<DateTime<FixedOffset>> for CelValue {
    fn from(val: DateTime<FixedOffset>) -> CelValue {
        CelValue::from_timestamp(val.into())
    }
}
impl From<Vec<ByteCode>> for CelValue {
    fn from(val: Vec<ByteCode>) -> CelValue {
        CelValue::from_bytecode(val)
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

impl TryInto<f64> for CelValue {
    type Error = CelError;

    fn try_into(self) -> CelResult<f64> {
        if let CelValue::Float(val) = self {
            return Ok(val);
        }

        Err(CelError::internal("Convertion Error"))
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

impl TryInto<String> for CelValue {
    type Error = CelError;

    fn try_into(self) -> CelResult<String> {
        if let CelValue::String(val) = self {
            return Ok(val);
        }

        Err(CelError::internal("Convertion Error"))
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

impl TryInto<HashMap<String, CelValue>> for CelValue {
    type Error = CelError;

    fn try_into(self) -> CelResult<HashMap<String, CelValue>> {
        if let CelValue::Map(val) = self {
            return Ok(val);
        }

        Err(CelError::internal("Convertion Error"))
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

impl TryInto<i64> for CelValue {
    type Error = CelError;

    fn try_into(self) -> CelResult<i64> {
        if let CelValue::Int(val) = self {
            return Ok(val);
        }

        Err(CelError::internal("Convertion Error"))
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

impl TryInto<Vec<ByteCode>> for CelValue {
    type Error = CelError;

    fn try_into(self) -> CelResult<Vec<ByteCode>> {
        if let CelValue::ByteCode(val) = self {
            return Ok(val.into());
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
        } else {
            match (self, other) {
                #[cfg(feature = "protobuf")]
                (CelValue::Message(lhs), CelValue::Message(rhs)) => {
                    lhs.descriptor_dyn().eq(lhs.as_ref(), rhs.as_ref())
                }
                #[cfg(feature = "protobuf")]
                (
                    CelValue::Enum {
                        descriptor: lhs_descriptor,
                        value: lhs_value,
                    },
                    CelValue::Enum {
                        descriptor: rhs_descriptor,
                        value: rhs_value,
                    },
                ) => lhs_value == rhs_value && lhs_descriptor == rhs_descriptor,
                _ => false,
            }
        }
    }
}

impl Add for CelValue {
    type Output = CelValue;

    fn add(self, rhs_val: Self) -> Self::Output {
        self.error_prop_or(rhs_val, |lhs_val, rhs_val| {
            let type1 = lhs_val.as_type();
            let type2 = rhs_val.as_type();

            let (lhs, rhs) = if cfg!(feature = "type_prop") {
                CelValue::type_prop(lhs_val, rhs_val)
            } else {
                (lhs_val, rhs_val)
            };

            match lhs {
                CelValue::Int(val1) => {
                    if let CelValue::Int(val2) = rhs {
                        return CelValue::from(val1 + val2);
                    }
                }
                CelValue::UInt(val1) => {
                    if let CelValue::UInt(val2) = rhs {
                        return CelValue::from(val1 + val2);
                    }
                }
                CelValue::Float(val1) => {
                    if let CelValue::Float(val2) = rhs {
                        return CelValue::from(val1 + val2);
                    }
                }
                CelValue::String(val1) => {
                    if let CelValue::String(val2) = rhs {
                        let mut res = val1;
                        res.push_str(&val2);
                        return CelValue::from_str(res.as_ref());
                    }
                }
                CelValue::Bytes(val1) => {
                    if let CelValue::Bytes(val2) = rhs {
                        let mut res = val1;
                        res.extend(val2.into_vec());
                        return CelValue::Bytes(res);
                    }
                }
                CelValue::List(val1) => {
                    if let CelValue::List(val2) = rhs {
                        let mut res = val1;
                        res.extend_from_slice(&val2);
                        return CelValue::from_val_slice(res.as_ref());
                    }
                }
                CelValue::TimeStamp(v1) => {
                    if let CelValue::Duration(v2) = rhs {
                        return CelValue::from_timestamp(v1 + v2);
                    }
                }
                CelValue::Duration(v1) => match rhs {
                    CelValue::TimeStamp(v2) => return CelValue::from_timestamp(v2 + v1),
                    CelValue::Duration(v2) => return CelValue::Duration(v1 + v2),
                    _ => {}
                },
                _ => {}
            }

            CelValue::from_err(CelError::invalid_op(&format!(
                "Invalid op '+' between {:?} and {:?}",
                type1, type2
            )))
        })
    }
}

impl Sub for CelValue {
    type Output = CelValue;

    fn sub(self, rhs_val: Self) -> Self::Output {
        self.error_prop_or(rhs_val, |lhs_val, rhs_val| {
            let type1 = lhs_val.as_type();
            let type2 = rhs_val.as_type();

            let (lhs, rhs) = if cfg!(feature = "type_prop") {
                CelValue::type_prop(lhs_val, rhs_val)
            } else {
                (lhs_val, rhs_val)
            };

            match lhs {
                CelValue::Int(val1) => {
                    if let CelValue::Int(val2) = rhs {
                        return CelValue::from(val1 - val2);
                    }
                }
                CelValue::UInt(val1) => {
                    if let CelValue::UInt(val2) = rhs {
                        return CelValue::from(val1 - val2);
                    }
                }
                CelValue::Float(val1) => {
                    if let CelValue::Float(val2) = rhs {
                        return CelValue::from(val1 - val2);
                    }
                }
                CelValue::TimeStamp(v1) => match rhs {
                    CelValue::Duration(v2) => return CelValue::from_timestamp(v1 - v2),
                    CelValue::TimeStamp(v2) => return CelValue::from_duration(v1 - v2),
                    _ => {}
                },
                CelValue::Duration(v1) => match rhs {
                    CelValue::TimeStamp(v2) => return CelValue::from_timestamp(v2 - v1),
                    CelValue::Duration(v2) => return CelValue::from_duration(v1 - v2),
                    _ => {}
                },
                _ => {}
            }

            CelValue::from_err(CelError::invalid_op(&format!(
                "Invalid op '-' between {:?} and {:?}",
                type1, type2
            )))
        })
    }
}

impl Mul for CelValue {
    type Output = CelValue;

    fn mul(self, rhs_val: Self) -> Self::Output {
        self.error_prop_or(rhs_val, |lhs_val, rhs_val| {
            let type1 = lhs_val.as_type();
            let type2 = rhs_val.as_type();

            let (lhs, rhs) = if cfg!(feature = "type_prop") {
                CelValue::type_prop(lhs_val, rhs_val)
            } else {
                (lhs_val, rhs_val)
            };

            match lhs {
                CelValue::Int(val1) => {
                    if let CelValue::Int(val2) = rhs {
                        return CelValue::from(val1 * val2);
                    }
                }
                CelValue::UInt(val1) => {
                    if let CelValue::UInt(val2) = rhs {
                        return CelValue::from(val1 * val2);
                    }
                }
                CelValue::Float(val1) => {
                    if let CelValue::Float(val2) = rhs {
                        return CelValue::from(val1 * val2);
                    }
                }
                _ => {}
            }

            CelValue::from_err(CelError::invalid_op(&format!(
                "Invalid op '*' between {:?} and {:?}",
                type1, type2
            )))
        })
    }
}

impl Div for CelValue {
    type Output = CelValue;

    fn div(self, rhs_val: Self) -> Self::Output {
        self.error_prop_or(rhs_val, |lhs_val, rhs_val| {
            let type1 = lhs_val.as_type();
            let type2 = rhs_val.as_type();

            let (lhs, rhs) = if cfg!(feature = "type_prop") {
                CelValue::type_prop(lhs_val, rhs_val)
            } else {
                (lhs_val, rhs_val)
            };

            match lhs {
                CelValue::Int(val1) => {
                    if let CelValue::Int(val2) = rhs {
                        if val2 == 0 {
                            return CelValue::from_err(CelError::DivideByZero);
                        }

                        return CelValue::from(val1 / val2);
                    }
                }
                CelValue::UInt(val1) => {
                    if let CelValue::UInt(val2) = rhs {
                        if val2 == 0 {
                            return CelValue::from_err(CelError::DivideByZero);
                        }

                        return CelValue::from(val1 / val2);
                    }
                }
                CelValue::Float(val1) => {
                    if let CelValue::Float(val2) = rhs {
                        return CelValue::from(val1 / val2);
                    }
                }
                _ => {}
            }

            CelValue::from_err(CelError::invalid_op(&format!(
                "Invalid op '/' between {:?} and {:?}",
                type1, type2
            )))
        })
    }
}

impl Rem for CelValue {
    type Output = CelValue;

    fn rem(self, rhs_val: Self) -> Self::Output {
        self.error_prop_or(rhs_val, |lhs_val, rhs_val| {
            let type1 = lhs_val.as_type();
            let type2 = rhs_val.as_type();

            let (lhs, rhs) = if cfg!(feature = "type_prop") {
                CelValue::type_prop(lhs_val, rhs_val)
            } else {
                (lhs_val, rhs_val)
            };

            match lhs {
                CelValue::Int(val1) => {
                    if let CelValue::Int(val2) = rhs {
                        return CelValue::from(val1 % val2);
                    }
                }
                CelValue::UInt(val1) => {
                    if let CelValue::UInt(val2) = rhs {
                        return CelValue::from(val1 % val2);
                    }
                }
                _ => {}
            }

            CelValue::from_err(CelError::invalid_op(&format!(
                "Invalid op '/' between {:?} and {:?}",
                type1, type2
            )))
        })
    }
}

impl Neg for CelValue {
    type Output = CelValue;

    fn neg(self) -> Self::Output {
        if self.is_err() {
            return self.clone();
        }

        let type1 = self.as_type();

        match self {
            CelValue::Int(val1) => {
                return CelValue::from(-val1);
            }
            CelValue::Float(val1) => {
                return CelValue::from(-val1);
            }
            _ => {}
        }

        CelValue::from_err(CelError::invalid_op(&format!(
            "Invalid op '-' on {:?}",
            type1
        )))
    }
}

impl Not for CelValue {
    type Output = CelValue;

    #[cfg(not(feature = "type_prop"))]
    fn not(self) -> Self::Output {
        let type1 = self.as_type();

        if self.is_err() {
            return self.clone();
        }

        match self {
            CelValue::Bool(val1) => {
                return CelValue::from(!val1);
            }
            _ => {}
        }

        CelValue::from_err(CelError::invalid_op(&format!(
            "Invalid op '-' on {:?}",
            type1
        )))
    }

    #[cfg(feature = "type_prop")]
    fn not(self) -> Self::Output {
        if self.is_err() {
            return self.clone();
        }

        (!self.is_truthy()).into()
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
            #[cfg(feature = "protobuf")]
            Message(msg) => write!(f, "{}", msg.as_ref()),
            #[cfg(feature = "protobuf")]
            Enum { descriptor, value } => {
                if let Some(v) = descriptor.value_by_number(*value) {
                    write!(f, "{}::{} ({})", descriptor.full_name(), v.name(), value)
                } else {
                    write!(f, "{}::({})", descriptor.full_name(), value)
                }
            }
            Dyn(val) => write!(f, "{}", val.as_ref()),
            Err(err) => write!(f, "Err: {}", err),
        }
    }
}

impl From<CelByteCode> for CelValue {
    fn from(value: CelByteCode) -> Self {
        CelValue::ByteCode(value)
    }
}

impl From<CelBytes> for CelValue {
    fn from(value: CelBytes) -> Self {
        CelValue::Bytes(value)
    }
}

impl From<CelError> for CelValue {
    fn from(value: CelError) -> Self {
        CelValue::Err(value)
    }
}

impl<T: Into<CelValue>> From<Vec<T>> for CelValue {
    fn from(value: Vec<T>) -> Self {
        CelValue::List(
            value
                .into_iter()
                .map(|i| i.into())
                .collect::<Vec<CelValue>>(),
        )
    }
}

impl<T: Into<CelValue>> From<CelResult<T>> for CelValue {
    fn from(value: CelResult<T>) -> Self {
        match value {
            Ok(val) => val.into(),
            Err(e) => e.into(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::CelValue;

    #[test]
    fn test_add() {
        let res = CelValue::from(4i64) + CelValue::from(5i64);

        let val: i64 = res.try_into().unwrap();
        assert!(val == 9);
    }

    #[cfg(not(feature = "type_prop"))]
    #[test]
    fn test_bad_op() {
        let res = CelValue::from(3i64) + CelValue::from(4.2);

        assert!(res.is_err());
    }
}
