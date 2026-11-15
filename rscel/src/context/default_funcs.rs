use super::bind_context::RsCelFunction;
use crate::{interp::runtime_start_time, BindContext, CelError, CelValue};

mod base64;
mod format;
mod list;
mod math;
mod size;
mod sort;
mod string;
mod time_funcs;
mod uom;

// List of built in functions that come with the interp. The three fields are:
//    1. The name of the function in CEL -- &str
//    2. The callback method when that function is invoked -- &RsCelFunction
//    3. Whether or not the function is pure* in CEL -- bool
//
// * pure in this case does not include that a Vec might have 2 memory addresses when
// returning a list function as it would in Rust or C, but that the value in CEL is the
// same evertime the function is called with the same args, given the same input is
// provided.
pub const DEFAULT_FUNCS: &[(&str, &RsCelFunction, bool)] = &[
    ("contains", &string::contains::contains, true),
    ("containsI", &string::contains::contains_i, true),
    ("size", &size::size, true),
    ("flatten", &list::flatten::flatten, true),
    ("reverse", &list::reverse::reverse, true),
    ("slice", &list::slice::slice, true),
    ("sum", &list::sum_impl, true),
    ("unique", &list::unique::unique, true),
    ("sort", &sort::sort, true),
    ("startsWith", &string::starts_with::starts_with, true),
    ("endsWith", &string::ends_with::ends_with, true),
    ("startsWithI", &string::starts_with::starts_with_i, true),
    ("endsWithI", &string::ends_with::ends_with_i, true),
    ("matches", &string::matches::matches, true),
    (
        "matchCaptures",
        &string::match_captures::match_captures,
        true,
    ),
    (
        "matchReplaceOnce",
        &string::match_replace_once::match_replace_once,
        true,
    ),
    ("matchReplace", &string::match_replace::match_replace, true),
    ("indexOf", &string::index_of::index_of, true),
    ("lastIndexOf", &string::index_of::last_index_of, true),
    (
        "matchCapturesAll",
        &string::match_captures_all::match_captures_all,
        true,
    ),
    ("padEnd", &string::pad_end_impl, true),
    ("padStart", &string::pad_start_impl, true),
    ("repeat", &string::repeat::repeat, true),
    ("replaceI", &string::replace_i::replace_i, true),
    ("toLower", &string::to_lower_impl, true),
    ("toUpper", &string::to_upper_impl, true),
    ("lowerAscii", &string::to_lower_impl, true),
    ("upperAscii", &string::to_upper_impl, true),
    ("trimMatches", &string::trim_matches::trim_matches, true),
    ("remove", &string::remove::remove, true),
    ("replace", &string::replace::replace, true),
    ("rsplit", &string::split::rsplit, true),
    ("split", &string::split::split, true),
    ("splitAt", &string::split::split_at, true),
    ("trim", &string::trim_impl, true),
    ("trimStart", &string::trim_start_impl, true),
    (
        "trimStartMatches",
        &string::trim_start_matches::trim_start_matches,
        true,
    ),
    ("trimEnd", &string::trim_end_impl, true),
    (
        "trimEndMatches",
        &string::trim_end_matches::trim_end_matches,
        true,
    ),
    (
        "splitWhiteSpace",
        &string::split_whitespace::split_whitespace,
        true,
    ),
    ("decode", &base64::decode, true),
    ("encode", &base64::encode, true),
    ("charAt", &string::char_at::char_at, true),
    ("join", &string::join::join, true),
    ("quote", &string::quote::quote, true),
    ("substring", &string::substring::substring, true),
    ("abs", &math::abs::abs, true),
    ("cbrt", &math::cbrt::cbrt, true),
    ("ceil", &math::ceil::ceil, true),
    ("clamp", &clamp_impl, true),
    ("exp", &math::exp::exp, true),
    ("floor", &math::floor::floor, true),
    ("lg", &math::lg::lg, true),
    ("ln", &math::ln::ln, true),
    ("log", &math::log::log, true),
    ("greatest", &max_impl, true),
    ("least", &min_impl, true),
    ("max", &max_impl, true),
    ("min", &min_impl, true),
    ("pow", &math::pow::pow, true),
    ("round", &math::round::round, true),
    ("sqrt", &math::sqrt::sqrt, true),
    ("trunc", &math::trunc::trunc, true),
    ("getDate", &time_funcs::get_date::get_date, true),
    (
        "getDayOfMonth",
        &time_funcs::get_day_of_month::get_day_of_month,
        true,
    ),
    (
        "getDayOfWeek",
        &time_funcs::get_day_of_week::get_day_of_week,
        true,
    ),
    (
        "getDayOfYear",
        &time_funcs::get_day_of_year::get_day_of_year,
        true,
    ),
    (
        "getFullYear",
        &time_funcs::get_full_year::get_full_year,
        true,
    ),
    ("getHours", &time_funcs::get_hours::get_hours, true),
    (
        "getMilliseconds",
        &time_funcs::get_milliseconds::get_milliseconds,
        true,
    ),
    ("getMinutes", &time_funcs::get_minutes::get_minutes, true),
    ("getMonth", &time_funcs::get_month::get_month, true),
    ("getSeconds", &time_funcs::get_seconds::get_seconds, true),
    ("setDate", &time_funcs::set_date::set_date, true),
    (
        "setFullYear",
        &time_funcs::set_full_year::set_full_year,
        true,
    ),
    ("setHours", &time_funcs::set_hours::set_hours, true),
    (
        "setMilliseconds",
        &time_funcs::set_milliseconds::set_milliseconds,
        true,
    ),
    ("setMinutes", &time_funcs::set_minutes::set_minutes, true),
    ("setMonth", &time_funcs::set_month::set_month, true),
    ("setSeconds", &time_funcs::set_seconds::set_seconds, true),
    ("startOfDay", &time_funcs::start_of_day::start_of_day, true),
    (
        "startOfMonth",
        &time_funcs::start_of_month::start_of_month,
        true,
    ),
    (
        "startOfYear",
        &time_funcs::start_of_year::start_of_year,
        true,
    ),
    ("toRfc3339", &time_funcs::to_rfc3339::to_rfc3339, true),
    (
        "toTimestampString",
        &time_funcs::to_rfc3339::to_rfc3339,
        true,
    ),
    ("now", &now_impl, false),
    ("clockNow", &clock_timestamp_impl, false),
    ("zip", &zip_impl, true),
    ("uomConvert", &uom::uom_convert, true),
    ("format", &format::format, true),
];

pub fn load_default_funcs(exec_ctx: &mut BindContext) {
    for (name, func, _pure) in DEFAULT_FUNCS.iter() {
        exec_ctx.bind_func(name, *func);
    }
}

fn min_impl(this: CelValue, args: Vec<CelValue>) -> CelValue {
    let vals: &[CelValue] = if let CelValue::List(ref l) = this {
        l.as_slice()
    } else {
        args.as_slice()
    };

    if vals.is_empty() {
        return CelValue::from_err(CelError::argument("min() requires at least one argument"));
    }

    let mut curr_min = &vals[0];
    for val in &vals[1..] {
        if val.clone().lt(curr_min.clone()).is_true() {
            curr_min = val;
        }
    }
    curr_min.clone()
}

fn max_impl(this: CelValue, args: Vec<CelValue>) -> CelValue {
    let vals: &[CelValue] = if let CelValue::List(ref l) = this {
        l.as_slice()
    } else {
        args.as_slice()
    };

    if vals.is_empty() {
        return CelValue::from_err(CelError::argument("max() requires at least one argument"));
    }

    let mut curr_max = &vals[0];
    for val in &vals[1..] {
        if val.clone().gt(curr_max.clone()).is_true() {
            curr_max = val;
        }
    }
    curr_max.clone()
}

fn clamp_impl(_this: CelValue, args: Vec<CelValue>) -> CelValue {
    if args.len() != 3 {
        return CelValue::from_err(CelError::argument(
            "clamp() requires exactly three arguments: value, min, max",
        ));
    }
    let (val, lo, hi) = (&args[0], &args[1], &args[2]);
    if val.clone().lt(lo.clone()).is_true() {
        lo.clone()
    } else if val.clone().gt(hi.clone()).is_true() {
        hi.clone()
    } else {
        val.clone()
    }
}

fn zip_impl(_this: CelValue, args: Vec<CelValue>) -> CelValue {
    if args.is_empty() {
        return CelValue::from_val_slice(&[]);
    }

    let mut ret_val: Vec<CelValue> = Vec::new();
    let mut vecs = Vec::new();

    for arg in args.into_iter() {
        if let CelValue::List(l) = arg {
            vecs.push(l);
        } else {
            return CelValue::from_err(CelError::Value(
                "All inputs to zip must be lists".to_owned(),
            ));
        }
    }

    let min_len = vecs.iter().map(|v| v.len()).min().unwrap_or(0);

    let mut iters: Vec<_> = vecs.into_iter().map(|v| v.into_iter()).collect();

    for _ in 0..min_len {
        let zipped: Vec<_> = iters.iter_mut().map(|i| i.next().unwrap()).collect();

        ret_val.push(zipped.into());
    }

    ret_val.into()
}

fn now_impl(this: CelValue, args: Vec<CelValue>) -> CelValue {
    if !this.is_null() {
        return CelValue::from_err(CelError::argument("now() is not a method"));
    }

    if !args.is_empty() {
        return CelValue::from_err(CelError::argument("now() expects no arguments"));
    }

    runtime_start_time().into()
}

fn clock_timestamp_impl(this: CelValue, args: Vec<CelValue>) -> CelValue {
    if !this.is_null() {
        return CelValue::from_err(CelError::argument("clockNow() is not a method"));
    }

    if !args.is_empty() {
        return CelValue::from_err(CelError::argument("clockNow() expects no arguments"));
    }

    chrono::Utc::now().into()
}
