use super::bind_context::RsCelFunction;
use crate::{BindContext, CelError, CelValue};

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
    ("toLower", &string::to_lower_impl),
    ("toUpper", &string::to_upper_impl),
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
    ("sqrt", &math::sqrt::sqrt),
    ("pow", &math::pow::pow),
    ("log", &math::log::log),
    ("lg", &math::lg::lg),
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
    ("now", &now_impl),
    ("zip", &zip_impl),
    ("uomConvert", &uom::uom_convert),
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
