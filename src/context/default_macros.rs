use crate::{
    parser::Expr,
    value_cell::{ValueCell, ValueCellError, ValueCellResult},
    CelContext, ExecContext,
};

use super::{exec_context::RsCellMacro, utils::extract_ident};

const DEFAULT_MACROS: &[(&str, RsCellMacro)] = &[("map", map_impl)];

pub fn load_default_macros(exec_ctx: &mut ExecContext) {
    for (name, macro_) in DEFAULT_MACROS.iter() {
        exec_ctx.bind_macro(name, *macro_)
    }
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
