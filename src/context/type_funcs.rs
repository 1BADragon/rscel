use chrono::{DateTime, Utc};

use crate::{BindContext, CelError, CelValue, CelValueDyn};

pub fn construct_type(type_name: &str, args: &[CelValue]) -> CelValue {
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
    bind_ctx.add_type("type", CelValue::type_type());
    bind_ctx.add_type("timestamp", CelValue::timestamp_type());
    bind_ctx.add_type("duration", CelValue::duration_type());
    bind_ctx.add_type("null_type", CelValue::null_type());
    bind_ctx.add_type("dyn", CelValue::dyn_type())
}

fn bool_impl(_: CelValue, args: &[CelValue]) -> CelValue {
    if args.len() != 1 {
        return CelValue::from_err(CelError::argument("bool() expects exactly one argument"));
    }

    args[0].is_truthy().into()
}

fn int_impl(_: CelValue, args: &[CelValue]) -> CelValue {
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

fn uint_impl(_: CelValue, args: &[CelValue]) -> CelValue {
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

fn double_impl(_: CelValue, args: &[CelValue]) -> CelValue {
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

fn bytes_impl(_this: CelValue, args: &[CelValue]) -> CelValue {
    use CelValue::*;
    if args.len() != 1 {
        return CelValue::from_err(CelError::argument("bytes() expects exactly one argument"));
    }

    match &args[0] {
        String(val) => CelValue::from_bytes(val.as_bytes().to_vec()),
        other => CelValue::from_err(CelError::value(&format!(
            "int conversion invalid for {:?}",
            other
        ))),
    }
}

fn string_impl(_this: CelValue, args: &[CelValue]) -> CelValue {
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

fn type_impl(_this: CelValue, args: &[CelValue]) -> CelValue {
    if args.len() != 1 {
        return CelValue::from_err(CelError::argument("type() expects one argument"));
    }

    args[0].as_type()
}

fn timestamp_impl(_this: CelValue, args: &[CelValue]) -> CelValue {
    if args.len() != 1 {
        return CelValue::from_err(CelError::argument("timestamp() expect one argument"));
    }

    if let CelValue::String(str_val) = &args[0] {
        match (&str_val).parse::<DateTime<Utc>>() {
            Ok(val) => CelValue::from_timestamp(&val),
            Err(_) => CelValue::from_err(CelError::value("Invalid timestamp format")),
        }
    } else {
        CelValue::from_err(CelError::value("timestamp() expects a string argument"))
    }
}

fn duration_impl(_this: CelValue, args: &[CelValue]) -> CelValue {
    if args.len() != 1 {
        return CelValue::from_err(CelError::argument("duration() expects one argument"));
    }

    if let CelValue::String(str_val) = &args[0] {
        match duration_str::parse_chrono(str_val) {
            Ok(val) => CelValue::from_duration(&val),
            Err(_) => CelValue::from_err(CelError::value("Invalid duration format")),
        }
    } else {
        CelValue::from_err(CelError::value("duration() expects a string argument"))
    }
}

fn dyn_impl(_this: CelValue, args: &[CelValue]) -> CelValue {
    if args.len() != 1 {
        return CelValue::from_err(CelError::argument("dyn() expects one argument"));
    }

    args[0].clone()
}
