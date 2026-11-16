use crate::context::bound_predicate_executor::BoundPredicateExecutor;
use crate::interp::Interpreter;
use crate::types::MacroArg;
use crate::{CelError, CelValue, CelValueDyn};

pub fn filter_impl(ctx: &Interpreter, this: CelValue, args: &[MacroArg]) -> CelValue {
    if args.len() != 2 {
        return CelValue::from_err(CelError::argument(
            "filter() macro expects exactly 2 arguments",
        ));
    }

    match this {
        CelValue::List(list) => filter_list(ctx, list, &args[0], &args[1]),
        CelValue::Map(map) => filter_map(ctx, map, &args[0], &args[1]),
        CelValue::Err(e) => CelValue::Err(e),
        _ => CelValue::from_err(CelError::value("filter() only available on list")),
    }
}

fn filter_list(
    ctx: &Interpreter,
    list: Vec<CelValue>,
    binder: &MacroArg,
    predicate: &MacroArg,
) -> CelValue {
    let mut p = match BoundPredicateExecutor::new(binder, predicate, ctx) {
        Ok(p) => p,
        Err(e) => return e.into(),
    };
    let mut filtered_list = Vec::new();

    for value in list.into_iter() {
        let res = match p.eval(value.clone()) {
            Ok(val) => val,
            Err(err) => return err.into(),
        };

        if res.is_truthy() {
            filtered_list.push(value);
        }
    }

    filtered_list.into()
}

fn filter_map(
    ctx: &Interpreter,
    map: std::collections::HashMap<String, CelValue>,
    binder: &MacroArg,
    predicate: &MacroArg,
) -> CelValue {
    let mut p = match BoundPredicateExecutor::new(binder, predicate, ctx) {
        Ok(p) => p,
        Err(e) => return e.into(),
    };
    let mut filtered_list = Vec::new();

    for key in map.into_keys() {
        let value: CelValue = key.into();
        let res = match p.eval(value.clone()) {
            Ok(val) => val,
            Err(err) => return err.into(),
        };

        if res.is_truthy() {
            filtered_list.push(value);
        }
    }

    filtered_list.into()
}
