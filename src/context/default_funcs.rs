use super::bind_context::RsCelFunction;
use crate::{
    cel_error::{CelError, CelResult},
    BindContext, CelValue, CelValueInner,
};
use chrono::{DateTime, Utc};
use regex::Regex;

const DEFAULT_FUNCS: &[(&str, RsCelFunction)] = &[
    ("int", int_impl),
    ("uint", uint_impl),
    ("double", double_impl),
    ("bytes", bytes_impl),
    ("string", string_impl),
    ("contains", contains_impl),
    ("size", size_impl),
    ("startsWith", starts_with_impl),
    ("endsWith", ends_with_impl),
    ("matches", matches_impl),
    ("type", type_impl),
    ("timestamp", timestamp_impl),
    ("duration", duration_impl),
    ("abs", abs_impl),
    ("sqrt", sqrt_impl),
    ("pow", pow_impl),
];

pub fn load_default_funcs(exec_ctx: &mut BindContext) {
    for (name, func) in DEFAULT_FUNCS.iter() {
        exec_ctx.bind_func(name, *func);
    }
}

fn int_impl(_: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    use CelValueInner::*;

    if args.len() != 1 {
        return Err(CelError::argument("int() expects exactly one argument"));
    }

    match args[0].inner() {
        Int(val) => Ok(CelValue::from_int(*val)),
        UInt(val) => Ok(CelValue::from_int(*val as i64)),
        Float(val) => Ok(CelValue::from_int(*val as i64)),
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
    use CelValueInner::*;

    if args.len() != 1 {
        return Err(CelError::argument("uint() expects exactly one argument"));
    }

    match args[0].inner() {
        Int(val) => Ok(CelValue::from_uint(*val as u64)),
        UInt(val) => Ok(CelValue::from_uint(*val)),
        Float(val) => Ok(CelValue::from_uint(*val as u64)),
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
    use CelValueInner::*;

    if args.len() != 1 {
        return Err(CelError::argument("double() expects exactly one argument"));
    }

    match args[0].inner() {
        Int(val) => Ok(CelValue::from_float(*val as f64)),
        UInt(val) => Ok(CelValue::from_float(*val as f64)),
        Float(val) => Ok(CelValue::from_float(*val)),
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
    use CelValueInner::*;
    if args.len() != 1 {
        return Err(CelError::argument("bytes() expects exactly one argument"));
    }

    match &args[0].inner() {
        String(val) => Ok(CelValue::from_bytes(val.as_bytes().to_vec())),
        other => Err(CelError::value(&format!(
            "int conversion invalid for {:?}",
            other
        ))),
    }
}

fn string_impl(_this: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    use CelValueInner::*;

    if args.len() != 1 {
        return Err(CelError::argument("string() expects exactly one argument"));
    }

    let arg_type = args[0].as_type();

    Ok(match args[0].inner() {
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

fn contains_impl(this: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    if args.len() != 1 {
        return Err(CelError::argument(
            "contains() expects exactly one argument",
        ));
    }

    if let CelValueInner::String(this_str) = this.into_inner() {
        if let CelValueInner::String(rhs) = args[0].inner() {
            Ok(CelValue::from_bool(this_str.contains(rhs)))
        } else {
            Err(CelError::value("contains() arg must be string"))
        }
    } else {
        Err(CelError::value("contains() can only operate on string"))
    }
}

fn size_impl(_this: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    if args.len() != 1 {
        return Err(CelError::argument("size() expects exactly one argument"));
    }

    Ok(CelValue::from_uint(match args[0].inner() {
        CelValueInner::String(s) => s.len() as u64,
        CelValueInner::Bytes(b) => b.len() as u64,
        CelValueInner::List(l) => l.len() as u64,
        CelValueInner::Map(m) => m.len() as u64,
        _ => {
            return Err(CelError::value(
                "size() only available for types {string, bytes, list, map}",
            ))
        }
    }))
}

fn starts_with_impl(this: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    if args.len() != 1 {
        return Err(CelError::argument(
            "endsWith() expects exactly one argument",
        ));
    }

    if let CelValueInner::String(lhs) = this.inner() {
        if let CelValueInner::String(rhs) = args[0].inner() {
            return Ok(lhs.starts_with(rhs).into());
        }
    }

    Err(CelError::value("endsWith must be form string.(string)"))
}

fn ends_with_impl(this: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    if args.len() != 1 {
        return Err(CelError::argument(
            "endsWith() expects exactly one argument",
        ));
    }

    if let CelValueInner::String(lhs) = this.inner() {
        if let CelValueInner::String(rhs) = args[0].inner() {
            return Ok(lhs.ends_with(rhs).into());
        }
    }

    Err(CelError::value("endsWith must be form string.(string)"))
}

fn matches_impl(this: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    let (vc_lhs, vc_rhs) = if let CelValueInner::Null = this.inner() {
        if args.len() != 2 {
            return Err(CelError::argument("matches() expects exactly two argument"));
        }
        (&args[0], &args[1])
    } else {
        if args.len() != 1 {
            return Err(CelError::argument("matches() expects exactly one argument"));
        }
        (&this, &args[0])
    };

    if let CelValueInner::String(lhs) = vc_lhs.inner() {
        if let CelValueInner::String(rhs) = vc_rhs.inner() {
            match Regex::new(rhs) {
                Ok(re) => return Ok(re.is_match(lhs).into()),
                Err(err) => {
                    return Err(CelError::value(&format!(
                        "Invalid regular expression: {}",
                        err
                    )))
                }
            }
        }
    }

    Err(CelError::value(
        "matches has the forms string.(string) or (string, string)",
    ))
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

    if let CelValueInner::String(str_val) = args[0].inner() {
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

    if let CelValueInner::String(str_val) = args[0].inner() {
        match duration_str::parse_chrono(str_val) {
            Ok(val) => Ok(CelValue::from_duration(&val)),
            Err(_) => Err(CelError::value("Invalid duration format")),
        }
    } else {
        Err(CelError::value("duration() expects a string argument"))
    }
}

fn abs_impl(_this: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    if args.len() != 1 {
        return Err(CelError::argument("abs() expects one argument"));
    }

    match args[0].inner() {
        CelValueInner::Int(i) => Ok(i.abs().into()),
        CelValueInner::UInt(u) => Ok((*u).into()),
        CelValueInner::Float(f) => Ok(f.abs().into()),
        _ => Err(CelError::value("abs() expect numerical argument")),
    }
}

fn sqrt_impl(_this: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    if args.len() != 1 {
        return Err(CelError::argument("sqrt() expects one argument"));
    }

    match args[0].inner() {
        CelValueInner::Float(f) => Ok(f.sqrt().into()),
        _ => Err(CelError::value("abs() expect double argument")),
    }
}

fn pow_impl(_this: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    if args.len() != 2 {
        return Err(CelError::argument("pow() expects two argument"));
    }

    match args[0].inner() {
        CelValueInner::Int(i) => match args[1].inner() {
            CelValueInner::Int(e) => Ok(i.pow(*e as u32).into()),
            CelValueInner::UInt(e) => Ok(i.pow(*e as u32).into()),
            _ => Err(CelError::argument("pow() expects integer exponent")),
        },
        CelValueInner::UInt(u) => match args[1].inner() {
            CelValueInner::Int(e) => Ok(u.pow(*e as u32).into()),
            CelValueInner::UInt(e) => Ok(u.pow(*e as u32).into()),
            _ => Err(CelError::argument("pow() expects integer exponent")),
        },
        CelValueInner::Float(f) => match args[1].inner() {
            CelValueInner::Int(e) => Ok(f.powi(*e as i32).into()),
            CelValueInner::UInt(e) => Ok(f.powi(*e as i32).into()),
            CelValueInner::Float(e) => Ok(f.powf(*e).into()),
            _ => Err(CelError::argument(
                "pow() expect integer or float for exponent",
            )),
        },
        _ => Err(CelError::value("abs() expect numerical argument")),
    }
}
