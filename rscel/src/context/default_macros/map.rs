use super::helpers;
use crate::interp::Interpreter;
use crate::types::MacroArg;
use crate::{CelError, CelValue, CelValueDyn};

pub fn map_impl(ctx: &Interpreter, this: CelValue, args: &[MacroArg]) -> CelValue {
    if !(args.len() == 2 || args.len() == 3) {
        return CelValue::from_err(CelError::argument(
            "map() macro expects exactly 2 or 3 arguments",
        ));
    }

    let ident_name = match args[0].as_binder() {
        Ok(s) => s.to_owned(),
        Err(e) => return e.into(),
    };

    // 2-arg form: map(x, expr)
    // 3-arg form: map(x, predicate, expr) — only include when predicate is truthy
    let (map_expr, filter_expr) = if args.len() == 2 {
        (&args[1], None)
    } else {
        (&args[2], Some(&args[1]))
    };

    match this {
        CelValue::List(list) => map_list(ctx, list, &ident_name, map_expr, filter_expr),
        CelValue::Map(map) => map_map(ctx, map, &ident_name, map_expr, filter_expr),
        CelValue::Err(e) => CelValue::Err(e),
        _ => CelValue::from_err(CelError::value("map() only available on list")),
    }
}

fn map_list(
    ctx: &Interpreter,
    list: Vec<CelValue>,
    ident_name: &str,
    map_expr: &MacroArg,
    filter_expr: Option<&MacroArg>,
) -> CelValue {
    let (cel, mut bindings) = helpers::child_scope(ctx);
    let mut mapped = Vec::new();

    for value in list.into_iter() {
        bindings.bind_param(ident_name, value.clone());
        let interp = Interpreter::new(&cel, &bindings);

        if let Some(pred) = filter_expr {
            let predicate = match pred.eval(&interp) {
                Ok(val) => val,
                Err(err) => return err.into(),
            };
            if !predicate.is_truthy() {
                continue;
            }
        }

        match map_expr.eval(&interp) {
            Ok(val) => mapped.push(val),
            Err(err) => return err.into(),
        }
    }

    mapped.into()
}

fn map_map(
    ctx: &Interpreter,
    map: std::collections::HashMap<String, CelValue>,
    ident_name: &str,
    map_expr: &MacroArg,
    filter_expr: Option<&MacroArg>,
) -> CelValue {
    let (cel, mut bindings) = helpers::child_scope(ctx);
    let mut mapped = Vec::new();

    for key in map.into_keys() {
        let value: CelValue = key.into();
        bindings.bind_param(ident_name, value.clone());
        let interp = Interpreter::new(&cel, &bindings);

        if let Some(pred) = filter_expr {
            let predicate = match pred.eval(&interp) {
                Ok(val) => val,
                Err(err) => return err.into(),
            };
            if !predicate.is_truthy() {
                continue;
            }
        }

        match map_expr.eval(&interp) {
            Ok(val) => mapped.push(val),
            Err(err) => return err.into(),
        }
    }

    mapped.into()
}
