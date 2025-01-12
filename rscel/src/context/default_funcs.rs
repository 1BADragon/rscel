use super::bind_context::RsCelFunction;
use crate::{BindContext, CelError, CelValue};

mod math;
mod size;
mod string;
mod time_funcs;

const DEFAULT_FUNCS: &[(&str, &'static RsCelFunction)] = &[
    ("contains", &string::contains::contains),
    ("containsI", &string::contains::contains_i),
    ("size", &size::size),
    ("startsWith", &string::starts_with::starts_with),
    ("endsWith", &string::ends_with::ends_with),
    ("startsWithI", &string::starts_with::starts_with_i),
    ("endsWithI", &string::ends_with::ends_with_i),
    ("matches", &string::matches::matches),
    ("toLower", &string::to_lower_impl),
    ("toUpper", &string::to_upper_impl),
    ("trim", &string::trim_impl),
    ("trimStart", &string::trim_start_impl),
    ("trimEnd", &string::trim_end_impl),
    (
        "splitWhiteSpace",
        &string::split_whitespace::split_whitespace,
    ),
    ("abs", &math::abs::abs),
    ("sqrt", &math::sqrt::sqrt),
    ("pow", &math::pow::pow),
    ("log", &math::log::log),
    ("ceil", &math::ceil::ceil),
    ("floor", &math::floor::floor),
    ("round", &math::round::round),
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

pub fn load_default_funcs(exec_ctx: &mut BindContext) {
    for (name, func) in DEFAULT_FUNCS.iter() {
        exec_ctx.bind_func(name, *func);
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
