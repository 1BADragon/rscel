use chrono::{DateTime, Utc};

use crate::{CelError, CelResult, CelValue};

pub fn construct_type(type_name: &str, args: &[CelValue]) -> CelResult<CelValue> {
    match type_name {
        "bool" => bool_impl(CelValue::from_null(), args),
        "int" => int_impl(CelValue::from_null(), args),
        "uint" => uint_impl(CelValue::from_null(), args),
        "float" => double_impl(CelValue::from_null(), args),
        "bytes" => bytes_impl(CelValue::from_null(), args),
        "string" => string_impl(CelValue::from_null(), args),
        "type" => type_impl(CelValue::from_null(), args),
        "timestamp" => timestamp_impl(CelValue::from_null(), args),
        "duration" => duration_impl(CelValue::from_null(), args),

        _ => Err(CelError::runtime(&format!(
            "{} is not constructable",
            type_name
        ))),
    }
}

fn bool_impl(_: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    if args.len() != 1 {
        return Err(CelError::argument("bool() expects exactly one argument"));
    }

    Ok(args[0].is_truthy().into())
}

fn int_impl(_: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    use CelValue::*;

    if args.len() != 1 {
        return Err(CelError::argument("int() expects exactly one argument"));
    }

    match &args[0] {
        Int(val) => Ok(CelValue::from_int(*val)),
        UInt(val) => Ok(CelValue::from_int(*val as i64)),
        Float(val) => Ok(CelValue::from_int(*val as i64)),
        Bool(val) => Ok(CelValue::from_int(if *val { 1 } else { 0 })),
        String(val) => match val.parse::<i64>() {
            Ok(res) => Ok(CelValue::from_int(res)),
            Err(_err) => Err(CelError::value(&format!(
                "int conversion invalid for \"{}\"",
                val
            ))),
        },
        TimeStamp(val) => Ok(CelValue::from_int(val.timestamp())),
        other => Err(CelError::value(&format!(
            "int conversion invalid for {:?}",
            other
        ))),
    }
}

fn uint_impl(_: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    use CelValue::*;

    if args.len() != 1 {
        return Err(CelError::argument("uint() expects exactly one argument"));
    }

    match &args[0] {
        Int(val) => Ok(CelValue::from_uint(*val as u64)),
        UInt(val) => Ok(CelValue::from_uint(*val)),
        Float(val) => Ok(CelValue::from_uint(*val as u64)),
        Bool(val) => Ok(CelValue::from_uint(if *val { 1 } else { 0 })),
        String(val) => match val.parse::<u64>() {
            Ok(res) => Ok(CelValue::from_uint(res)),
            Err(_err) => Err(CelError::value(&format!(
                "int conversion invalid for \"{}\"",
                val
            ))),
        },
        other => Err(CelError::value(&format!(
            "int conversion invalid for {:?}",
            other
        ))),
    }
}

fn double_impl(_: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    use CelValue::*;

    if args.len() != 1 {
        return Err(CelError::argument("double() expects exactly one argument"));
    }

    match &args[0] {
        Int(val) => Ok(CelValue::from_float(*val as f64)),
        UInt(val) => Ok(CelValue::from_float(*val as f64)),
        Float(val) => Ok(CelValue::from_float(*val)),
        Bool(val) => Ok(CelValue::from_float(if *val { 1.0 } else { 0.0 })),
        String(val) => match val.parse::<f64>() {
            Ok(res) => Ok(CelValue::from_float(res)),
            Err(_err) => Err(CelError::value(&format!(
                "int conversion invalid for \"{}\"",
                val
            ))),
        },
        other => Err(CelError::value(&format!(
            "int conversion invalid for {:?}",
            other
        ))),
    }
}

fn bytes_impl(_this: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    use CelValue::*;
    if args.len() != 1 {
        return Err(CelError::argument("bytes() expects exactly one argument"));
    }

    match &args[0] {
        String(val) => Ok(CelValue::from_bytes(val.as_bytes().to_vec())),
        other => Err(CelError::value(&format!(
            "int conversion invalid for {:?}",
            other
        ))),
    }
}

fn string_impl(_this: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    use CelValue::*;

    if args.len() != 1 {
        return Err(CelError::argument("string() expects exactly one argument"));
    }

    let arg_type = args[0].as_type();

    Ok(match &args[0] {
        Int(i) => i.to_string().into(),
        UInt(i) => i.to_string().into(),
        Float(f) => f.to_string().into(),
        String(s) => s.clone().into(),
        Bytes(b) => match std::string::String::from_utf8(b.clone()) {
            Ok(s) => s.into(),
            Err(_) => return Err(CelError::value("Bad bytes in utf8 convertion")),
        },
        TimeStamp(ts) => ts.to_rfc3339().into(),
        Duration(d) => d.to_string().into(),
        _ => {
            return Err(CelError::value(&format!(
                "string() invalid for {:?}",
                arg_type
            )))
        }
    })
}

fn type_impl(_this: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    if args.len() != 1 {
        return Err(CelError::argument("type() expects one argument"));
    }

    Ok(args[0].as_type())
}

fn timestamp_impl(_this: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    if args.len() != 1 {
        return Err(CelError::argument("timestamp() expect one argument"));
    }

    if let CelValue::String(str_val) = &args[0] {
        match (&str_val).parse::<DateTime<Utc>>() {
            Ok(val) => Ok(CelValue::from_timestamp(&val)),
            Err(_) => Err(CelError::value("Invalid timestamp format")),
        }
    } else {
        Err(CelError::value("timestamp() expects a string argument"))
    }
}

fn duration_impl(_this: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    if args.len() != 1 {
        return Err(CelError::argument("duration() expects one argument"));
    }

    if let CelValue::String(str_val) = &args[0] {
        match duration_str::parse_chrono(str_val) {
            Ok(val) => Ok(CelValue::from_duration(&val)),
            Err(_) => Err(CelError::value("Invalid duration format")),
        }
    } else {
        Err(CelError::value("duration() expects a string argument"))
    }
}
