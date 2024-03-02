use crate::{
    cel_error::{CelError, CelResult},
    interp::Interpreter,
    utils::eval_ident,
    BindContext, ByteCode, CelContext, CelValue,
};

use super::bind_context::RsCelMacro;

const DEFAULT_MACROS: &[(&str, &'static RsCelMacro)] = &[
    ("has", &has_impl),
    ("all", &all_impl),
    ("exists", &exists_impl),
    ("exists_one", &exists_one_impl),
    ("filter", &filter_impl),
    ("map", &map_impl),
    ("reduce", &reduce_impl),
    ("coalesce", &coalesce_impl),
];

pub fn load_default_macros(exec_ctx: &mut BindContext) {
    for (name, macro_) in DEFAULT_MACROS.iter() {
        exec_ctx.bind_macro(name, *macro_)
    }
}

fn has_impl(ctx: &Interpreter, _this: CelValue, exprlist: &[&[ByteCode]]) -> CelResult<CelValue> {
    if exprlist.len() != 1 {
        return Err(CelError::argument("has() macro expects exactly 1 argument"));
    }

    let res = ctx.run_raw(&exprlist[0], true);
    match res {
        Ok(_) => Ok(CelValue::true_()),
        Err(err) => match err {
            CelError::Binding { .. } => Ok(CelValue::false_()),
            CelError::Attribute { .. } => Ok(CelValue::false_()),
            _ => Err(err),
        },
    }
}

fn all_impl(ctx: &Interpreter, this: CelValue, bytecode: &[&[ByteCode]]) -> CelResult<CelValue> {
    if bytecode.len() != 2 {
        return Err(CelError::argument(
            "all() macro expects exactly 2 arguments",
        ));
    }

    let ident_name = eval_ident(bytecode[0])?;
    if let CelValue::List(list) = this {
        let cel = ctx.cel_copy().unwrap_or_else(|| CelContext::new());
        let mut bindings = ctx.bindings_copy().unwrap_or_else(|| BindContext::new());

        for v in list.into_iter() {
            // make a copy of the context to make borrow checker happy
            bindings.bind_param(&ident_name, v.clone());
            let interp = Interpreter::new(&cel, &bindings);

            let res = interp.run_raw(bytecode[1], true)?;

            if !res.is_truthy() {
                return Ok(false.into());
            }
        }

        Ok(true.into())
    } else {
        Err(CelError::value("all() only available on list"))
    }
}

fn exists_impl(ctx: &Interpreter, this: CelValue, bytecode: &[&[ByteCode]]) -> CelResult<CelValue> {
    if bytecode.len() != 2 {
        return Err(CelError::argument(
            "exists() macro expects exactly 2 arguments",
        ));
    }

    let ident_name = eval_ident(bytecode[0])?;
    if let CelValue::List(list) = this {
        let cel = ctx.cel_copy().unwrap_or_else(|| CelContext::new());
        let mut bindings = ctx.bindings_copy().unwrap_or_else(|| BindContext::new());

        for v in list.into_iter() {
            bindings.bind_param(&ident_name, v.clone());
            let interp = Interpreter::new(&cel, &bindings);

            let res = interp.run_raw(bytecode[1], true)?;

            if res.is_truthy() {
                return Ok(true.into());
            }
        }

        Ok(false.into())
    } else {
        Err(CelError::value("exists() only available on list"))
    }
}

fn exists_one_impl(
    ctx: &Interpreter,
    this: CelValue,
    bytecode: &[&[ByteCode]],
) -> CelResult<CelValue> {
    if bytecode.len() != 2 {
        return Err(CelError::argument(
            "exists_one() macro expects exactly 2 arguments",
        ));
    }

    let ident_name = eval_ident(bytecode[0])?;

    if let CelValue::List(list) = this {
        let cel = ctx.cel_copy().unwrap_or_else(|| CelContext::new());
        let mut bindings = ctx.bindings_copy().unwrap_or_else(|| BindContext::new());

        let mut count = 0;
        for v in list.into_iter() {
            bindings.bind_param(&ident_name, v.clone());
            let interp = Interpreter::new(&cel, &bindings);

            let res = interp.run_raw(bytecode[1], true)?;

            if res.is_truthy() {
                count += 1;

                if count > 1 {
                    return Ok(false.into());
                }
            }
        }

        Ok((count == 1).into())
    } else {
        Err(CelError::value("exists_one() only available on list"))
    }
}

fn filter_impl(ctx: &Interpreter, this: CelValue, bytecode: &[&[ByteCode]]) -> CelResult<CelValue> {
    if bytecode.len() != 2 {
        return Err(CelError::argument(
            "filter() macro expects exactly 2 arguments",
        ));
    }

    let ident_name = eval_ident(bytecode[0])?;

    if let CelValue::List(list) = this {
        let mut filtered_list: Vec<CelValue> = Vec::new();
        let cel = ctx.cel_copy().unwrap_or_else(|| CelContext::new());
        let mut bindings = ctx.bindings_copy().unwrap_or_else(|| BindContext::new());

        for v in list.into_iter() {
            // make a copy of the context to make borrow checker happy
            bindings.bind_param(&ident_name, v.clone());
            let interp = Interpreter::new(&cel, &bindings);

            if interp.run_raw(bytecode[1], true)?.is_truthy() {
                filtered_list.push(v.clone());
            }
        }
        Ok(filtered_list.into())
    } else {
        Err(CelError::value("filter() only available on list"))
    }
}

fn map_impl(ctx: &Interpreter, this: CelValue, bytecode: &[&[ByteCode]]) -> CelResult<CelValue> {
    if !(bytecode.len() == 2 || bytecode.len() == 3) {
        return Err(CelError::argument(
            "map() macro expects exactly 2 or 3 arguments",
        ));
    }

    let ident_name = eval_ident(bytecode[0])?;

    if let CelValue::List(list) = this {
        let mut mapped_list: Vec<CelValue> = Vec::new();
        let cel = ctx.cel_copy().unwrap_or_else(|| CelContext::new());
        // make a copy of the context to make borrow checker happy
        let mut bindings = ctx.bindings_copy().unwrap_or_else(|| BindContext::new());

        // optimize so we are only checking bytecode's len once
        if bytecode.len() == 2 {
            for v in list.into_iter() {
                bindings.bind_param(&ident_name, v.clone());
                let interp = Interpreter::new(&cel, &bindings);

                mapped_list.push(interp.run_raw(bytecode[1], true)?);
            }
        } else if bytecode.len() == 3 {
            for v in list.into_iter() {
                bindings.bind_param(&ident_name, v.clone());
                let interp = Interpreter::new(&cel, &bindings);

                if interp.run_raw(bytecode[1], true)?.is_truthy() {
                    mapped_list.push(interp.run_raw(bytecode[2], true)?);
                }
            }
        } else {
            return Err(CelError::internal("Bytecode len check failed"));
        }

        Ok(mapped_list.into())
    } else {
        Err(CelError::value("map() only available on list"))
    }
}

// reduce [].reduce(curr, next, expression, starting)
fn reduce_impl(ctx: &Interpreter, this: CelValue, bytecode: &[&[ByteCode]]) -> CelResult<CelValue> {
    if bytecode.len() != 4 {
        return Err(CelError::argument("reduce() macro expects 4 arguments"));
    }

    let curr_name = eval_ident(bytecode[0])?;
    let next_name = eval_ident(bytecode[1])?;
    let mut cur_value = ctx.run_raw(bytecode[3], true)?;
    let cel = ctx.cel_copy().unwrap_or_else(|| CelContext::new());
    let mut bindings = ctx.bindings_copy().unwrap_or_else(|| BindContext::new());

    if let CelValue::List(list) = this {
        for next in list.into_iter() {
            bindings.bind_param(&next_name, next);
            bindings.bind_param(&curr_name, cur_value);

            let interp = Interpreter::new(&cel, &bindings);

            cur_value = interp.run_raw(bytecode[2], true)?;
        }

        Ok(cur_value)
    } else {
        Err(CelError::value("reduce() only availble on list"))
    }
}

fn coalesce_impl(
    ctx: &Interpreter,
    _this: CelValue,
    bytecode: &[&[ByteCode]],
) -> CelResult<CelValue> {
    for arg in bytecode.iter() {
        let res = ctx.run_raw(arg, true);
        match res {
            Ok(CelValue::Null) => {}
            Err(err) => match err {
                CelError::Binding { .. } | CelError::Attribute { .. } => {}
                _ => return Err(err),
            },
            Ok(v) => return Ok(v),
        };
    }

    Ok(CelValue::from_null())
}
