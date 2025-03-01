use crate::{
    interp::Interpreter, types::CelByteCode, utils::eval_ident, BindContext, CelContext, CelError,
    CelValue, CelValueDyn,
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

const COMPILE_MACROS: &[(&str, &'static RsCelMacro)] = &[
    ("all", &all_impl),
    ("exists", &exists_impl),
    ("exists_one", &exists_one_impl),
    ("filter", &filter_impl),
    ("map", &map_impl),
    ("reduce", &reduce_impl),
];

pub fn load_default_macros(exec_ctx: &mut BindContext) {
    for (name, macro_) in DEFAULT_MACROS.iter() {
        exec_ctx.bind_macro(name, *macro_)
    }
}

pub fn load_compile_macros(exec_ctx: &mut BindContext) {
    for (name, macro_) in COMPILE_MACROS.iter() {
        exec_ctx.bind_macro(name, *macro_)
    }
}

fn has_impl(ctx: &Interpreter, _this: CelValue, exprlist: &[&CelByteCode]) -> CelValue {
    if exprlist.len() != 1 {
        return CelValue::from_err(CelError::argument("has() macro expects exactly 1 argument"));
    }

    let res = ctx.run_raw(&exprlist[0], true);
    match res {
        Ok(_) => CelValue::true_(),
        Err(err) => match err {
            CelError::Binding { .. } => CelValue::false_(),
            CelError::Attribute { .. } => CelValue::false_(),
            _ => CelValue::from_err(err),
        },
    }
}

fn all_impl(ctx: &Interpreter, this: CelValue, bytecode: &[&CelByteCode]) -> CelValue {
    if bytecode.len() != 2 {
        return CelValue::from_err(CelError::argument(
            "all() macro expects exactly 2 arguments",
        ));
    }

    let ident_name = match eval_ident(bytecode[0]) {
        Ok(s) => s,
        Err(e) => return e.into(),
    };

    if let CelValue::List(list) = this {
        let cel = ctx.cel_copy().unwrap_or_else(|| CelContext::new());
        let mut bindings = ctx.bindings_copy().unwrap_or_else(|| BindContext::new());

        for v in list.into_iter() {
            // make a copy of the context to make borrow checker happy
            bindings.bind_param(&ident_name, v.clone());
            let interp = Interpreter::new(&cel, &bindings);

            let res = match interp.run_raw(bytecode[1], true) {
                Ok(r) => r,
                Err(e) => e.into(),
            };

            if !res.is_truthy() {
                return false.into();
            }
        }

        true.into()
    } else {
        CelValue::from_err(CelError::value("all() only available on list"))
    }
}

fn exists_impl(ctx: &Interpreter, this: CelValue, bytecode: &[&CelByteCode]) -> CelValue {
    if bytecode.len() != 2 {
        return CelValue::from_err(CelError::argument(
            "exists() macro expects exactly 2 arguments",
        ));
    }

    let ident_name = match eval_ident(bytecode[0]) {
        Ok(it) => it,
        Err(err) => return err.into(),
    };
    if let CelValue::List(list) = this {
        let cel = ctx.cel_copy().unwrap_or_else(|| CelContext::new());
        let mut bindings = ctx.bindings_copy().unwrap_or_else(|| BindContext::new());

        for v in list.into_iter() {
            bindings.bind_param(&ident_name, v.clone());
            let interp = Interpreter::new(&cel, &bindings);

            let res = match interp.run_raw(bytecode[1], true) {
                Ok(it) => it,
                Err(err) => return err.into(),
            };

            if res.is_truthy() {
                return true.into();
            }
        }

        false.into()
    } else {
        CelValue::from_err(CelError::value("exists() only available on list"))
    }
}

fn exists_one_impl(ctx: &Interpreter, this: CelValue, bytecode: &[&CelByteCode]) -> CelValue {
    if bytecode.len() != 2 {
        return CelValue::from_err(CelError::argument(
            "exists_one() macro expects exactly 2 arguments",
        ));
    }

    let ident_name = match eval_ident(bytecode[0]) {
        Ok(it) => it,
        Err(err) => return err.into(),
    };

    if let CelValue::List(list) = this {
        let cel = ctx.cel_copy().unwrap_or_else(|| CelContext::new());
        let mut bindings = ctx.bindings_copy().unwrap_or_else(|| BindContext::new());

        let mut count = 0;
        for v in list.into_iter() {
            bindings.bind_param(&ident_name, v.clone());
            let interp = Interpreter::new(&cel, &bindings);

            let res = match interp.run_raw(bytecode[1], true) {
                Ok(it) => it,
                Err(err) => return err.into(),
            };

            if res.is_truthy() {
                count += 1;

                if count > 1 {
                    return false.into();
                }
            }
        }

        (count == 1).into()
    } else {
        CelValue::from_err(CelError::value("exists_one() only available on list"))
    }
}

fn filter_impl(ctx: &Interpreter, this: CelValue, bytecode: &[&CelByteCode]) -> CelValue {
    if bytecode.len() != 2 {
        return CelValue::from_err(CelError::argument(
            "filter() macro expects exactly 2 arguments",
        ));
    }

    let ident_name = match eval_ident(bytecode[0]) {
        Ok(it) => it,
        Err(err) => return err.into(),
    };

    if let CelValue::List(list) = this {
        let mut filtered_list: Vec<CelValue> = Vec::new();
        let cel = ctx.cel_copy().unwrap_or_else(|| CelContext::new());
        let mut bindings = ctx.bindings_copy().unwrap_or_else(|| BindContext::new());

        for v in list.into_iter() {
            // make a copy of the context to make borrow checker happy
            bindings.bind_param(&ident_name, v.clone());
            let interp = Interpreter::new(&cel, &bindings);

            if match interp.run_raw(bytecode[1], true) {
                Ok(it) => it,
                Err(err) => return err.into(),
            }
            .is_truthy()
            {
                filtered_list.push(v.clone());
            }
        }
        filtered_list.into()
    } else if let CelValue::Map(map) = this {
        let mut filtered_list: Vec<CelValue> = Vec::new();
        let cel = ctx.cel_copy().unwrap_or_else(|| CelContext::new());
        let mut bindings = ctx.bindings_copy().unwrap_or_else(|| BindContext::new());

        for v in map.into_keys() {
            let cel_value: CelValue = v.into();
            // make a copy of the context to make borrow checker happy
            bindings.bind_param(&ident_name, cel_value.clone());
            let interp = Interpreter::new(&cel, &bindings);

            if match interp.run_raw(bytecode[1], true) {
                Ok(it) => it,
                Err(err) => return err.into(),
            }
            .is_truthy()
            {
                filtered_list.push(cel_value);
            }
        }
        filtered_list.into()
    } else {
        CelValue::from_err(CelError::value("filter() only available on list"))
    }
}

fn map_impl(ctx: &Interpreter, this: CelValue, bytecode: &[&CelByteCode]) -> CelValue {
    if !(bytecode.len() == 2 || bytecode.len() == 3) {
        return CelValue::from_err(CelError::argument(
            "map() macro expects exactly 2 or 3 arguments",
        ));
    }

    let ident_name = match eval_ident(bytecode[0]) {
        Ok(it) => it,
        Err(err) => return err.into(),
    };

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

                mapped_list.push(match interp.run_raw(bytecode[1], true) {
                    Ok(it) => it,
                    Err(err) => return err.into(),
                });
            }
        } else if bytecode.len() == 3 {
            for v in list.into_iter() {
                bindings.bind_param(&ident_name, v.clone());
                let interp = Interpreter::new(&cel, &bindings);

                if match interp.run_raw(bytecode[1], true) {
                    Ok(it) => it,
                    Err(err) => return err.into(),
                }
                .is_truthy()
                {
                    mapped_list.push(match interp.run_raw(bytecode[2], true) {
                        Ok(it) => it,
                        Err(err) => return err.into(),
                    });
                }
            }
        } else {
            return CelValue::from_err(CelError::internal("Bytecode len check failed"));
        }

        mapped_list.into()
    } else if let CelValue::Map(map) = this {
        let mut mapped_list: Vec<CelValue> = Vec::new();
        let cel = ctx.cel_copy().unwrap_or_else(|| CelContext::new());
        // make a copy of the context to make borrow checker happy
        let mut bindings = ctx.bindings_copy().unwrap_or_else(|| BindContext::new());

        // optimize so we are only checking bytecode's len once
        if bytecode.len() == 2 {
            for v in map.into_keys().map(|k| Into::<CelValue>::into(k)) {
                bindings.bind_param(&ident_name, v.clone());
                let interp = Interpreter::new(&cel, &bindings);

                mapped_list.push(match interp.run_raw(bytecode[1], true) {
                    Ok(it) => it,
                    Err(err) => return err.into(),
                });
            }
        } else if bytecode.len() == 3 {
            for v in map.into_keys().map(|k| Into::<CelValue>::into(k)) {
                bindings.bind_param(&ident_name, v.clone());
                let interp = Interpreter::new(&cel, &bindings);

                if match interp.run_raw(bytecode[1], true) {
                    Ok(it) => it,
                    Err(err) => return err.into(),
                }
                .is_truthy()
                {
                    mapped_list.push(match interp.run_raw(bytecode[2], true) {
                        Ok(it) => it,
                        Err(err) => return err.into(),
                    });
                }
            }
        } else {
            return CelValue::from_err(CelError::internal("Bytecode len check failed"));
        }

        mapped_list.into()
    } else {
        CelValue::from_err(CelError::value("map() only available on list"))
    }
}

// reduce [].reduce(curr, next, expression, starting)
fn reduce_impl(ctx: &Interpreter, this: CelValue, bytecode: &[&CelByteCode]) -> CelValue {
    if bytecode.len() != 4 {
        return CelValue::from_err(CelError::argument("reduce() macro expects 4 arguments"));
    }

    let curr_name = match eval_ident(bytecode[0]) {
        Ok(it) => it,
        Err(err) => return err.into(),
    };
    let next_name = match eval_ident(bytecode[1]) {
        Ok(it) => it,
        Err(err) => return err.into(),
    };
    let mut cur_value = match ctx.run_raw(bytecode[3], true) {
        Ok(it) => it,
        Err(err) => return err.into(),
    };
    let cel = ctx.cel_copy().unwrap_or_else(|| CelContext::new());
    let mut bindings = ctx.bindings_copy().unwrap_or_else(|| BindContext::new());

    if let CelValue::List(list) = this {
        for next in list.into_iter() {
            bindings.bind_param(&next_name, next);
            bindings.bind_param(&curr_name, cur_value);

            let interp = Interpreter::new(&cel, &bindings);

            cur_value = match interp.run_raw(bytecode[2], true) {
                Ok(it) => it,
                Err(err) => return err.into(),
            };
        }

        cur_value
    } else {
        CelValue::from_err(CelError::value("reduce() only availble on list"))
    }
}

fn coalesce_impl(ctx: &Interpreter, _this: CelValue, bytecode: &[&CelByteCode]) -> CelValue {
    for arg in bytecode.iter() {
        let res = ctx.run_raw(arg, true);
        match res {
            Ok(CelValue::Null) => {}
            Err(err) => match err {
                CelError::Binding { .. } | CelError::Attribute { .. } => {}
                _ => return CelValue::from_err(err),
            },
            Ok(v) => return v,
        };
    }

    CelValue::from_null()
}
