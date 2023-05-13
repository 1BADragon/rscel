use crate::{
    value_cell::{ValueCell, ValueCellError, ValueCellResult},
    ExecContext,
};

use super::exec_context::RsCellCallback;

const DEFAULT_FUNCS: &[(&str, RsCellCallback)] = &[
    ("int", int_impl),
    ("uint", uint_impl),
    ("double", double_impl),
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
        return Err(ValueCellError::with_msg("int expects exactly one argument"));
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
        return Err(ValueCellError::with_msg("int expects exactly one argument"));
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
        return Err(ValueCellError::with_msg("int expects exactly one argument"));
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
