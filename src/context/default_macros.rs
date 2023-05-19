use crate::{
    ast::grammar::Expr,
    value_cell::{ValueCell, ValueCellError, ValueCellResult},
    CelContext, ExecContext,
};

use super::{exec_context::RsCellMacro, utils::extract_ident};

const DEFAULT_MACROS: &[(&str, RsCellMacro)] = &[
    ("has", has_impl),
    ("all", all_impl),
    ("exists", exists_impl),
    ("exists_one", exists_one_impl),
    ("filter", filter_impl),
    ("map", map_impl),
];

pub fn load_default_macros(exec_ctx: &mut ExecContext) {
    for (name, macro_) in DEFAULT_MACROS.iter() {
        exec_ctx.bind_macro(name, *macro_)
    }
}

fn has_impl(ctx: &CelContext, _this: ValueCell, exprlist: &[&Expr]) -> ValueCellResult<ValueCell> {
    if exprlist.len() != 1 {
        return Err(ValueCellError::with_msg(
            "has() macro expects exactly 1 argument",
        ));
    }

    // Clone the exec context first
    let mut tmp_ctx = ctx.clone();
    let exec_ctx = match ctx.exec_context() {
        Some(exec) => exec,
        None => return Err(ValueCellError::with_msg("Internal Error")),
    };

    match tmp_ctx.eval_expr(&exprlist[0], &exec_ctx) {
        Ok(_) => Ok(ValueCell::from_bool(true)),
        Err(_) => Ok(ValueCell::from_bool(false)),
    }
}

fn all_impl(ctx: &CelContext, this: ValueCell, exprlist: &[&Expr]) -> ValueCellResult<ValueCell> {
    if exprlist.len() != 2 {
        return Err(ValueCellError::with_msg(
            "map() macro expects exactly 2 arguments",
        ));
    }

    let ident_name = match extract_ident(exprlist[0]) {
        Some(name) => name,
        None => return Err(ValueCellError::with_msg("Predicate is invalid map")),
    };

    if let ValueCell::List(list) = this {
        for v in list.into_iter() {
            let mut tmp_ctx = ctx.clone();
            let mut exec_ctx = match ctx.exec_context() {
                Some(exec) => exec,
                None => return Err(ValueCellError::with_msg("Internal Error")),
            };

            exec_ctx.bind_param(&ident_name, v.into_json_value());

            if let Ok(ValueCell::Bool(res)) = tmp_ctx.eval_expr(exprlist[1], &exec_ctx) {
                if !res {
                    return Ok(ValueCell::from_bool(false));
                }
            }
        }

        return Ok(ValueCell::from_bool(true));
    }

    Err(ValueCellError::with_msg(
        "all() macro expects list predicate",
    ))
}

fn exists_impl(
    ctx: &CelContext,
    this: ValueCell,
    exprlist: &[&Expr],
) -> ValueCellResult<ValueCell> {
    if exprlist.len() != 2 {
        return Err(ValueCellError::with_msg(
            "map() macro expects exactly 2 arguments",
        ));
    }

    let ident_name = match extract_ident(exprlist[0]) {
        Some(name) => name,
        None => return Err(ValueCellError::with_msg("Predicate is invalid map")),
    };

    if let ValueCell::List(list) = this {
        for v in list.into_iter() {
            let mut tmp_ctx = ctx.clone();
            let mut exec_ctx = match ctx.exec_context() {
                Some(exec) => exec,
                None => return Err(ValueCellError::with_msg("Internal Error")),
            };

            exec_ctx.bind_param(&ident_name, v.into_json_value());

            if let Ok(ValueCell::Bool(res)) = tmp_ctx.eval_expr(exprlist[1], &exec_ctx) {
                if res {
                    return Ok(ValueCell::from_bool(true));
                }
            }
        }

        return Ok(ValueCell::from_bool(false));
    }

    Err(ValueCellError::with_msg(
        "exists() macro expects list predicate",
    ))
}

fn exists_one_impl(
    ctx: &CelContext,
    this: ValueCell,
    exprlist: &[&Expr],
) -> ValueCellResult<ValueCell> {
    if exprlist.len() != 2 {
        return Err(ValueCellError::with_msg(
            "map() macro expects exactly 2 arguments",
        ));
    }

    let ident_name = match extract_ident(exprlist[0]) {
        Some(name) => name,
        None => return Err(ValueCellError::with_msg("Predicate is invalid map")),
    };

    if let ValueCell::List(list) = this {
        let mut count = 0;
        for v in list.into_iter() {
            let mut tmp_ctx = ctx.clone();
            let mut exec_ctx = match ctx.exec_context() {
                Some(exec) => exec,
                None => return Err(ValueCellError::with_msg("Internal Error")),
            };

            exec_ctx.bind_param(&ident_name, v.into_json_value());

            if let Ok(ValueCell::Bool(res)) = tmp_ctx.eval_expr(exprlist[1], &exec_ctx) {
                if res {
                    count += 1;

                    if count > 1 {
                        return Ok(ValueCell::from_bool(false));
                    }
                }
            }
        }

        return Ok(ValueCell::from_bool(count == 1));
    }

    Err(ValueCellError::with_msg(
        "exists_one() macro expects list predicate",
    ))
}

fn filter_impl(
    ctx: &CelContext,
    this: ValueCell,
    exprlist: &[&Expr],
) -> ValueCellResult<ValueCell> {
    if exprlist.len() != 2 {
        return Err(ValueCellError::with_msg(
            "map() macro expects exactly 2 arguments",
        ));
    }

    let ident_name = match extract_ident(exprlist[0]) {
        Some(name) => name,
        None => return Err(ValueCellError::with_msg("Predicate is invalid map")),
    };

    if let ValueCell::List(list) = this {
        let mut new_list: Vec<ValueCell> = Vec::new();
        for v in list.into_iter() {
            let mut tmp_ctx = ctx.clone();
            let mut exec_ctx = match ctx.exec_context() {
                Some(exec) => exec,
                None => return Err(ValueCellError::with_msg("Internal Error")),
            };

            exec_ctx.bind_param(&ident_name, v.clone().into_json_value());

            if let ValueCell::Bool(res) = tmp_ctx.eval_expr(exprlist[1], &exec_ctx)? {
                if res {
                    new_list.push(v);
                }
            }
        }

        return Ok(ValueCell::List(new_list));
    }

    Err(ValueCellError::with_msg(
        "filter() macro expects list predicate",
    ))
}

fn map_impl(ctx: &CelContext, this: ValueCell, exprlist: &[&Expr]) -> ValueCellResult<ValueCell> {
    if exprlist.len() != 2 {
        return Err(ValueCellError::with_msg(
            "map() macro expects exactly 2 arguments",
        ));
    }

    let ident_name = match extract_ident(exprlist[0]) {
        Some(name) => name,
        None => return Err(ValueCellError::with_msg("Predicate is invalid map")),
    };

    if let ValueCell::List(list) = this {
        let mut mapped_list: Vec<ValueCell> = Vec::new();
        for v in list.into_iter() {
            // make a copy of the context to make borrow checker happy
            let mut tmp_ctx = ctx.clone();
            let mut exec_ctx = match ctx.exec_context() {
                Some(exec) => exec,
                None => return Err(ValueCellError::with_msg("Internal Error")),
            };

            exec_ctx.bind_param(&ident_name, v.into_json_value());
            mapped_list.push(tmp_ctx.eval_expr(exprlist[1], &exec_ctx)?);
        }
        Ok(ValueCell::List(mapped_list))
    } else {
        Err(ValueCellError::with_msg("map() only available on list"))
    }
}
