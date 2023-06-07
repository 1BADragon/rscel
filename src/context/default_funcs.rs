use super::bind_context::RsCelFunction;
use crate::{
    value_cell::{ValueCell, ValueCellError, ValueCellInner, ValueCellResult},
    BindContext,
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
];

pub fn load_default_funcs(exec_ctx: &mut BindContext) {
    for (name, func) in DEFAULT_FUNCS.iter() {
        exec_ctx.bind_func(name, *func);
    }
}

fn int_impl(_: ValueCell, args: &[ValueCell]) -> ValueCellResult<ValueCell> {
    use ValueCellInner::*;

    if args.len() != 1 {
        return Err(ValueCellError::with_msg(
            "int() expects exactly one argument",
        ));
    }

    match args[0].inner() {
        Int(val) => Ok(ValueCell::from_int(*val)),
        UInt(val) => Ok(ValueCell::from_int(*val as i64)),
        Float(val) => Ok(ValueCell::from_int(*val as i64)),
        String(val) => match val.parse::<i64>() {
            Ok(res) => Ok(ValueCell::from_int(res)),
            Err(_err) => Err(ValueCellError::with_msg(&format!(
                "int conversion invalid for \"{}\"",
                val
            ))),
        },
        TimeStamp(val) => Ok(ValueCell::from_int(val.timestamp())),
        other => Err(ValueCellError::with_msg(&format!(
            "int conversion invalid for {:?}",
            other
        ))),
    }
}

fn uint_impl(_: ValueCell, args: &[ValueCell]) -> ValueCellResult<ValueCell> {
    use ValueCellInner::*;

    if args.len() != 1 {
        return Err(ValueCellError::with_msg(
            "uint() expects exactly one argument",
        ));
    }

    match args[0].inner() {
        Int(val) => Ok(ValueCell::from_uint(*val as u64)),
        UInt(val) => Ok(ValueCell::from_uint(*val)),
        Float(val) => Ok(ValueCell::from_uint(*val as u64)),
        String(val) => match val.parse::<u64>() {
            Ok(res) => Ok(ValueCell::from_uint(res)),
            Err(_err) => Err(ValueCellError::with_msg(&format!(
                "int conversion invalid for \"{}\"",
                val
            ))),
        },
        other => Err(ValueCellError::with_msg(&format!(
            "int conversion invalid for {:?}",
            other
        ))),
    }
}

fn double_impl(_: ValueCell, args: &[ValueCell]) -> ValueCellResult<ValueCell> {
    use ValueCellInner::*;

    if args.len() != 1 {
        return Err(ValueCellError::with_msg(
            "double() expects exactly one argument",
        ));
    }

    match args[0].inner() {
        Int(val) => Ok(ValueCell::from_float(*val as f64)),
        UInt(val) => Ok(ValueCell::from_float(*val as f64)),
        Float(val) => Ok(ValueCell::from_float(*val)),
        String(val) => match val.parse::<f64>() {
            Ok(res) => Ok(ValueCell::from_float(res)),
            Err(_err) => Err(ValueCellError::with_msg(&format!(
                "int conversion invalid for \"{}\"",
                val
            ))),
        },
        other => Err(ValueCellError::with_msg(&format!(
            "int conversion invalid for {:?}",
            other
        ))),
    }
}

fn bytes_impl(_this: ValueCell, args: &[ValueCell]) -> ValueCellResult<ValueCell> {
    use ValueCellInner::*;
    if args.len() != 1 {
        return Err(ValueCellError::with_msg(
            "bytes() expects exactly one argument",
        ));
    }

    match &args[0].inner() {
        String(val) => Ok(ValueCell::from_bytes(val.as_bytes().to_vec())),
        other => Err(ValueCellError::with_msg(&format!(
            "int conversion invalid for {:?}",
            other
        ))),
    }
}

fn string_impl(_this: ValueCell, args: &[ValueCell]) -> ValueCellResult<ValueCell> {
    use ValueCellInner::*;

    if args.len() != 1 {
        return Err(ValueCellError::with_msg(
            "string() expects exactly one argument",
        ));
    }

    let arg_type = args[0].as_type();

    Ok(match args[0].inner() {
        Int(i) => i.to_string().into(),
        UInt(i) => i.to_string().into(),
        Float(f) => f.to_string().into(),
        String(s) => s.clone().into(),
        Bytes(b) => match std::string::String::from_utf8(b.clone()) {
            Ok(s) => s.into(),
            Err(_) => return Err(ValueCellError::with_msg("Bad bytes in utf8 convertion")),
        },
        TimeStamp(ts) => ts.to_rfc3339().into(),
        Duration(d) => d.to_string().into(),
        _ => {
            return Err(ValueCellError::with_msg(&format!(
                "string() invalid for {:?}",
                arg_type
            )))
        }
    })
}

fn contains_impl(this: ValueCell, args: &[ValueCell]) -> ValueCellResult<ValueCell> {
    if args.len() != 1 {
        return Err(ValueCellError::with_msg(
            "contains() expects exactly one argument",
        ));
    }

    if let ValueCellInner::String(this_str) = this.into_inner() {
        if let ValueCellInner::String(rhs) = args[0].inner() {
            Ok(ValueCell::from_bool(this_str.contains(rhs)))
        } else {
            Err(ValueCellError::with_msg("contains() arg must be string"))
        }
    } else {
        Err(ValueCellError::with_msg(
            "contains() can only operate on string",
        ))
    }
}

fn size_impl(_this: ValueCell, args: &[ValueCell]) -> ValueCellResult<ValueCell> {
    if args.len() != 1 {
        return Err(ValueCellError::with_msg(
            "size() expects exactly one argument",
        ));
    }

    Ok(ValueCell::from_uint(match args[0].inner() {
        ValueCellInner::String(s) => s.len() as u64,
        ValueCellInner::Bytes(b) => b.len() as u64,
        ValueCellInner::List(l) => l.len() as u64,
        ValueCellInner::Map(m) => m.len() as u64,
        _ => {
            return Err(ValueCellError::with_msg(
                "size() only available for types {string, bytes, list, map}",
            ))
        }
    }))
}

fn starts_with_impl(this: ValueCell, args: &[ValueCell]) -> ValueCellResult<ValueCell> {
    if args.len() != 1 {
        return Err(ValueCellError::with_msg(
            "endsWith() expects exactly one argument",
        ));
    }

    if let ValueCellInner::String(lhs) = this.inner() {
        if let ValueCellInner::String(rhs) = args[0].inner() {
            return Ok(lhs.starts_with(rhs).into());
        }
    }

    Err(ValueCellError::with_msg(
        "endsWith must be form string.(string)",
    ))
}

fn ends_with_impl(this: ValueCell, args: &[ValueCell]) -> ValueCellResult<ValueCell> {
    if args.len() != 1 {
        return Err(ValueCellError::with_msg(
            "endsWith() expects exactly one argument",
        ));
    }

    if let ValueCellInner::String(lhs) = this.inner() {
        if let ValueCellInner::String(rhs) = args[0].inner() {
            return Ok(lhs.ends_with(rhs).into());
        }
    }

    Err(ValueCellError::with_msg(
        "endsWith must be form string.(string)",
    ))
}

fn matches_impl(this: ValueCell, args: &[ValueCell]) -> ValueCellResult<ValueCell> {
    let (vc_lhs, vc_rhs) = if let ValueCellInner::Null = this.inner() {
        if args.len() != 2 {
            return Err(ValueCellError::with_msg(
                "matches() expects exactly two argument",
            ));
        }
        (&args[0], &args[1])
    } else {
        if args.len() != 1 {
            return Err(ValueCellError::with_msg(
                "matches() expects exactly one argument",
            ));
        }
        (&this, &args[0])
    };

    if let ValueCellInner::String(lhs) = vc_lhs.inner() {
        if let ValueCellInner::String(rhs) = vc_rhs.inner() {
            match Regex::new(rhs) {
                Ok(re) => return Ok(re.is_match(lhs).into()),
                Err(err) => {
                    return Err(ValueCellError::with_msg(&format!(
                        "Invalid regular expression: {}",
                        err
                    )))
                }
            }
        }
    }

    Err(ValueCellError::with_msg(
        "matches has the forms string.(string) or (string, string)",
    ))
}

fn type_impl(_this: ValueCell, args: &[ValueCell]) -> ValueCellResult<ValueCell> {
    if args.len() != 1 {
        return Err(ValueCellError::with_msg("type() expects one argument"));
    }

    Ok(args[0].as_type())
}

fn timestamp_impl(_this: ValueCell, args: &[ValueCell]) -> ValueCellResult<ValueCell> {
    if args.len() != 1 {
        return Err(ValueCellError::with_msg("timestamp() expect one argument"));
    }

    if let ValueCellInner::String(str_val) = args[0].inner() {
        match (&str_val).parse::<DateTime<Utc>>() {
            Ok(val) => Ok(ValueCell::from_timestamp(&val)),
            Err(_) => Err(ValueCellError::with_msg("Invalid timestamp format")),
        }
    } else {
        Err(ValueCellError::with_msg(
            "timestamp() expects a string argument",
        ))
    }
}

fn duration_impl(_this: ValueCell, args: &[ValueCell]) -> ValueCellResult<ValueCell> {
    if args.len() != 1 {
        return Err(ValueCellError::with_msg("duration() expects on argument"));
    }

    if let ValueCellInner::String(str_val) = args[0].inner() {
        match duration_str::parse_chrono(str_val) {
            Ok(val) => Ok(ValueCell::from_duration(&val)),
            Err(_) => Err(ValueCellError::with_msg("Invalid duration format")),
        }
    } else {
        Err(ValueCellError::with_msg(
            "duration() expects a string argument",
        ))
    }
}
