use super::bind_context::RsCelFunction;
use crate::{BindContext, CelError, CelValue};
use regex::Regex;

mod contains;
mod contains_i;
mod size;
mod starts_with;

mod time_funcs;

const DEFAULT_FUNCS: &[(&str, &'static RsCelFunction)] = &[
    ("contains", &contains::contains),
    ("containsI", &contains_i::contains_l),
    ("size", &size::size),
    ("startsWith", &starts_with::starts_with),
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
    ("getDate", &time_funcs::get_date::get_date),
    (
        "getDayOfMonth",
        &time_funcs::get_day_of_month::get_day_of_month,
    ),
    (
        "getDayOfWeek",
        &time_funcs::get_day_of_week::get_day_of_week,
    ),
    (
        "getDayOfYear",
        &time_funcs::get_day_of_year::get_day_of_year,
    ),
    ("getFullYear", &time_funcs::get_full_year::get_full_year),
    ("getHours", &time_funcs::get_hours::get_hours),
    (
        "getMilliseconds",
        &time_funcs::get_milliseconds::get_milliseconds,
    ),
    ("getMinutes", &time_funcs::get_minutes::get_minutes),
    ("getMonth", &time_funcs::get_month::get_month),
    ("getSeconds", &time_funcs::get_seconds::get_seconds),
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
