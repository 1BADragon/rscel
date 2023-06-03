use crate::{
    interp::Interpreter,
    value_cell::{ValueCell, ValueCellError, ValueCellResult},
    BindContext, ByteCode, CelContext, ValueCellInner,
};

use super::bind_context::RsCellMacro;

const DEFAULT_MACROS: &[(&str, RsCellMacro)] = &[
    // ("has", has_impl),
    // ("all", all_impl),
    // ("exists", exists_impl),
    // ("exists_one", exists_one_impl),
    // ("filter", filter_impl),
    ("map", map_impl),
];

pub fn load_default_macros(exec_ctx: &mut BindContext) {
    for (name, macro_) in DEFAULT_MACROS.iter() {
        exec_ctx.bind_macro(name, *macro_)
    }
}

// fn has_impl(ctx: &CelContext, _this: ValueCell, exprlist: &[&Expr]) -> ValueCellResult<ValueCell> {
//     if exprlist.len() != 1 {
//         return Err(ValueCellError::with_msg(
//             "has() macro expects exactly 1 argument",
//         ));
//     }

//     // Clone the exec context first
//     let mut tmp_ctx = ctx.clone();
//     let exec_ctx = match ctx.exec_context() {
//         Some(exec) => exec,
//         None => return Err(ValueCellError::with_msg("Internal Error")),
//     };

//     match tmp_ctx.eval_expr(&exprlist[0], &exec_ctx) {
//         Ok(_) => Ok(ValueCell::from_bool(true)),
//         Err(_) => Ok(ValueCell::from_bool(false)),
//     }
// }

// fn all_impl(ctx: &CelContext, this: ValueCell, exprlist: &[&Expr]) -> ValueCellResult<ValueCell> {
//     if exprlist.len() != 2 {
//         return Err(ValueCellError::with_msg(
//             "map() macro expects exactly 2 arguments",
//         ));
//     }

//     let ident_name = match extract_ident(exprlist[0]) {
//         Some(name) => name,
//         None => return Err(ValueCellError::with_msg("Predicate is invalid map")),
//     };

//     if let ValueCellInner::List(list) = this.inner() {
//         for v in list.into_iter() {
//             let mut tmp_ctx = ctx.clone();
//             let mut exec_ctx = match ctx.exec_context() {
//                 Some(exec) => exec,
//                 None => return Err(ValueCellError::with_msg("Internal Error")),
//             };

//             exec_ctx.bind_param(&ident_name, v.clone());

//             if let Ok(vc) = tmp_ctx.eval_expr(exprlist[1], &exec_ctx) {
//                 if let ValueCellInner::Bool(res) = vc.inner() {
//                     if !res {
//                         return Ok(ValueCell::from_bool(false));
//                     }
//                 }
//             }
//         }

//         return Ok(ValueCell::from_bool(true));
//     }

//     Err(ValueCellError::with_msg(
//         "all() macro expects list predicate",
//     ))
// }

// fn exists_impl(
//     ctx: &CelContext,
//     this: ValueCell,
//     exprlist: &[&Expr],
// ) -> ValueCellResult<ValueCell> {
//     if exprlist.len() != 2 {
//         return Err(ValueCellError::with_msg(
//             "map() macro expects exactly 2 arguments",
//         ));
//     }

//     let ident_name = match extract_ident(exprlist[0]) {
//         Some(name) => name,
//         None => return Err(ValueCellError::with_msg("Predicate is invalid map")),
//     };

//     if let ValueCellInner::List(list) = this.inner() {
//         for v in list.into_iter() {
//             let mut tmp_ctx = ctx.clone();
//             let mut exec_ctx = match ctx.exec_context() {
//                 Some(exec) => exec,
//                 None => return Err(ValueCellError::with_msg("Internal Error")),
//             };

//             exec_ctx.bind_param(&ident_name, v.clone());

//             if let Ok(vc) = tmp_ctx.eval_expr(exprlist[1], &exec_ctx) {
//                 if let ValueCellInner::Bool(res) = vc.inner() {
//                     if *res {
//                         return Ok(ValueCell::from_bool(true));
//                     }
//                 }
//             }
//         }

//         return Ok(ValueCell::from_bool(false));
//     }

//     Err(ValueCellError::with_msg(
//         "exists() macro expects list predicate",
//     ))
// }

// fn exists_one_impl(
//     ctx: &CelContext,
//     this: ValueCell,
//     exprlist: &[&Expr],
// ) -> ValueCellResult<ValueCell> {
//     if exprlist.len() != 2 {
//         return Err(ValueCellError::with_msg(
//             "map() macro expects exactly 2 arguments",
//         ));
//     }

//     let ident_name = match extract_ident(exprlist[0]) {
//         Some(name) => name,
//         None => return Err(ValueCellError::with_msg("Predicate is invalid map")),
//     };

//     if let ValueCellInner::List(list) = this.inner() {
//         let mut count = 0;
//         for v in list.into_iter() {
//             let mut tmp_ctx = ctx.clone();
//             let mut exec_ctx = match ctx.exec_context() {
//                 Some(exec) => exec,
//                 None => return Err(ValueCellError::with_msg("Internal Error")),
//             };

//             exec_ctx.bind_param(&ident_name, v.clone());

//             if let Ok(vc) = tmp_ctx.eval_expr(exprlist[1], &exec_ctx) {
//                 if let ValueCellInner::Bool(res) = vc.inner() {
//                     if *res {
//                         count += 1;

//                         if count > 1 {
//                             return Ok(ValueCell::from_bool(false));
//                         }
//                     }
//                 }
//             }
//         }

//         return Ok(ValueCell::from_bool(count == 1));
//     }

//     Err(ValueCellError::with_msg(
//         "exists_one() macro expects list predicate",
//     ))
// }

// fn filter_impl(
//     ctx: &CelContext,
//     this: ValueCell,
//     exprlist: &[&Expr],
// ) -> ValueCellResult<ValueCell> {
//     if exprlist.len() != 2 {
//         return Err(ValueCellError::with_msg(
//             "map() macro expects exactly 2 arguments",
//         ));
//     }

//     let ident_name = match extract_ident(exprlist[0]) {
//         Some(name) => name,
//         None => return Err(ValueCellError::with_msg("Predicate is invalid map")),
//     };

//     if let ValueCellInner::List(list) = this.inner() {
//         let mut new_list: Vec<ValueCell> = Vec::new();
//         for v in list.into_iter() {
//             let mut tmp_ctx = ctx.clone();
//             let mut exec_ctx = match ctx.exec_context() {
//                 Some(exec) => exec,
//                 None => return Err(ValueCellError::with_msg("Internal Error")),
//             };

//             exec_ctx.bind_param(&ident_name, v.clone());

//             if let ValueCellInner::Bool(res) = tmp_ctx.eval_expr(exprlist[1], &exec_ctx)?.inner() {
//                 if *res {
//                     new_list.push(v.clone());
//                 }
//             }
//         }

//         return Ok(new_list.into());
//     }

//     Err(ValueCellError::with_msg(
//         "filter() macro expects list predicate",
//     ))
// }

fn map_impl(
    ctx: &Interpreter,
    this: ValueCell,
    bytecode: &[&[ByteCode]],
) -> ValueCellResult<ValueCell> {
    if bytecode.len() != 2 {
        return Err(ValueCellError::with_msg(
            "map() macro expects exactly 2 arguments",
        ));
    }

    let ident_prog = ctx.run_raw(bytecode[0])?;
    let ident_name = if let ValueCellInner::Ident(ident) = ident_prog.inner() {
        ident
    } else {
        return Err(ValueCellError::with_msg("map() predicate must be ident"));
    };

    if let ValueCellInner::List(list) = this.inner() {
        let mut mapped_list: Vec<ValueCell> = Vec::new();
        let cel = ctx.cel_copy().unwrap_or_else(|| CelContext::new());
        let mut bindings = ctx.bindings_copy().unwrap_or_else(|| BindContext::new());

        for v in list.into_iter() {
            // make a copy of the context to make borrow checker happy
            bindings.bind_param(&ident_name, v.clone());
            let interp = Interpreter::new(&cel, &bindings);

            mapped_list.push(interp.run_raw(bytecode[1])?);
        }
        Ok(mapped_list.into())
    } else {
        Err(ValueCellError::with_msg("map() only available on list"))
    }
}
