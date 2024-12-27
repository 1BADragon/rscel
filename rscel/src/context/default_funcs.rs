use std::str::FromStr;

use super::bind_context::RsCelFunction;
use crate::{BindContext, CelError, CelResult, CelValue};
use chrono::{DateTime, Datelike, Timelike};
use chrono_tz::Tz;
use regex::Regex;

mod size;

const DEFAULT_FUNCS: &[(&str, &'static RsCelFunction)] = &[
    ("contains", &contains_impl),
    ("containsI", &contains_i_impl),
    ("size", &size::size),
    ("startsWith", &starts_with_impl),
    ("endsWith", &ends_with_impl),
    ("matches", &matches_impl),
    ("startsWithI", &starts_with_i_impl),
    ("endsWithI", &ends_with_i_impl),
    ("toLower", &to_lower_impl),
    ("toUpper", &to_upper_impl),
    ("trim", &trim_impl),
    ("trimStart", &trim_start_impl),
    ("trimEnd", &trim_end_impl),
    ("splitWhiteSpace", &split_whitespace_impl),
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
    ("getDayOfWeek", &get_day_of_week_impl),
    ("getDayOfYear", &get_day_of_year_impl),
    ("getFullYear", &get_full_year_impl),
    ("getHours", &get_hours_impl),
    ("getMilliseconds", &get_milliseconds_impl),
    ("getMinutes", &get_minutes_impl),
    ("getMonth", &get_month_impl),
    ("getSeconds", &get_seconds_impl),
];

macro_rules! string_func {
    ($cel_func_name: ident, $func_name:ident, $str_func:ident) => {
        fn $func_name(this: CelValue, args: Vec<CelValue>) -> CelValue {
            if args.len() > 0 {
                return CelValue::from_err(CelError::argument(
                    "$cel_func_name does not take any argments",
                ));
            }

            if let CelValue::String(s) = this {
                CelValue::String(s.$str_func().chars().collect())
            } else {
                return CelValue::from_err(CelError::value(
                    "$cel_func_name only available on string",
                ));
            }
        }
    };
}

pub fn load_default_funcs(exec_ctx: &mut BindContext) {
    for (name, func) in DEFAULT_FUNCS.iter() {
        exec_ctx.bind_func(name, *func);
    }
}

fn contains_impl(this: CelValue, args: Vec<CelValue>) -> CelValue {
    if args.len() != 1 {
        return CelValue::from_err(CelError::argument(
            "contains() expects exactly one argument",
        ));
    }

    if let CelValue::String(this_str) = this {
        if let CelValue::String(rhs) = &args[0] {
            CelValue::from_bool(this_str.contains(rhs))
        } else {
            CelValue::from_err(CelError::value("contains() arg must be string"))
        }
    } else {
        CelValue::from_err(CelError::value("contains() can only operate on string"))
    }
}

fn contains_i_impl(this: CelValue, args: Vec<CelValue>) -> CelValue {
    if args.len() != 1 {
        return CelValue::from_err(CelError::argument(
            "contains() expects exactly one argument",
        ));
    }

    if let CelValue::String(this_str) = this {
        if let CelValue::String(rhs) = &args[0] {
            CelValue::from_bool(this_str.to_lowercase().contains(&rhs.to_lowercase()))
        } else {
            CelValue::from_err(CelError::value("containsI() arg must be string"))
        }
    } else {
        CelValue::from_err(CelError::value("containsI() can only operate on string"))
    }
}

fn matches_impl(this: CelValue, args: Vec<CelValue>) -> CelValue {
    let (vc_lhs, vc_rhs) = if let CelValue::Null = this {
        if args.len() != 2 {
            return CelValue::from_err(CelError::argument(
                "matches() expects exactly two argument",
            ));
        }
        (&args[0], &args[1])
    } else {
        if args.len() != 1 {
            return CelValue::from_err(CelError::argument(
                "matches() expects exactly one argument",
            ));
        }
        (&this, &args[0])
    };

    if let CelValue::String(lhs) = vc_lhs {
        if let CelValue::String(rhs) = vc_rhs {
            match Regex::new(rhs) {
                Ok(re) => return re.is_match(lhs).into(),
                Err(err) => {
                    return CelValue::from_err(CelError::value(&format!(
                        "Invalid regular expression: {}",
                        err
                    )))
                }
            }
        }
    }

    CelValue::from_err(CelError::value(
        "matches has the forms string.(string) or (string, string)",
    ))
}

fn starts_with_impl(this: CelValue, args: Vec<CelValue>) -> CelValue {
    if args.len() != 1 {
        return CelValue::from_err(CelError::argument(
            "endsWith() expects exactly one argument",
        ));
    }

    if let CelValue::String(lhs) = this {
        if let CelValue::String(rhs) = &args[0] {
            return lhs.starts_with(rhs).into();
        }
    }

    CelValue::from_err(CelError::value("endsWith must be form string.(string)"))
}

fn ends_with_impl(this: CelValue, args: Vec<CelValue>) -> CelValue {
    if args.len() != 1 {
        return CelValue::from_err(CelError::argument(
            "endsWith() expects exactly one argument",
        ));
    }

    if let CelValue::String(lhs) = this {
        if let CelValue::String(rhs) = &args[0] {
            return lhs.ends_with(rhs).into();
        }
    }

    CelValue::from_err(CelError::value("endsWith must be form string.(string)"))
}

fn starts_with_i_impl(this: CelValue, args: Vec<CelValue>) -> CelValue {
    if args.len() != 1 {
        return CelValue::from_err(CelError::argument(
            "endsWith() expects exactly one argument",
        ));
    }

    if let CelValue::String(lhs) = this {
        if let CelValue::String(rhs) = &args[0] {
            return lhs.to_lowercase().starts_with(&rhs.to_lowercase()).into();
        }
    }

    CelValue::from_err(CelError::value("endsWith must be form string.(string)"))
}

fn ends_with_i_impl(this: CelValue, args: Vec<CelValue>) -> CelValue {
    if args.len() != 1 {
        return CelValue::from_err(CelError::argument(
            "endsWith() expects exactly one argument",
        ));
    }

    if let CelValue::String(lhs) = this {
        if let CelValue::String(rhs) = &args[0] {
            return lhs.to_lowercase().ends_with(&rhs.to_lowercase()).into();
        }
    }

    CelValue::from_err(CelError::value("endsWith must be form string.(string)"))
}

string_func!(toLower, to_lower_impl, to_lowercase);
string_func!(toUpper, to_upper_impl, to_uppercase);
string_func!(trim, trim_impl, trim);
string_func!(trimStart, trim_start_impl, trim_start);
string_func!(trimEnd, trim_end_impl, trim_end);

fn split_whitespace_impl(this: CelValue, args: Vec<CelValue>) -> CelValue {
    if args.len() != 0 {
        return CelValue::from_err(CelError::argument(
            "split_whitespace() expects no arguments",
        ));
    }

    if let CelValue::String(s) = this {
        s.split_whitespace()
            .map(|t| t.into())
            .collect::<Vec<CelValue>>()
            .into()
    } else {
        CelValue::from_err(CelError::argument("split_whitespace() expects string"))
    }
}

fn abs_impl(_this: CelValue, args: Vec<CelValue>) -> CelValue {
    if args.len() != 1 {
        return CelValue::from_err(CelError::argument("abs() expects one argument"));
    }

    match args[0] {
        CelValue::Int(i) => i.abs().into(),
        CelValue::UInt(u) => (u).into(),
        CelValue::Float(f) => f.abs().into(),
        _ => CelValue::from_err(CelError::value("abs() expect numerical argument")),
    }
}

fn sqrt_impl(_this: CelValue, args: Vec<CelValue>) -> CelValue {
    if args.len() != 1 {
        return CelValue::from_err(CelError::argument("sqrt() expects one argument"));
    }

    match args[0] {
        CelValue::Float(f) => f.sqrt().into(),
        _ => CelValue::from_err(CelError::value("abs() expect double argument")),
    }
}

fn pow_impl(_this: CelValue, args: Vec<CelValue>) -> CelValue {
    if args.len() != 2 {
        return CelValue::from_err(CelError::argument("pow() expects two argument"));
    }

    match args[0] {
        CelValue::Int(i) => match args[1] {
            CelValue::Int(e) => i.pow(e as u32).into(),
            CelValue::UInt(e) => i.pow(e as u32).into(),
            _ => CelValue::from_err(CelError::argument("pow() expects integer exponent")),
        },
        CelValue::UInt(u) => match args[1] {
            CelValue::Int(e) => u.pow(e as u32).into(),
            CelValue::UInt(e) => u.pow(e as u32).into(),
            _ => CelValue::from_err(CelError::argument("pow() expects integer exponent")),
        },
        CelValue::Float(f) => match args[1] {
            CelValue::Int(e) => f.powi(e as i32).into(),
            CelValue::UInt(e) => f.powi(e as i32).into(),
            CelValue::Float(e) => f.powf(e).into(),
            _ => CelValue::from_err(CelError::argument(
                "pow() expect integer or float for exponent",
            )),
        },
        _ => CelValue::from_err(CelError::value("abs() expect numerical argument")),
    }
}

fn log_impl(_this: CelValue, args: Vec<CelValue>) -> CelValue {
    if args.len() != 1 {
        return CelValue::from_err(CelError::argument("log() expects one argument"));
    }

    match args[0] {
        CelValue::Int(i) => i.ilog10().into(),
        CelValue::UInt(u) => u.ilog10().into(),
        CelValue::Float(f) => f.log10().into(),
        _ => CelValue::from_err(CelError::value("log() expects numerical argument")),
    }
}

fn ceil_impl(_this: CelValue, args: Vec<CelValue>) -> CelValue {
    if args.len() != 1 {
        return CelValue::from_err(CelError::argument("ceil() expects on argument"));
    }

    match args[0] {
        CelValue::Int(i) => (i).into(),
        CelValue::UInt(u) => (u).into(),
        CelValue::Float(f) => (f.ceil() as i64).into(),
        _ => CelValue::from_err(CelError::argument("ceil() expects numeric type")),
    }
}

fn floor_impl(_this: CelValue, args: Vec<CelValue>) -> CelValue {
    if args.len() != 1 {
        return CelValue::from_err(CelError::argument("floor() expects on argument"));
    }

    match args[0] {
        CelValue::Int(i) => (i).into(),
        CelValue::UInt(u) => (u).into(),
        CelValue::Float(f) => (f.floor() as i64).into(),
        _ => CelValue::from_err(CelError::argument("floor() expects numeric type")),
    }
}

fn round_impl(_this: CelValue, args: Vec<CelValue>) -> CelValue {
    if args.len() != 1 {
        return CelValue::from_err(CelError::argument("round() expects on argument"));
    }

    match args[0] {
        CelValue::Int(i) => (i).into(),
        CelValue::UInt(u) => (u).into(),
        CelValue::Float(f) => (f.round() as i64).into(),
        _ => CelValue::from_err(CelError::argument("round() expects numeric type")),
    }
}

fn min_impl(_this: CelValue, args: Vec<CelValue>) -> CelValue {
    if args.len() == 0 {
        return CelValue::from_err(CelError::argument("min() requires at lease one argument"));
    }

    let mut curr_min: Option<&CelValue> = None;

    for val in args.iter() {
        match curr_min {
            Some(curr) => {
                if val.clone().lt(curr.clone()).is_true() {
                    curr_min = Some(&val);
                }
            }
            None => curr_min = Some(&val),
        }
    }

    match curr_min {
        Some(v) => v.clone(),
        None => CelValue::from_null(),
    }
}

fn max_impl(_this: CelValue, args: Vec<CelValue>) -> CelValue {
    if args.len() == 0 {
        return CelValue::from_err(CelError::argument("max() requires at lease one argument"));
    }

    let mut curr_max: Option<&CelValue> = None;

    for val in args.iter() {
        match curr_max {
            Some(curr) => {
                if val.clone().gt(curr.clone()).is_true() {
                    curr_max = Some(val);
                }
            }
            None => curr_max = Some(val),
        }
    }

    match curr_max {
        Some(v) => v.clone(),
        None => CelValue::from_null(),
    }
}

fn get_adjusted_datetime(this: CelValue, args: Vec<CelValue>) -> CelResult<DateTime<Tz>> {
    if let CelValue::TimeStamp(ts) = this {
        if args.len() == 0 {
            return Ok(ts.with_timezone(&Tz::UTC));
        } else if args.len() == 1 {
            if let CelValue::String(ref s) = args[0] {
                if let Ok(tz) = Tz::from_str(s) {
                    Ok(ts.with_timezone(&tz))
                } else {
                    Err(CelError::argument("Failed to parse timezone"))
                }
            } else {
                Err(CelError::argument("Argument must be a string"))
            }
        } else {
            Err(CelError::argument("Expected either 0 or 1 argumnets"))
        }
    } else if let CelValue::Err(e) = this {
        Err(e)
    } else {
        Err(CelError::argument("First parameter is not a timestamp"))
    }
}

fn get_date_impl(this: CelValue, args: Vec<CelValue>) -> CelValue {
    let date = match get_adjusted_datetime(this, args) {
        Ok(it) => it,
        Err(err) => return err.into(),
    };

    let day_of_month = date.day();
    CelValue::from_int(day_of_month as i64)
}

fn get_day_of_month_impl(this: CelValue, args: Vec<CelValue>) -> CelValue {
    let date = match get_adjusted_datetime(this, args) {
        Ok(it) => it,
        Err(err) => return err.into(),
    };

    let day_of_month = date.day() - 1;
    CelValue::from_int(day_of_month as i64)
}

fn get_day_of_week_impl(this: CelValue, args: Vec<CelValue>) -> CelValue {
    let date = match get_adjusted_datetime(this, args) {
        Ok(it) => it,
        Err(err) => return err.into(),
    };

    let day_of_week = date.weekday().number_from_sunday() - 1;
    CelValue::from_int(day_of_week as i64)
}

fn get_day_of_year_impl(this: CelValue, args: Vec<CelValue>) -> CelValue {
    let date = match get_adjusted_datetime(this, args) {
        Ok(it) => it,
        Err(err) => return err.into(),
    };

    let day_of_year = date.ordinal() - 1;
    CelValue::from_int(day_of_year as i64)
}

fn get_full_year_impl(this: CelValue, args: Vec<CelValue>) -> CelValue {
    let date = match get_adjusted_datetime(this, args) {
        Ok(it) => it,
        Err(err) => return err.into(),
    };

    let year = date.year();
    CelValue::from_int(year as i64)
}

fn get_hours_impl(this: CelValue, args: Vec<CelValue>) -> CelValue {
    match this {
        CelValue::Duration(d) => d.num_hours().into(),
        other => {
            let date = match get_adjusted_datetime(other, args) {
                Ok(it) => it,
                Err(err) => return err.into(),
            };

            let hours = date.time().hour();
            CelValue::from_int(hours as i64)
        }
    }
}

fn get_milliseconds_impl(this: CelValue, args: Vec<CelValue>) -> CelValue {
    match this {
        CelValue::Duration(d) => (d.subsec_nanos() / 1000000).into(),
        other => {
            let date = match get_adjusted_datetime(other, args) {
                Ok(it) => it,
                Err(err) => return err.into(),
            };

            let milliseconds = date.timestamp_subsec_millis();
            CelValue::from_int(milliseconds as i64)
        }
    }
}

fn get_minutes_impl(this: CelValue, args: Vec<CelValue>) -> CelValue {
    match this {
        CelValue::Duration(d) => d.num_minutes().into(),
        other => {
            let date = match get_adjusted_datetime(other, args) {
                Ok(it) => it,
                Err(err) => return err.into(),
            };

            let minutes = date.time().minute();
            CelValue::from_int(minutes as i64)
        }
    }
}

fn get_month_impl(this: CelValue, args: Vec<CelValue>) -> CelValue {
    let date = match get_adjusted_datetime(this, args) {
        Ok(it) => it,
        Err(err) => return err.into(),
    };

    let month = date.month0();
    CelValue::from_int(month as i64)
}

fn get_seconds_impl(this: CelValue, args: Vec<CelValue>) -> CelValue {
    match this {
        CelValue::Duration(d) => d.num_seconds().into(),
        other => {
            let date = match get_adjusted_datetime(other, args) {
                Ok(it) => it,
                Err(err) => return err.into(),
            };

            let second = date.time().second();
            CelValue::from_int(second as i64)
        }
    }
}
