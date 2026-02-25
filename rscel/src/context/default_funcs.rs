use super::bind_context::RsCelFunction;
use crate::{BindContext, CelError, CelValue};

mod format;
mod list;
mod math;
mod size;
mod sort;
mod string;
mod time_funcs;
mod uom;

const DEFAULT_FUNCS: &[(&str, &'static RsCelFunction)] = &[
    ("contains", &string::contains::contains),
    ("containsI", &string::contains::contains_i),
    ("size", &size::size),
    ("flatten", &list::flatten::flatten),
    ("reverse", &list::reverse::reverse),
    ("slice", &list::slice::slice),
    ("sum", &list::sum_impl),
    ("unique", &list::unique::unique),
    ("sort", &sort::sort),
    ("startsWith", &string::starts_with::starts_with),
    ("endsWith", &string::ends_with::ends_with),
    ("startsWithI", &string::starts_with::starts_with_i),
    ("endsWithI", &string::ends_with::ends_with_i),
    ("matches", &string::matches::matches),
    ("matchCaptures", &string::match_captures::match_captures),
    (
        "matchReplaceOnce",
        &string::match_replace_once::match_replace_once,
    ),
    ("matchReplace", &string::match_replace::match_replace),
    ("indexOf", &string::index_of::index_of),
    ("lastIndexOf", &string::index_of::last_index_of),
    ("matchCapturesAll", &string::match_captures_all::match_captures_all),
    ("padEnd", &string::pad_end_impl),
    ("padStart", &string::pad_start_impl),
    ("repeat", &string::repeat::repeat),
    ("replaceI", &string::replace_i::replace_i),
    ("toLower", &string::to_lower_impl),
    ("toUpper", &string::to_upper_impl),
    ("trimMatches", &string::trim_matches::trim_matches),
    ("remove", &string::remove::remove),
    ("replace", &string::replace::replace),
    ("rsplit", &string::split::rsplit),
    ("split", &string::split::split),
    ("splitAt", &string::split::split_at),
    ("trim", &string::trim_impl),
    ("trimStart", &string::trim_start_impl),
    (
        "trimStartMatches",
        &string::trim_start_matches::trim_start_matches,
    ),
    ("trimEnd", &string::trim_end_impl),
    (
        "trimEndMatches",
        &string::trim_end_matches::trim_end_matches,
    ),
    (
        "splitWhiteSpace",
        &string::split_whitespace::split_whitespace,
    ),
    ("abs", &math::abs::abs),
    ("cbrt", &math::cbrt::cbrt),
    ("ceil", &math::ceil::ceil),
    ("clamp", &clamp_impl),
    ("exp", &math::exp::exp),
    ("floor", &math::floor::floor),
    ("lg", &math::lg::lg),
    ("ln", &math::ln::ln),
    ("log", &math::log::log),
    ("max", &max_impl),
    ("min", &min_impl),
    ("pow", &math::pow::pow),
    ("round", &math::round::round),
    ("sqrt", &math::sqrt::sqrt),
    ("trunc", &math::trunc::trunc),
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
    ("setDate", &time_funcs::set_date::set_date),
    ("setFullYear", &time_funcs::set_full_year::set_full_year),
    ("setHours", &time_funcs::set_hours::set_hours),
    (
        "setMilliseconds",
        &time_funcs::set_milliseconds::set_milliseconds,
    ),
    ("setMinutes", &time_funcs::set_minutes::set_minutes),
    ("setMonth", &time_funcs::set_month::set_month),
    ("setSeconds", &time_funcs::set_seconds::set_seconds),
    ("startOfDay", &time_funcs::start_of_day::start_of_day),
    ("startOfMonth", &time_funcs::start_of_month::start_of_month),
    ("startOfYear", &time_funcs::start_of_year::start_of_year),
    ("toRfc3339", &time_funcs::to_rfc3339::to_rfc3339),
    ("toTimestampString", &time_funcs::to_rfc3339::to_rfc3339),
    ("now", &now_impl),
    ("zip", &zip_impl),
    ("uomConvert", &uom::uom_convert),
    ("format", &format::format),
];

pub fn load_default_funcs(exec_ctx: &mut BindContext) {
    for (name, func) in DEFAULT_FUNCS.iter() {
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

fn now_impl(_this: CelValue, args: Vec<CelValue>) -> CelValue {
    if !args.is_empty() {
        return CelValue::from_err(CelError::argument("now() expects no arguments"));
    }

    CelValue::from_timestamp(chrono::Utc::now())
}
