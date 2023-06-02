mod types;
use std::{collections::HashMap, fmt};
pub use types::{ByteCode, JmpWhen};

use crate::{
    context::RsCallable, BindContext, CelContext, RsCellFunction, RsCellMacro, ValueCell,
    ValueCellError, ValueCellInner, ValueCellResult,
};

struct InterpStack<'a> {
    stack: Vec<ValueCell>,

    ctx: &'a Interpreter<'a>,
}

impl<'a> InterpStack<'a> {
    fn new(ctx: &'a Interpreter) -> InterpStack<'a> {
        InterpStack {
            stack: Vec::new(),
            ctx,
        }
    }

    fn push(&mut self, val: ValueCell) {
        self.stack.push(val)
    }

    fn pop(&mut self) -> ValueCellResult<ValueCell> {
        match self.stack.pop() {
            Some(val) => {
                if let ValueCellInner::Ident(name) = val.inner() {
                    if let Some(val) = self.ctx.get_param_by_name(name) {
                        Ok(val.clone())
                    } else {
                        Err(ValueCellError::with_msg(&format!(
                            "Ident {} is not bound",
                            name
                        )))
                    }
                } else {
                    Ok(val)
                }
            }
            None => Err(ValueCellError::with_msg("No value on stack!")),
        }
    }

    fn pop_noresolve(&mut self) -> ValueCellResult<ValueCell> {
        match self.stack.pop() {
            Some(val) => Ok(val),
            None => Err(ValueCellError::with_msg("No value on stack!")),
        }
    }

    fn len(&self) -> usize {
        self.stack.len()
    }
}

impl<'a> fmt::Debug for InterpStack<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.stack)
    }
}

pub struct Interpreter<'a> {
    cel: Option<&'a CelContext>,
    bindings: Option<&'a BindContext>,
}

impl<'a> Interpreter<'a> {
    pub fn new(cel: &'a CelContext, bindings: &'a BindContext) -> Interpreter<'a> {
        Interpreter {
            cel: Some(cel),
            bindings: Some(bindings),
        }
    }

    fn empty() -> Interpreter<'a> {
        Interpreter {
            cel: None,
            bindings: None,
        }
    }

    pub fn cel_copy(&self) -> Option<CelContext> {
        self.cel.cloned()
    }

    pub fn bindings_copy(&self) -> Option<BindContext> {
        self.bindings.cloned()
    }

    pub fn run_program(&self, name: &str) -> ValueCellResult<ValueCell> {
        match self.cel {
            Some(cel) => match cel.get_program(name) {
                Some(prog) => self.run_raw(prog.bytecode()),
                None => Err(ValueCellError::with_msg(&format!(
                    "No program named {} bound",
                    name
                ))),
            },
            None => Err(ValueCellError::with_msg(
                "No CEL context bound to interpreter",
            )),
        }
    }

    pub fn run_raw(&self, prog: &[ByteCode]) -> ValueCellResult<ValueCell> {
        println!("Running prog: {:?}", prog);

        let mut pc: usize = 0;
        let mut stack = InterpStack::new(self);

        while pc < prog.len() {
            let oldpc = pc;
            pc += 1;
            match &prog[oldpc] {
                ByteCode::Push(val) => stack.push(val.clone()),
                ByteCode::Or => {
                    let v2 = stack.pop()?;
                    let v1 = stack.pop()?;

                    stack.push(v1.or(&v2)?)
                }
                ByteCode::And => {
                    let v2 = stack.pop()?;
                    let v1 = stack.pop()?;

                    stack.push(v1.and(&v2)?)
                }
                ByteCode::Not => {
                    let v1 = stack.pop()?;

                    stack.push((!v1)?);
                }
                ByteCode::Neg => {
                    let v1 = stack.pop()?;

                    stack.push((-v1)?);
                }
                ByteCode::Add => {
                    let v2 = stack.pop()?;
                    let v1 = stack.pop()?;

                    stack.push((v1 + v2)?);
                }
                ByteCode::Sub => {
                    let v2 = stack.pop()?;
                    let v1 = stack.pop()?;

                    stack.push((v1 - v2)?);
                }
                ByteCode::Mul => {
                    let v2 = stack.pop()?;
                    let v1 = stack.pop()?;

                    stack.push((v1 * v2)?);
                }
                ByteCode::Div => {
                    let v2 = stack.pop()?;
                    let v1 = stack.pop()?;

                    stack.push((v1 / v2)?);
                }
                ByteCode::Mod => {
                    let v2 = stack.pop()?;
                    let v1 = stack.pop()?;

                    stack.push((v1 % v2)?);
                }
                ByteCode::Jmp(dist) => pc = pc + *dist as usize,
                ByteCode::JmpCond {
                    when,
                    dist,
                    leave_val,
                } => {
                    let v1 = stack.pop()?;
                    match when {
                        JmpWhen::True => {
                            if let ValueCellInner::Bool(v) = v1.inner() {
                                if *v {
                                    pc += *dist as usize
                                }
                            } else {
                                return Err(ValueCellError::with_msg(&format!(
                                    "JMP TRUE invalid on type {:?}",
                                    v1.as_type()
                                )));
                            }
                        }
                        JmpWhen::False => {
                            if let ValueCellInner::Bool(v) = v1.inner() {
                                if !v {
                                    pc += *dist as usize
                                }
                            } else {
                                return Err(ValueCellError::with_msg(&format!(
                                    "JMP FALSE invalid on type {:?}",
                                    v1.as_type()
                                )));
                            }
                        }
                    };
                    if *leave_val {
                        stack.push(v1);
                    }
                }
                ByteCode::Lt => {
                    let v2 = stack.pop()?;
                    let v1 = stack.pop()?;

                    stack.push(v1.lt(&v2)?);
                }
                ByteCode::Le => {
                    let v2 = stack.pop()?;
                    let v1 = stack.pop()?;

                    stack.push(v1.le(&v2)?);
                }
                ByteCode::Eq => {
                    let v2 = stack.pop()?;
                    let v1 = stack.pop()?;

                    stack.push(v1.eq(&v2)?);
                }
                ByteCode::Ne => {
                    let v2 = stack.pop()?;
                    let v1 = stack.pop()?;

                    stack.push(v1.neq(&v2)?);
                }
                ByteCode::Ge => {
                    let v2 = stack.pop()?;
                    let v1 = stack.pop()?;

                    stack.push(v1.ge(&v2)?);
                }
                ByteCode::Gt => {
                    let v2 = stack.pop()?;
                    let v1 = stack.pop()?;

                    stack.push(v1.gt(&v2)?);
                }
                ByteCode::In => {
                    let rhs = stack.pop()?;
                    let lhs = stack.pop()?;

                    let rhs_type = rhs.as_type();
                    let lhs_type = lhs.as_type();

                    match rhs.inner() {
                        ValueCellInner::List(l) => 'outer: {
                            for value in l.iter() {
                                if lhs == *value {
                                    stack.push(true.into());
                                    break 'outer;
                                }
                            }

                            stack.push(false.into());
                        }
                        ValueCellInner::Map(m) => {
                            if let ValueCellInner::String(r) = lhs.inner() {
                                stack.push(ValueCell::from_bool(m.contains_key(r)));
                            } else {
                                return Err(ValueCellError::with_msg(&format!(
                                    "Op 'in' invalid between {:?} and {:?}",
                                    lhs_type, rhs_type
                                )));
                            }
                        }
                        _ => {
                            return Err(ValueCellError::with_msg(&format!(
                                "Op 'in' invalid between {:?} and {:?}",
                                lhs_type, rhs_type
                            )));
                        }
                    }
                }
                ByteCode::MkList(size) => {
                    let mut v = Vec::new();

                    for _ in 0..*size {
                        v.push(stack.pop()?)
                    }

                    v.reverse();
                    stack.push(v.into());
                }
                ByteCode::MkDict(size) => {
                    let mut map = HashMap::new();

                    for _ in 0..*size {
                        let key = if let ValueCellInner::String(key) = stack.pop()?.into_inner() {
                            key
                        } else {
                            return Err(ValueCellError::with_msg(
                                "Only strings can be used as Object keys",
                            ));
                        };

                        map.insert(key, stack.pop()?);
                    }

                    stack.push(map.into());
                }
                ByteCode::Index => {
                    let index = stack.pop()?;
                    let obj = stack.pop()?;

                    if let ValueCellInner::List(list) = obj.inner() {
                        let index = if let ValueCellInner::UInt(index) = index.inner() {
                            *index as usize
                        } else if let ValueCellInner::Int(index) = index.inner() {
                            if *index < 0 {
                                return Err(ValueCellError::with_msg(
                                    "Negative index is not allowed",
                                ));
                            }
                            *index as usize
                        } else {
                            return Err(ValueCellError::with_msg(
                                "List index can only be int or uint",
                            ));
                        };

                        if index >= list.len() {
                            return Err(ValueCellError::with_msg("List access out of bounds"));
                        }

                        stack.push(list[index].clone());
                    } else if let ValueCellInner::Map(map) = obj.inner() {
                        if let ValueCellInner::String(index) = index.inner() {
                            match map.get(index) {
                                Some(val) => stack.push(val.clone()),
                                None => {
                                    return Err(ValueCellError::with_msg(&format!(
                                        "Object does not contain key \"{}\"",
                                        index
                                    )))
                                }
                            }
                        }
                    } else {
                        return Err(ValueCellError::with_msg(&format!(
                            "Index operator invalide between {:?} and {:?}",
                            index.as_type(),
                            obj.as_type()
                        )));
                    }
                }
                ByteCode::Access => {
                    let index = stack.pop_noresolve()?;
                    let obj = stack.pop()?;

                    if let ValueCellInner::Ident(ident) = index.inner() {
                        if let ValueCellInner::Map(map) = obj.inner() {
                            match map.get(ident.as_str()) {
                                Some(val) => stack.push(val.clone()),
                                None => {
                                    if let Some(func) = self.get_func_by_name(ident.as_str()) {
                                        stack.push(ValueCell::from_binding(
                                            RsCallable::Function(func),
                                            &obj,
                                        ));
                                    } else if let Some(macro_) =
                                        self.get_macro_by_name(ident.as_str())
                                    {
                                        stack.push(ValueCell::from_binding(
                                            RsCallable::Macro(macro_),
                                            &obj,
                                        ));
                                    } else {
                                        return Err(ValueCellError::with_msg(&format!(
                                            "Object does not contain key \"{}\"",
                                            ident
                                        )));
                                    }
                                }
                            }
                        } else {
                            if let Some(func) = self.get_func_by_name(ident.as_str()) {
                                stack
                                    .push(ValueCell::from_binding(RsCallable::Function(func), &obj))
                            } else if let Some(macro_) = self.get_macro_by_name(ident.as_str()) {
                                stack
                                    .push(ValueCell::from_binding(RsCallable::Macro(macro_), &obj));
                            } else {
                                return Err(ValueCellError::with_msg(&format!(
                                    "Index operator invalide between {:?} and {:?}",
                                    index.as_type(),
                                    obj.as_type()
                                )));
                            }
                        }
                    } else {
                        return Err(ValueCellError::with_msg(&format!(
                            "Index operator invalid between {:?} and {:?}",
                            index.as_type(),
                            obj.as_type()
                        )));
                    }
                }
                ByteCode::Call(n_args) => {
                    let mut args = Vec::new();

                    for _ in 0..*n_args {
                        args.push(stack.pop()?)
                    }

                    match stack.pop_noresolve()?.into_inner() {
                        ValueCellInner::Ident(func_name) => {
                            if let Some(func) = self.get_func_by_name(&func_name) {
                                let arg_values = self.resolve_args(args)?;
                                stack.push(func(ValueCell::from_null(), arg_values.into())?);
                            } else {
                                return Err(ValueCellError::with_msg(&format!(
                                    "{} is not callable",
                                    func_name
                                )));
                            }
                        }
                        ValueCellInner::BoundCall { callable, value } => match callable {
                            RsCallable::Function(func) => {
                                let arg_values = self.resolve_args(args)?;
                                stack.push(func(value, arg_values.into())?);
                            }
                            RsCallable::Macro(macro_) => {
                                let mut v = Vec::new();
                                for arg in args.iter() {
                                    if let ValueCellInner::ByteCode(bc) = arg.inner() {
                                        v.push(bc.as_slice());
                                    } else {
                                        return Err(ValueCellError::with_msg(
                                            "macro args must be bytecode",
                                        ));
                                    }
                                }
                                stack.push(macro_(self, value, &v)?);
                            }
                        },
                        _ => return Err(ValueCellError::with_msg("only idents are callable")),
                    };
                }
            };
        }

        Ok(stack.pop().unwrap())
    }

    fn resolve_args(&self, args: Vec<ValueCell>) -> Result<Vec<ValueCell>, ValueCellError> {
        let mut arg_values = Vec::new();
        for arg in args.into_iter() {
            if let ValueCellInner::ByteCode(bc) = arg.inner() {
                arg_values.push(self.run_raw(&bc)?);
            } else {
                arg_values.push(arg)
            }
        }
        Ok(arg_values)
    }

    fn get_param_by_name<'l>(&'l self, name: &str) -> Option<&'l ValueCell> {
        self.bindings?.get_param(name)
    }

    fn get_func_by_name(&self, name: &str) -> Option<RsCellFunction> {
        self.bindings?.get_func(name)
    }
    fn get_macro_by_name(&self, name: &str) -> Option<RsCellMacro> {
        self.bindings?.get_macro(name)
    }

    fn resolve_fqn(&self, fqn: &[ValueCell]) -> ValueCellResult<ValueCell> {
        let mut iter = fqn.iter();

        let mut current = if let Some(vc) = iter.next() {
            match vc.inner() {
                ValueCellInner::Ident(ident) => match self.get_param_by_name(ident) {
                    Some(val) => val.clone(),
                    None => {
                        return Err(ValueCellError::with_msg(&format!(
                            "Ident '{}' does not exist",
                            ident
                        )))
                    }
                },
                other => other.clone().into(),
            }
        } else {
            return Err(ValueCellError::with_msg("Empty Ident"));
        };

        for member_name in iter {
            match current.inner() {
                ValueCellInner::Map(map) => {
                    if let ValueCellInner::Ident(member_name_str) = member_name.inner() {
                        current = if let Some(member) = map.get(member_name_str) {
                            member.clone()
                        } else {
                            return Err(ValueCellError::with_msg(&format!(
                                "member {} does not exist on {:?}",
                                member_name_str, &current
                            )));
                        }
                    } else {
                        return Err(ValueCellError::with_msg(
                            "Only idents can be member accesses",
                        ));
                    }
                }
                _ => {
                    return Err(ValueCellError::with_msg(&format!(
                        "member access invalid on {:?}",
                        current
                    )))
                }
            }
        }

        return Ok(current);
    }
}

#[cfg(test)]
mod test {
    use crate::ValueCell;

    use super::{types::ByteCode, Interpreter};
    use test_case::test_case;

    #[test_case(ByteCode::Add, 7.into())]
    #[test_case(ByteCode::Sub, 1.into())]
    #[test_case(ByteCode::Mul, 12.into())]
    #[test_case(ByteCode::Div, 1.into())]
    #[test_case(ByteCode::Mod, 1.into())]
    #[test_case(ByteCode::Lt, false.into())]
    #[test_case(ByteCode::Le, false.into())]
    #[test_case(ByteCode::Eq, false.into())]
    #[test_case(ByteCode::Ne, true.into())]
    #[test_case(ByteCode::Ge, true.into())]
    #[test_case(ByteCode::Gt, true.into())]
    fn test_interp_ops(op: ByteCode, expected: ValueCell) {
        let mut prog = vec![ByteCode::Push(4.into()), ByteCode::Push(3.into())];
        prog.push(op);
        let interp = Interpreter::empty();

        assert!(interp.run_raw(&prog).unwrap() == expected);
    }
}
