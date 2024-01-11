use std::str::FromStr;

use super::bind_context::RsCelFunction;
use crate::{
    cel_error::{CelError, CelResult},
    BindContext, CelValue,
};
use chrono::{DateTime, Datelike, FixedOffset, TimeZone, Utc};
use chrono_tz::Tz;
use regex::Regex;

const DEFAULT_FUNCS: &[(&str, &'static RsCelFunction)] = &[
    ("bool", &bool_impl),
    ("int", &int_impl),
    ("uint", &uint_impl),
    ("double", &double_impl),
    ("bytes", &bytes_impl),
    ("string", &string_impl),
    ("contains", &contains_impl),
    ("size", &size_impl),
    ("startsWith", &starts_with_impl),
    ("endsWith", &ends_with_impl),
    ("matches", &matches_impl),
    ("type", &type_impl),
    ("timestamp", &timestamp_impl),
    ("duration", &duration_impl),
    ("abs", &abs_impl),
    ("sqrt", &sqrt_impl),
    ("pow", &pow_impl),
    ("log", &log_impl),
    ("ceil", &ceil_impl),
    ("floor", &floor_impl),
    ("round", &round_impl),
    ("min", &min_impl),
    ("max", &max_impl),
    ("getDate", &get_date_impl),
    ("getDayOfMonth", &get_day_of_month_impl),
];

pub fn load_default_funcs(exec_ctx: &mut BindContext) {
    for (name, func) in DEFAULT_FUNCS.iter() {
        exec_ctx.bind_func(name, *func);
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

fn contains_impl(this: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    if args.len() != 1 {
        return Err(CelError::argument(
            "contains() expects exactly one argument",
        ));
    }

    if let CelValue::String(this_str) = this {
        if let CelValue::String(rhs) = &args[0] {
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

    Ok(CelValue::from_uint(match &args[0] {
        CelValue::String(s) => s.len() as u64,
        CelValue::Bytes(b) => b.len() as u64,
        CelValue::List(l) => l.len() as u64,
        CelValue::Map(m) => m.len() as u64,
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

    if let CelValue::String(lhs) = this {
        if let CelValue::String(rhs) = &args[0] {
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

    if let CelValue::String(lhs) = this {
        if let CelValue::String(rhs) = &args[0] {
            return Ok(lhs.ends_with(rhs).into());
        }
    }

    Err(CelError::value("endsWith must be form string.(string)"))
}

fn matches_impl(this: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    let (vc_lhs, vc_rhs) = if let CelValue::Null = this {
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

    if let CelValue::String(lhs) = vc_lhs {
        if let CelValue::String(rhs) = vc_rhs {
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

fn abs_impl(_this: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    if args.len() != 1 {
        return Err(CelError::argument("abs() expects one argument"));
    }

    match args[0] {
        CelValue::Int(i) => Ok(i.abs().into()),
        CelValue::UInt(u) => Ok((u).into()),
        CelValue::Float(f) => Ok(f.abs().into()),
        _ => Err(CelError::value("abs() expect numerical argument")),
    }
}

fn sqrt_impl(_this: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    if args.len() != 1 {
        return Err(CelError::argument("sqrt() expects one argument"));
    }

    match args[0] {
        CelValue::Float(f) => Ok(f.sqrt().into()),
        _ => Err(CelError::value("abs() expect double argument")),
    }
}

fn pow_impl(_this: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    if args.len() != 2 {
        return Err(CelError::argument("pow() expects two argument"));
    }

    match args[0] {
        CelValue::Int(i) => match args[1] {
            CelValue::Int(e) => Ok(i.pow(e as u32).into()),
            CelValue::UInt(e) => Ok(i.pow(e as u32).into()),
            _ => Err(CelError::argument("pow() expects integer exponent")),
        },
        CelValue::UInt(u) => match args[1] {
            CelValue::Int(e) => Ok(u.pow(e as u32).into()),
            CelValue::UInt(e) => Ok(u.pow(e as u32).into()),
            _ => Err(CelError::argument("pow() expects integer exponent")),
        },
        CelValue::Float(f) => match args[1] {
            CelValue::Int(e) => Ok(f.powi(e as i32).into()),
            CelValue::UInt(e) => Ok(f.powi(e as i32).into()),
            CelValue::Float(e) => Ok(f.powf(e).into()),
            _ => Err(CelError::argument(
                "pow() expect integer or float for exponent",
            )),
        },
        _ => Err(CelError::value("abs() expect numerical argument")),
    }
}

fn log_impl(_this: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    if args.len() != 1 {
        return Err(CelError::argument("log() expects one argument"));
    }

    match args[0] {
        CelValue::Int(i) => Ok(i.ilog10().into()),
        CelValue::UInt(u) => Ok(u.ilog10().into()),
        CelValue::Float(f) => Ok(f.log10().into()),
        _ => Err(CelError::value("log() expects numerical argument")),
    }
}

fn ceil_impl(_this: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    if args.len() != 1 {
        return Err(CelError::argument("ceil() expects on argument"));
    }

    match args[0] {
        CelValue::Int(i) => Ok((i).into()),
        CelValue::UInt(u) => Ok((u).into()),
        CelValue::Float(f) => Ok((f.ceil() as i64).into()),
        _ => Err(CelError::argument("ceil() expects numeric type")),
    }
}

fn floor_impl(_this: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    if args.len() != 1 {
        return Err(CelError::argument("floor() expects on argument"));
    }

    match args[0] {
        CelValue::Int(i) => Ok((i).into()),
        CelValue::UInt(u) => Ok((u).into()),
        CelValue::Float(f) => Ok((f.floor() as i64).into()),
        _ => Err(CelError::argument("floor() expects numeric type")),
    }
}

fn round_impl(_this: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    if args.len() != 1 {
        return Err(CelError::argument("round() expects on argument"));
    }

    match args[0] {
        CelValue::Int(i) => Ok((i).into()),
        CelValue::UInt(u) => Ok((u).into()),
        CelValue::Float(f) => Ok((f.round() as i64).into()),
        _ => Err(CelError::argument("round() expects numeric type")),
    }
}

fn min_impl(_this: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    if args.len() == 0 {
        return Err(CelError::argument("min() requires at lease one argument"));
    }

    let mut curr_min = None;

    for val in args.into_iter() {
        match curr_min {
            Some(curr) => {
                if val.lt(curr)?.is_true() {
                    curr_min = Some(val);
                }
            }
            None => curr_min = Some(val),
        }
    }

    return Ok(curr_min.unwrap().clone());
}

fn max_impl(_this: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    if args.len() == 0 {
        return Err(CelError::argument("max() requires at lease one argument"));
    }

    let mut curr_min = None;

    for val in args.into_iter() {
        match curr_min {
            Some(curr) => {
                if val.gt(curr)?.is_true() {
                    curr_min = Some(val);
                }
            }
            None => curr_min = Some(val),
        }
    }

    return Ok(curr_min.unwrap().clone());
}

fn get_adjusted_datetime(this: CelValue, args: &[CelValue]) -> CelResult<DateTime<FixedOffset>> {
    if let CelValue::TimeStamp(ts) = this {
        if args.len() == 0 {
            return Ok(ts.into());
        } else if args.len() == 1 {
            if let CelValue::String(s) = args[0] {
                if let Ok(tz) = Tz::from_str(&s) {
                    Ok(ts.with_timezone(tz.offset_from_utc_date()))
                } else {
                    Err(CelError::argument("Failed to parse timezone"))
                }
            } else {
                Err(CelError::argument("Argument must be a string"))
            }
        } else {
            Err(CelError::argument("Expected either 0 or 1 argumnets"))
        }
    } else {
        Err(CelError::argument("First parameter is not a timestamp"))
    }
}

fn get_date_impl(this: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    let date = get_adjusted_datetime(this, args)?;

    let day_of_month = date.day();
    Ok(CelValue::from_int(day_of_month as i64))
}

fn get_day_of_month_impl(this: CelValue, args: &[CelValue]) -> CelResult<CelValue> {
    let date = get_adjusted_datetime(this, args)?;

    let day_of_month = date.day() - 1;
    Ok(CelValue::from_int(day_of_month as i64))
}
