use super::bind_context::RsCelFunction;
use crate::{BindContext, CelError, CelValue};

mod contains;
mod ends_with;
mod matches;
mod math;
mod size;
mod split_whitespace;
mod starts_with;

mod time_funcs;

const DEFAULT_FUNCS: &[(&str, &'static RsCelFunction)] = &[
    ("contains", &contains::contains),
    ("containsI", &contains::contains_i),
    ("size", &size::size),
    ("startsWith", &starts_with::starts_with),
    ("endsWith", &ends_with::ends_with),
    ("startsWithI", &starts_with::starts_with_i),
    ("endsWithI", &ends_with::ends_with_i),
    ("matches", &matches::matches),
    ("toLower", &to_lower_impl),
    ("toUpper", &to_upper_impl),
    ("trim", &trim_impl),
    ("trimStart", &trim_start_impl),
    ("trimEnd", &trim_end_impl),
    ("splitWhiteSpace", &split_whitespace::split_whitespace),
    ("abs", &math::abs::abs),
    ("sqrt", &math::sqrt::sqrt),
    ("pow", &math::pow::pow),
    ("log", &math::log::log),
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

string_func!(toLower, to_lower_impl, to_lowercase);
string_func!(toUpper, to_upper_impl, to_uppercase);
string_func!(trim, trim_impl, trim);
string_func!(trimStart, trim_start_impl, trim_start);
string_func!(trimEnd, trim_end_impl, trim_end);

fn ceil_impl(_this: CelValue, args: Vec<CelValue>) -> CelValue {
    if args.len() != 1 {
        return CelValue::from_err(CelError::argument("ceil() expects one argument"));
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
        return CelValue::from_err(CelError::argument("floor() expects one argument"));
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
        return CelValue::from_err(CelError::argument("round() expects one argument"));
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
