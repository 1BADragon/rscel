use std::{collections::HashMap, str::FromStr};

use chrono::TimeZone;
use num::FromPrimitive;
use rscel::{CelError, CelResult, CelValue};
use wasm_bindgen::prelude::*;

use crate::{object_iter::ObjectIterator, values};

#[wasm_bindgen]
pub struct WasmCelValue(CelValue);

impl WasmCelValue {
    pub fn new(inner: CelValue) -> Self {
        Self(inner)
    }

    pub fn into_inner(self) -> CelValue {
        self.0
    }
}

impl From<CelValue> for WasmCelValue {
    fn from(value: CelValue) -> Self {
        Self(value)
    }
}

fn extract_number_value<T: num::cast::FromPrimitive + FromStr>(
    obj: &js_sys::Object,
    field: &str,
) -> CelResult<T> {
    let field_value = field.into();
    if let Ok(numval) = js_sys::Reflect::get(obj, &field_value) {
        if let Some(float_val) = numval.as_f64() {
            match FromPrimitive::from_f64(float_val) {
                Some(val) => Ok(val),
                None => Err(CelError::value(&format!(
                    "{} is invalid for {}",
                    float_val, field
                ))),
            }
        } else if let Some(str_val) = numval.as_string() {
            match str_val.parse::<T>() {
                Ok(val) => Ok(val),
                Err(_) => Err(CelError::value(&format!(
                    "{} is invalid for {}",
                    str_val, field
                ))),
            }
        } else if numval.is_bigint() {
            let bigint_val: js_sys::BigInt = numval.into();
            let str_val: String = bigint_val.to_string(10).unwrap().into();
            match str_val.parse::<T>() {
                Ok(val) => Ok(val),
                Err(_) => Err(CelError::value(&format!(
                    "{} is invalid for {}",
                    str_val, field
                ))),
            }
        } else {
            Err(CelError::value(&format!("Invalid value for {}", field)))
        }
    } else {
        Err(CelError::internal("Unable to collect object field"))
    }
}

fn js_val_to_cel(value: JsValue) -> CelResult<CelValue> {
    if value.is_object() {
        if value.is_instance_of::<js_sys::Date>() {
            let date: js_sys::Date = value.into();
            Ok(WasmCelValue::new(
                (chrono::Utc
                    .with_ymd_and_hms(
                        date.get_utc_full_year() as i32,
                        date.get_utc_month() + 1,
                        date.get_utc_date(),
                        date.get_utc_hours(),
                        date.get_utc_minutes(),
                        date.get_utc_seconds(),
                    )
                    .unwrap()
                    + chrono::TimeDelta::milliseconds(date.get_utc_milliseconds() as i64 % 1000))
                .into(),
            ))
        } else if value.is_array() {
            let mut list: Vec<CelValue> = Vec::new();

            for list_value in values(&value).into_iter() {
                list.push(TryInto::<WasmCelValue>::try_into(list_value)?.into_inner());
            }

            Ok(WasmCelValue::new(CelValue::from_list(list)))
        } else {
            let obj: js_sys::Object = value.into();

            if obj.has_own_property(&"cel_float".into()) {
                Ok(WasmCelValue::new(CelValue::from_float(
                    extract_number_value(&obj, "cel_float")?,
                )))
            } else if obj.has_own_property(&"cel_int".into()) {
                Ok(WasmCelValue::new(CelValue::from_int(extract_number_value(
                    &obj, "cel_int",
                )?)))
            } else if obj.has_own_property(&"cel_uint".into()) {
                Ok(WasmCelValue::new(CelValue::from_uint(
                    extract_number_value(&obj, "cel_uint")?,
                )))
            } else {
                let mut map = HashMap::new();

                for (key, value) in ObjectIterator::new(obj) {
                    map.insert(key, TryInto::<WasmCelValue>::try_into(value)?.into_inner());
                }

                Ok(WasmCelValue::new(CelValue::from_map(map)))
            }
        }
    } else if value.is_bigint() {
        let bigint_val: js_sys::BigInt = value.into();

        let str_val: String = bigint_val.to_string(10).unwrap().into();
        match str_val.parse::<i64>() {
            Ok(val) => Ok(WasmCelValue::new(val.into())),
            Err(_) => Err(CelError::value(&format!("{} is invalid for int", str_val))),
        }
    } else if let Some(numval) = value.dyn_ref::<js_sys::Number>() {
        if numval
            .to_string(10)
            .is_ok_and(|x| x.as_string().is_some_and(|s| s.contains('.')))
        {
            Ok(WasmCelValue::new(CelValue::from_float(numval.value_of())))
        } else {
            Ok(WasmCelValue::new(CelValue::from_int(
                numval.value_of() as i64
            )))
        }
    } else if value.is_string() {
        Ok(WasmCelValue::new(CelValue::from_string(
            value.as_string().unwrap(),
        )))
    } else if value.is_null() || value.is_undefined() {
        Ok(WasmCelValue::new(CelValue::from_null()))
    } else if value.is_truthy() {
        Ok(WasmCelValue::new(CelValue::from_bool(true)))
    } else if value.is_falsy() {
        Ok(WasmCelValue::new(CelValue::from_bool(false)))
    } else {
        Err(CelError::value("Unknown js binding"))
    }
}

fn cel_value_into_js(self) -> JsValue {
    match self.into_inner() {
        CelValue::Int(i) => i.into(),
        CelValue::UInt(u) => u.into(),
        CelValue::Float(f) => f.into(),
        CelValue::Bool(b) => b.into(),
        CelValue::String(s) => s.into(),
        CelValue::Bytes(b) => {
            let arr = js_sys::Uint8Array::new_with_length(b.len() as u32);

            for (index, byte) in b.into_iter().enumerate() {
                arr.set(&(byte as u64).into(), index as u32);
            }

            arr.into()
        }
        CelValue::List(l) => {
            let arr = js_sys::Array::new();

            for value in l.into_iter() {
                arr.push(&WasmCelValue::new(value).into());
            }

            arr.into()
        }
        CelValue::Map(m) => {
            let obj = js_sys::Object::new();

            for (key, value) in m.into_iter() {
                js_sys::Reflect::set(&obj, &key.into(), &WasmCelValue::new(value).into()).unwrap();
            }

            obj.into()
        }
        CelValue::Null => JsValue::undefined(),
        CelValue::Ident(ident) => {
            let obj = js_sys::Object::new();

            js_sys::Reflect::set(&obj, &"ident".into(), &ident.into()).unwrap();

            obj.into()
        }
        CelValue::Type(t) => {
            let obj = js_sys::Object::new();

            js_sys::Reflect::set(&obj, &"type".into(), &t.into()).unwrap();

            obj.into()
        }
        CelValue::TimeStamp(t) => js_sys::Date::new(&t.to_rfc3339().into()).into(),
        CelValue::Duration(d) => {
            let obj = js_sys::Object::new();

            js_sys::Reflect::set(&obj, &"sec".into(), &d.num_seconds().into()).unwrap();
            js_sys::Reflect::set(&obj, &"nsec".into(), &d.subsec_nanos().into()).unwrap();

            obj.into()
        }
        CelValue::ByteCode(_) => js_sys::Object::new().into(),
        _ => unimplemented!(),
    }
}
