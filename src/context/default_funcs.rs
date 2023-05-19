use super::exec_context::RsCellFunction;
use crate::{
    value_cell::{ValueCell, ValueCellError, ValueCellResult},
    ExecContext,
};
use regex::Regex;

const DEFAULT_FUNCS: &[(&str, RsCellFunction)] = &[
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
];

pub fn load_default_funcs(exec_ctx: &mut ExecContext) {
    for (name, func) in DEFAULT_FUNCS.iter() {
        exec_ctx.bind_func(name, *func);
    }
}

fn int_impl(_: ValueCell, args: ValueCell) -> ValueCellResult<ValueCell> {
    use ValueCell::*;
    let arg_list: Vec<ValueCell> = args.try_into()?;

    if arg_list.len() != 1 {
        return Err(ValueCellError::with_msg(
            "int() expects exactly one argument",
        ));
    }

    match &arg_list[0] {
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
        other => Err(ValueCellError::with_msg(&format!(
            "int conversion invalid for {:?}",
            other
        ))),
    }
}

fn uint_impl(_: ValueCell, args: ValueCell) -> ValueCellResult<ValueCell> {
    use ValueCell::*;
    let arg_list: Vec<ValueCell> = args.try_into()?;

    if arg_list.len() != 1 {
        return Err(ValueCellError::with_msg(
            "uint() expects exactly one argument",
        ));
    }

    match &arg_list[0] {
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

fn double_impl(_: ValueCell, args: ValueCell) -> ValueCellResult<ValueCell> {
    use ValueCell::*;
    let arg_list: Vec<ValueCell> = args.try_into()?;

    if arg_list.len() != 1 {
        return Err(ValueCellError::with_msg(
            "double() expects exactly one argument",
        ));
    }

    match &arg_list[0] {
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

fn bytes_impl(_this: ValueCell, args: ValueCell) -> ValueCellResult<ValueCell> {
    use ValueCell::*;
    let arg_list: Vec<ValueCell> = args.try_into()?;

    if arg_list.len() != 1 {
        return Err(ValueCellError::with_msg(
            "bytes() expects exactly one argument",
        ));
    }

    match &arg_list[0] {
        String(val) => Ok(ValueCell::from_bytes(val.as_bytes())),
        other => Err(ValueCellError::with_msg(&format!(
            "int conversion invalid for {:?}",
            other
        ))),
    }
}

fn string_impl(_this: ValueCell, args: ValueCell) -> ValueCellResult<ValueCell> {
    use ValueCell::*;
    let arg_list: Vec<ValueCell> = args.try_into()?;

    if arg_list.len() != 1 {
        return Err(ValueCellError::with_msg(
            "string() expects exactly one argument",
        ));
    }

    let arg_type = arg_list[0].as_type();

    Ok(match &arg_list[0] {
        Int(i) => i.to_string().into(),
        UInt(i) => i.to_string().into(),
        Float(f) => f.to_string().into(),
        String(s) => s.clone().into(),
        Bytes(b) => match std::string::String::from_utf8(b.clone()) {
            Ok(s) => s.into(),
            Err(_) => return Err(ValueCellError::with_msg("Bad bytes in utf8 convertion")),
        },
        _ => {
            return Err(ValueCellError::with_msg(&format!(
                "string() invalid for {:?}",
                arg_type
            )))
        }
    })
}

fn contains_impl(this: ValueCell, args: ValueCell) -> ValueCellResult<ValueCell> {
    let arg_list: Vec<ValueCell> = args.try_into()?;

    if arg_list.len() != 1 {
        return Err(ValueCellError::with_msg(
            "contains() expects exactly one argument",
        ));
    }

    if let ValueCell::String(this_str) = this {
        if let ValueCell::String(rhs) = &arg_list[0] {
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

fn size_impl(_this: ValueCell, args: ValueCell) -> ValueCellResult<ValueCell> {
    let arg_list: Vec<ValueCell> = args.try_into()?;

    if arg_list.len() != 1 {
        return Err(ValueCellError::with_msg(
            "size() expects exactly one argument",
        ));
    }

    Ok(ValueCell::from_uint(match &arg_list[0] {
        ValueCell::String(s) => s.len() as u64,
        ValueCell::Bytes(b) => b.len() as u64,
        ValueCell::List(l) => l.len() as u64,
        ValueCell::Map(m) => m.len() as u64,
        _ => {
            return Err(ValueCellError::with_msg(
                "size() only available for types {string, bytes, list, map}",
            ))
        }
    }))
}

fn starts_with_impl(this: ValueCell, args: ValueCell) -> ValueCellResult<ValueCell> {
    let arg_list: Vec<ValueCell> = args.try_into()?;

    if arg_list.len() != 1 {
        return Err(ValueCellError::with_msg(
            "endsWith() expects exactly one argument",
        ));
    }

    if let ValueCell::String(lhs) = this {
        if let ValueCell::String(rhs) = &arg_list[0] {
            return Ok(lhs.starts_with(rhs).into());
        }
    }

    Err(ValueCellError::with_msg(
        "endsWith must be form string.(string)",
    ))
}

fn ends_with_impl(this: ValueCell, args: ValueCell) -> ValueCellResult<ValueCell> {
    let arg_list: Vec<ValueCell> = args.try_into()?;

    if arg_list.len() != 1 {
        return Err(ValueCellError::with_msg(
            "endsWith() expects exactly one argument",
        ));
    }

    if let ValueCell::String(lhs) = this {
        if let ValueCell::String(rhs) = &arg_list[0] {
            return Ok(lhs.ends_with(rhs).into());
        }
    }

    Err(ValueCellError::with_msg(
        "endsWith must be form string.(string)",
    ))
}

fn matches_impl(this: ValueCell, args: ValueCell) -> ValueCellResult<ValueCell> {
    let arg_list: Vec<ValueCell> = args.try_into()?;

    let (vc_lhs, vc_rhs) = if let ValueCell::Null = this {
        if arg_list.len() != 2 {
            return Err(ValueCellError::with_msg(
                "matches() expects exactly two argument",
            ));
        }
        (&arg_list[0], &arg_list[1])
    } else {
        if arg_list.len() != 1 {
            return Err(ValueCellError::with_msg(
                "matches() expects exactly one argument",
            ));
        }
        (&this, &arg_list[0])
    };

    if let ValueCell::String(lhs) = vc_lhs {
        if let ValueCell::String(rhs) = vc_rhs {
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
