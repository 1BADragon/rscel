use std::{collections::HashMap, str::FromStr};

use num::FromPrimitive;
use wasm_bindgen::{JsCast, JsValue};

use crate::{CelError, CelResult, CelValue};

use super::{log, object_iter::ObjectIterator, values};

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

impl TryFrom<JsValue> for CelValue {
    type Error = CelError;
    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        if value.is_object() {
            if value.is_array() {
                let mut list: Vec<CelValue> = Vec::new();

                for list_value in values(&value).into_iter() {
                    list.push(list_value.try_into()?);
                }

                Ok(CelValue::from_list(list))
            } else {
                let obj: js_sys::Object = value.into();

                if obj.has_own_property(&"cel_float".into()) {
                    Ok(CelValue::from_float(extract_number_value(
                        &obj,
                        "cel_float",
                    )?))
                } else if obj.has_own_property(&"cel_int".into()) {
                    Ok(CelValue::from_int(extract_number_value(&obj, "cel_int")?))
                } else if obj.has_own_property(&"cel_uint".into()) {
                    Ok(CelValue::from_uint(extract_number_value(&obj, "cel_uint")?))
                } else {
                    let mut map = HashMap::new();

                    for (key, value) in ObjectIterator::new(obj) {
                        map.insert(key, value.try_into()?);
                    }

                    Ok(CelValue::from_map(map))
                }
            }
        } else if let Some(numval) = value.dyn_ref::<js_sys::Number>() {
            if numval
                .to_string(10)
                .is_ok_and(|x| x.as_string().is_some_and(|s| s.contains('.')))
            {
                Ok(CelValue::from_float(numval.value_of()))
            } else {
                Ok(CelValue::from_int(numval.value_of() as i64))
            }
        } else if value.is_string() {
            Ok(CelValue::from_string(value.as_string().unwrap()))
        } else if value.is_null() || value.is_undefined() {
            Ok(CelValue::from_null())
        } else if value.is_truthy() {
            Ok(CelValue::from_bool(true))
        } else if value.is_falsy() {
            Ok(CelValue::from_bool(false))
        } else {
            Err(CelError::value("Unknown js binding"))
        }
    }
}
