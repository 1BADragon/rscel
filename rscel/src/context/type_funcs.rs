use chrono::{DateTime, Duration, TimeZone, Utc};

use crate::{BindContext, CelError, CelValue, CelValueDyn};

pub fn construct_type(type_name: &str, args: Vec<CelValue>) -> CelValue {
    match type_name {
        "bool" => bool_impl(CelValue::from_null(), args),
        "int" => int_impl(CelValue::from_null(), args),
        "uint" => uint_impl(CelValue::from_null(), args),
        "float" => double_impl(CelValue::from_null(), args),
        "double" => double_impl(CelValue::from_null(), args),
        "bytes" => bytes_impl(CelValue::from_null(), args),
        "string" => string_impl(CelValue::from_null(), args),
        "type" => type_impl(CelValue::from_null(), args),
        "timestamp" => timestamp_impl(CelValue::from_null(), args),
        "duration" => duration_impl(CelValue::from_null(), args),
        "dyn" => dyn_impl(CelValue::from_null(), args),
        _ => CelValue::from_err(CelError::runtime(&format!(
            "{} is not constructable",
            type_name
        ))),
    }
}

pub fn load_default_types(bind_ctx: &mut BindContext) {
    bind_ctx.add_type("bool", CelValue::bool_type());
    bind_ctx.add_type("int", CelValue::int_type());
    bind_ctx.add_type("uint", CelValue::uint_type());
    bind_ctx.add_type("float", CelValue::float_type());
    bind_ctx.add_type("double", CelValue::float_type());
    bind_ctx.add_type("string", CelValue::string_type());
    bind_ctx.add_type("bytes", CelValue::bytes_type());
    bind_ctx.add_type("type", CelValue::type_type());
    bind_ctx.add_type("timestamp", CelValue::timestamp_type());
    bind_ctx.add_type("duration", CelValue::duration_type());
    bind_ctx.add_type("null_type", CelValue::null_type());
    bind_ctx.add_type("dyn", CelValue::dyn_type())
}

fn bool_impl(_: CelValue, args: Vec<CelValue>) -> CelValue {
    if args.len() != 1 {
        return CelValue::from_err(CelError::argument("bool() expects exactly one argument"));
    }

    let arg = args.into_iter().next().unwrap();

    match arg {
        CelValue::String(s) => match s.as_str() {
            "1" => true.into(),
            "t" => true.into(),
            "true" => true.into(),
            "TRUE" => true.into(),
            "True" => true.into(),
            "0" => false.into(),
            "f" => false.into(),
            "false" => false.into(),
            "FALSE" => false.into(),
            "False" => false.into(),
            s => {
                if cfg!(feature = "type_prop") {
                    (!s.is_empty()).into()
                } else {
                    CelValue::from_err(CelError::Value(format!(
                        "value '{}' cannot be converted to bool",
                        s
                    )))
                }
            }
        },
        CelValue::Bool(b) => b.into(),
        _ => {
            if cfg!(feature = "type_prop") {
                arg.is_truthy().into()
            } else {
                CelValue::from_err(CelError::Value(format!(
                    "type {} cannot be converted to bool",
                    arg.as_type()
                )))
            }
        }
    }
}

fn int_impl(_: CelValue, args: Vec<CelValue>) -> CelValue {
    use CelValue::*;

    if args.len() != 1 {
        return CelValue::from_err(CelError::argument("int() expects exactly one argument"));
    }

    match &args[0] {
        Int(val) => CelValue::from_int(*val),
        UInt(val) => CelValue::from_int(*val as i64),
        Float(val) => CelValue::from_int(*val as i64),
        Bool(val) => CelValue::from_int(if *val { 1 } else { 0 }),
        String(val) => match val.parse::<i64>() {
            Ok(res) => CelValue::from_int(res),
            Result::Err(_) => CelValue::from_err(CelError::value(&format!(
                "int conversion invalid for \"{}\"",
                val
            ))),
        },
        TimeStamp(val) => CelValue::from_int(val.timestamp()),
        other => CelValue::from_err(CelError::value(&format!(
            "int conversion invalid for {:?}",
            other
        ))),
    }
}

fn uint_impl(_: CelValue, args: Vec<CelValue>) -> CelValue {
    use CelValue::*;

    if args.len() != 1 {
        return CelValue::from_err(CelError::argument("uint() expects exactly one argument"));
    }

    match &args[0] {
        Int(val) => CelValue::from_uint(*val as u64),
        UInt(val) => CelValue::from_uint(*val),
        Float(val) => CelValue::from_uint(*val as u64),
        Bool(val) => CelValue::from_uint(if *val { 1 } else { 0 }),
        String(val) => match val.parse::<u64>() {
            Ok(res) => CelValue::from_uint(res),
            Result::Err(_err) => CelValue::from_err(CelError::value(&format!(
                "int conversion invalid for \"{}\"",
                val
            ))),
        },
        other => CelValue::from_err(CelError::value(&format!(
            "int conversion invalid for {:?}",
            other
        ))),
    }
}

fn double_impl(_: CelValue, args: Vec<CelValue>) -> CelValue {
    use CelValue::*;

    if args.len() != 1 {
        return CelValue::from_err(CelError::argument("double() expects exactly one argument"));
    }

    match &args[0] {
        Int(val) => CelValue::from_float(*val as f64),
        UInt(val) => CelValue::from_float(*val as f64),
        Float(val) => CelValue::from_float(*val),
        Bool(val) => CelValue::from_float(if *val { 1.0 } else { 0.0 }),
        String(val) => match val.parse::<f64>() {
            Ok(res) => CelValue::from_float(res),
            Result::Err(_err) => CelValue::from_err(CelError::value(&format!(
                "int conversion invalid for \"{}\"",
                val
            ))),
        },
        other => CelValue::from_err(CelError::value(&format!(
            "int conversion invalid for {:?}",
            other
        ))),
    }
}

fn bytes_impl(_this: CelValue, args: Vec<CelValue>) -> CelValue {
    use CelValue::*;
    if args.len() != 1 {
        return CelValue::from_err(CelError::argument("bytes() expects exactly one argument"));
    }

    let arg = args.into_iter().next().unwrap();

    match arg {
        String(val) => CelValue::from_bytes(val.as_bytes().to_vec()),
        Bytes(b) => CelValue::Bytes(b),
        other => CelValue::from_err(CelError::value(&format!(
            "int conversion invalid for {:?}",
            other
        ))),
    }
}

fn string_impl(_this: CelValue, args: Vec<CelValue>) -> CelValue {
    use CelValue::*;

    if args.len() != 1 {
        return CelValue::from_err(CelError::argument("string() expects exactly one argument"));
    }

    let arg_type = args[0].as_type();

    match &args[0] {
        Int(i) => i.to_string().into(),
        UInt(i) => i.to_string().into(),
        Float(f) => f.to_string().into(),
        String(s) => s.clone().into(),
        Bytes(b) => match std::string::String::from_utf8(b.clone()) {
            Ok(s) => s.into(),
            Result::Err(_) => CelValue::from_err(CelError::value("Bad bytes in utf8 convertion")),
        },
        TimeStamp(ts) => ts.to_rfc3339().into(),
        Duration(d) => d.to_string().into(),
        _ => {
            return CelValue::from_err(CelError::value(&format!(
                "string() invalid for {:?}",
                arg_type
            )))
        }
    }
}

fn type_impl(_this: CelValue, args: Vec<CelValue>) -> CelValue {
    if args.len() != 1 {
        return CelValue::from_err(CelError::argument("type() expects one argument"));
    }

    args[0].as_type()
}

fn timestamp_impl(_this: CelValue, args: Vec<CelValue>) -> CelValue {
    if args.len() != 1 {
        return CelValue::from_err(CelError::argument("timestamp() expect one argument"));
    }

    if let CelValue::String(str_val) = &args[0] {
        if let Ok(val) = str_val.parse::<DateTime<Utc>>() {
            CelValue::from_timestamp(val)
        } else if let Ok(val) = DateTime::parse_from_rfc2822(str_val) {
            CelValue::from_timestamp(val.to_utc())
        } else if let Ok(val) = DateTime::parse_from_rfc3339(str_val) {
            CelValue::from_timestamp(val.to_utc())
        } else {
            CelValue::from_err(CelError::value("Invalid timestamp format"))
        }
    } else if let CelValue::Int(i) = args[0] {
        use chrono::MappedLocalTime;
        match Utc.timestamp_opt(i, 0) {
            MappedLocalTime::Single(s) => CelValue::from_timestamp(s),
            _ => CelValue::from_err(CelError::value("Invalid timestamp value")),
        }
    } else if let CelValue::UInt(i) = args[0] {
        use chrono::MappedLocalTime;
        match Utc.timestamp_opt(i as i64, 0) {
            MappedLocalTime::Single(s) => CelValue::from_timestamp(s),
            _ => CelValue::from_err(CelError::value("Invalid timestamp value")),
        }
    } else if let CelValue::TimeStamp(ts) = args[0] {
        CelValue::from_timestamp(ts)
    } else {
        CelValue::from_err(CelError::value("timestamp() expects a string argument"))
    }
}

fn duration_impl(_this: CelValue, args: Vec<CelValue>) -> CelValue {
    if args.len() == 1 {
        match &args[0] {
            CelValue::String(str_val) => match duration_str::parse_chrono(str_val) {
                Ok(val) => CelValue::from_duration(val),
                Err(_) => CelValue::from_err(CelError::value("Invalid duration format")),
            },
            CelValue::Int(int_val) => match Duration::new(*int_val, 0) {
                Some(d) => d.into(),
                None => CelValue::from_err(CelError::value("Invalid argument for duration")),
            },
            CelValue::Duration(d) => CelValue::Duration(d.clone()),
            _ => CelValue::from_err(CelError::value("Duration expects either string or int")),
        }
    } else if args.len() == 2 {
        if let (CelValue::Int(sec), CelValue::Int(nsec)) = (&args[0], &args[1]) {
            match Duration::new(*sec, *nsec as u32) {
                Some(d) => d.into(),
                None => CelValue::from_err(CelError::value("Invalid argument for duration")),
            }
        } else {
            CelValue::from_err(CelError::value("Duration expects 2 ints"))
        }
    } else {
        CelValue::from_err(CelError::value("duration call not correct"))
    }
}

fn dyn_impl(_this: CelValue, args: Vec<CelValue>) -> CelValue {
    if args.len() != 1 {
        return CelValue::from_err(CelError::argument("dyn() expects one argument"));
    }

    args[0].clone()
}
