use crate::context::bound_predicate_executor::BoundPredicateExecutor;
use crate::interp::Interpreter;
use crate::types::MacroArg;
use crate::{CelError, CelValue, CelValueDyn};

pub fn map_impl(ctx: &Interpreter, this: CelValue, args: &[MacroArg]) -> CelValue {
    if !(args.len() == 2 || args.len() == 3) {
        return CelValue::from_err(CelError::argument(
            "map() macro expects exactly 2 or 3 arguments",
        ));
    }

    // 2-arg form: map(x, expr)
    // 3-arg form: map(x, predicate, expr) — only include when predicate is truthy
    let (map_expr, filter_expr) = if args.len() == 2 {
        (&args[1], None)
    } else {
        (&args[2], Some(&args[1]))
    };

    match this {
        CelValue::List(list) => map_list(ctx, list, &args[0], map_expr, filter_expr),
        CelValue::Map(map) => map_map(ctx, map, &args[0], map_expr, filter_expr),
        CelValue::Err(e) => CelValue::Err(e),
        _ => CelValue::from_err(CelError::value("map() only available on list")),
    }
}

fn map_list(
    ctx: &Interpreter,
    list: Vec<CelValue>,
    binder: &MacroArg,
    map_expr: &MacroArg,
    filter_expr: Option<&MacroArg>,
) -> CelValue {
    let mut mapper = match BoundPredicateExecutor::new(binder, map_expr, ctx) {
        Ok(p) => p,
        Err(e) => return e.into(),
    };
    let mut filter = match filter_expr.map(|f| BoundPredicateExecutor::new(binder, f, ctx)) {
        Some(Ok(p)) => Some(p),
        Some(Err(e)) => return e.into(),
        None => None,
    };
    let mut mapped = Vec::new();

    for value in list.into_iter() {
        if let Some(pred) = filter.as_mut() {
            let predicate = match pred.eval(value.clone()) {
                Ok(val) => val,
                Err(err) => return err.into(),
            };
            if !predicate.is_truthy() {
                continue;
            }
        }

        match mapper.eval(value) {
            Ok(val) => mapped.push(val),
            Err(err) => return err.into(),
        }
    }

    mapped.into()
}

fn map_map(
    ctx: &Interpreter,
    map: std::collections::HashMap<String, CelValue>,
    binder: &MacroArg,
    map_expr: &MacroArg,
    filter_expr: Option<&MacroArg>,
) -> CelValue {
    let mut mapper = match BoundPredicateExecutor::new(binder, map_expr, ctx) {
        Ok(p) => p,
        Err(e) => return e.into(),
    };
    let mut filter = match filter_expr.map(|f| BoundPredicateExecutor::new(binder, f, ctx)) {
        Some(Ok(p)) => Some(p),
        Some(Err(e)) => return e.into(),
        None => None,
    };
    let mut mapped = Vec::new();

    for key in map.into_keys() {
        let value: CelValue = key.into();
        if let Some(pred) = filter.as_mut() {
            let predicate = match pred.eval(value.clone()) {
                Ok(val) => val,
                Err(err) => return err.into(),
            };
            if !predicate.is_truthy() {
                continue;
            }
        }

        match mapper.eval(value) {
            Ok(val) => mapped.push(val),
            Err(err) => return err.into(),
        }
    }

    mapped.into()
}
