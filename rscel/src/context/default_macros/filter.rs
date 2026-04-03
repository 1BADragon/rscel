use super::helpers;
use crate::interp::Interpreter;
use crate::types::MacroArg;
use crate::{CelError, CelValue, CelValueDyn};

pub fn filter_impl(ctx: &Interpreter, this: CelValue, args: &[MacroArg]) -> CelValue {
    if args.len() != 2 {
        return CelValue::from_err(CelError::argument(
            "filter() macro expects exactly 2 arguments",
        ));
    }

    let ident_name = match args[0].as_binder() {
        Ok(s) => s.to_owned(),
        Err(e) => return e.into(),
    };

    match this {
        CelValue::List(list) => filter_list(ctx, list, &ident_name, &args[1]),
        CelValue::Map(map) => filter_map(ctx, map, &ident_name, &args[1]),
        CelValue::Err(e) => CelValue::Err(e),
        _ => CelValue::from_err(CelError::value("filter() only available on list")),
    }
}

fn filter_list(
    ctx: &Interpreter,
    list: Vec<CelValue>,
    ident_name: &str,
    predicate: &MacroArg,
) -> CelValue {
    let (cel, mut bindings) = helpers::child_scope(ctx);
    let mut filtered_list = Vec::new();

    for value in list.into_iter() {
        bindings.bind_param(ident_name, value.clone());
        let interp = Interpreter::new(&cel, &bindings);

        let res = match predicate.eval(&interp) {
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
    ident_name: &str,
    predicate: &MacroArg,
) -> CelValue {
    let (cel, mut bindings) = helpers::child_scope(ctx);
    let mut filtered_list = Vec::new();

    for key in map.into_keys() {
        let value: CelValue = key.into();
        bindings.bind_param(ident_name, value.clone());
        let interp = Interpreter::new(&cel, &bindings);

        let res = match predicate.eval(&interp) {
            Ok(val) => val,
            Err(err) => return err.into(),
        };

        if res.is_truthy() {
            filtered_list.push(value);
        }
    }

    filtered_list.into()
}
