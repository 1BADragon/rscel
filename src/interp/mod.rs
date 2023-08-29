mod types;
use std::{collections::HashMap, fmt};
pub use types::{ByteCode, JmpWhen};

use crate::{
    utils::ScopedCounter, BindContext, CelContext, CelError, CelResult, CelValue, RsCelFunction,
    RsCelMacro,
};

use types::CelStackValue;

use self::types::RsCallable;

struct InterpStack<'a, 'b> {
    stack: Vec<CelStackValue<'b>>,

    ctx: &'a Interpreter<'b>,
}

impl<'a, 'b> InterpStack<'a, 'b> {
    fn new(ctx: &'b Interpreter) -> InterpStack<'a, 'b> {
        InterpStack {
            stack: Vec::new(),
            ctx,
        }
    }

    fn push(&mut self, val: CelStackValue<'b>) {
        self.stack.push(val);
    }

    fn push_val(&mut self, val: CelValue) {
        self.stack.push(CelStackValue::Value(val));
    }

    fn pop(&mut self) -> CelResult<CelStackValue> {
        match self.stack.pop() {
            Some(stack_val) => {
                if let CelStackValue::Value(val) = stack_val {
                    if let CelValue::Ident(name) = val {
                        if let Some(val) = self.ctx.get_param_by_name(&name) {
                            return Ok(CelStackValue::Value(val.clone()));
                        } else if let Some(ctx) = self.ctx.cel {
                            // Allow for loaded programs to run as values
                            if let Some(prog) = ctx.get_program(&name) {
                                return self.ctx.run_raw(prog.bytecode()).map(|x| x.into());
                            }
                        }

                        return Err(CelError::binding(&name));
                    } else {
                        Ok(val.into())
                    }
                } else {
                    Ok(stack_val)
                }
            }
            None => Err(CelError::runtime("No value on stack!")),
        }
    }

    fn pop_val(&mut self) -> CelResult<CelValue> {
        self.pop()?.into_value()
    }

    fn pop_noresolve(&mut self) -> CelResult<CelStackValue<'b>> {
        match self.stack.pop() {
            Some(val) => Ok(val),
            None => Err(CelError::runtime("No value on stack!")),
        }
    }

    fn pop_tryresolve(&mut self) -> CelResult<CelStackValue<'b>> {
        match self.stack.pop() {
            Some(val) => match val.try_into()? {
                CelValue::Ident(name) => {
                    if let Some(val) = self.ctx.get_param_by_name(&name) {
                        Ok(val.clone().into())
                    } else {
                        Ok(CelStackValue::Value(CelValue::from_ident(&name)))
                    }
                }
                other => Ok(CelStackValue::Value(other.into())),
            },
            None => Err(CelError::runtime("No value on stack!")),
        }
    }
}

impl<'a, 'b> fmt::Debug for InterpStack<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.stack)
    }
}

pub struct Interpreter<'a> {
    cel: Option<&'a CelContext>,
    bindings: Option<&'a BindContext<'a>>,
    depth: ScopedCounter,
}

impl<'a> Interpreter<'a> {
    pub fn new(cel: &'a CelContext, bindings: &'a BindContext) -> Interpreter<'a> {
        Interpreter {
            cel: Some(cel),
            bindings: Some(bindings),
            depth: ScopedCounter::new(),
        }
    }

    pub fn empty() -> Interpreter<'a> {
        Interpreter {
            cel: None,
            bindings: None,
            depth: ScopedCounter::new(),
        }
    }

    pub fn add_bindings(&mut self, bindings: &'a BindContext) {
        self.bindings = Some(bindings);
    }

    pub fn cel_copy(&self) -> Option<CelContext> {
        self.cel.cloned()
    }

    pub fn bindings_copy(&self) -> Option<BindContext> {
        self.bindings.cloned()
    }

    pub fn run_program(&self, name: &str) -> CelResult<CelValue> {
        match self.cel {
            Some(cel) => match cel.get_program(name) {
                Some(prog) => self.run_raw(prog.bytecode()),
                None => Err(CelError::binding(&name)),
            },
            None => Err(CelError::internal("No CEL context bound to interpreter")),
        }
    }

    pub fn run_raw(&self, prog: &[ByteCode]) -> CelResult<CelValue> {
        let mut pc: usize = 0;
        let mut stack = InterpStack::new(self);

        let count = self.depth.inc();

        if count.count() > 32 {
            return Err(CelError::runtime("Max call depth excceded"));
        }

        while pc < prog.len() {
            let oldpc = pc;
            pc += 1;
            match &prog[oldpc] {
                ByteCode::Push(val) => stack.push(val.clone().into()),
                ByteCode::Or => {
                    let v2 = stack.pop_val()?;
                    let v1 = stack.pop_val()?;

                    stack.push_val(v1.or(&v2)?)
                }
                ByteCode::And => {
                    let v2 = stack.pop_val()?;
                    let v1 = stack.pop_val()?;

                    stack.push_val(v1.and(&v2)?)
                }
                ByteCode::Not => {
                    let v1 = stack.pop_val()?;

                    stack.push_val((!v1)?);
                }
                ByteCode::Neg => {
                    let v1 = stack.pop_val()?;

                    stack.push_val((-v1)?);
                }
                ByteCode::Add => {
                    let v2 = stack.pop_val()?;
                    let v1 = stack.pop_val()?;

                    stack.push_val((v1 + v2)?);
                }
                ByteCode::Sub => {
                    let v2 = stack.pop_val()?;
                    let v1 = stack.pop_val()?;

                    stack.push_val((v1 - v2)?);
                }
                ByteCode::Mul => {
                    let v2 = stack.pop_val()?;
                    let v1 = stack.pop_val()?;

                    stack.push_val((v1 * v2)?);
                }
                ByteCode::Div => {
                    let v2 = stack.pop_val()?;
                    let v1 = stack.pop_val()?;

                    stack.push_val((v1 / v2)?);
                }
                ByteCode::Mod => {
                    let v2 = stack.pop_val()?;
                    let v1 = stack.pop_val()?;

                    stack.push_val((v1 % v2)?);
                }
                ByteCode::Jmp(dist) => pc = pc + *dist as usize,
                ByteCode::JmpCond {
                    when,
                    dist,
                    leave_val,
                } => {
                    let v1 = stack.pop_val()?;
                    match when {
                        JmpWhen::True => {
                            if let CelValue::Bool(v) = v1 {
                                if v {
                                    pc += *dist as usize
                                }
                            } else {
                                return Err(CelError::invalid_op(&format!(
                                    "JMP TRUE invalid on type {:?}",
                                    v1.as_type()
                                )));
                            }
                        }
                        JmpWhen::False => {
                            if let CelValue::Bool(v) = v1 {
                                if !v {
                                    pc += *dist as usize
                                }
                            } else {
                                return Err(CelError::invalid_op(&format!(
                                    "JMP FALSE invalid on type {:?}",
                                    v1.as_type()
                                )));
                            }
                        }
                    };
                    if *leave_val {
                        stack.push_val(v1);
                    }
                }
                ByteCode::Lt => {
                    let v2 = stack.pop_val()?;
                    let v1 = stack.pop_val()?;

                    stack.push_val(v1.lt(&v2)?);
                }
                ByteCode::Le => {
                    let v2 = stack.pop_val()?;
                    let v1 = stack.pop_val()?;

                    stack.push_val(v1.le(&v2)?);
                }
                ByteCode::Eq => {
                    let v2 = stack.pop_val()?;
                    let v1 = stack.pop_val()?;

                    stack.push_val(v1.eq(&v2)?);
                }
                ByteCode::Ne => {
                    let v2 = stack.pop_val()?;
                    let v1 = stack.pop_val()?;

                    stack.push_val(v1.neq(&v2)?);
                }
                ByteCode::Ge => {
                    let v2 = stack.pop_val()?;
                    let v1 = stack.pop_val()?;

                    stack.push_val(v1.ge(&v2)?);
                }
                ByteCode::Gt => {
                    let v2 = stack.pop_val()?;
                    let v1 = stack.pop_val()?;

                    stack.push_val(v1.gt(&v2)?);
                }
                ByteCode::In => {
                    let rhs = stack.pop_val()?;
                    let lhs = stack.pop_val()?;

                    let rhs_type = rhs.as_type();
                    let lhs_type = lhs.as_type();

                    match rhs {
                        CelValue::List(l) => 'outer: {
                            for value in l.iter() {
                                if lhs == *value {
                                    stack.push_val(true.into());
                                    break 'outer;
                                }
                            }

                            stack.push_val(false.into());
                        }
                        CelValue::Map(m) => {
                            if let CelValue::String(r) = lhs {
                                stack.push_val(CelValue::from_bool(m.contains_key(&r)));
                            } else {
                                return Err(CelError::invalid_op(&format!(
                                    "Op 'in' invalid between {:?} and {:?}",
                                    lhs_type, rhs_type
                                )));
                            }
                        }
                        _ => {
                            return Err(CelError::invalid_op(&format!(
                                "Op 'in' invalid between {:?} and {:?}",
                                lhs_type, rhs_type
                            )));
                        }
                    }
                }
                ByteCode::MkList(size) => {
                    let mut v = Vec::new();

                    for _ in 0..*size {
                        v.push(stack.pop_val()?)
                    }

                    v.reverse();
                    stack.push_val(v.into());
                }
                ByteCode::MkDict(size) => {
                    let mut map = HashMap::new();

                    for _ in 0..*size {
                        let key = if let CelValue::String(key) = stack.pop_val()? {
                            key
                        } else {
                            return Err(CelError::value("Only strings can be used as Object keys"));
                        };

                        map.insert(key, stack.pop_val()?);
                    }

                    stack.push_val(map.into());
                }
                ByteCode::Index => {
                    let index = stack.pop_val()?;
                    let obj = stack.pop_val()?;

                    if let CelValue::List(list) = obj {
                        let index = if let CelValue::UInt(index) = index {
                            index as usize
                        } else if let CelValue::Int(index) = index {
                            if index < 0 {
                                return Err(CelError::value("Negative index is not allowed"));
                            }
                            index as usize
                        } else {
                            return Err(CelError::value("List index can only be int or uint"));
                        };

                        if index >= list.len() {
                            return Err(CelError::value("List access out of bounds"));
                        }

                        stack.push_val(list[index].clone());
                    } else if let CelValue::Map(map) = obj {
                        if let CelValue::String(index) = index {
                            match map.get(&index) {
                                Some(val) => stack.push_val(val.clone()),
                                None => {
                                    return Err(CelError::value(&format!(
                                        "Object does not contain key \"{}\"",
                                        index
                                    )))
                                }
                            }
                        }
                    } else {
                        return Err(CelError::value(&format!(
                            "Index operator invalide between {:?} and {:?}",
                            index.as_type(),
                            obj.as_type()
                        )));
                    }
                }
                ByteCode::Access => {
                    let index = stack.pop_noresolve()?;

                    if let CelValue::Ident(ident) = index.as_value()? {
                        let obj = stack.pop()?.into_value()?;
                        if let CelValue::Map(ref map) = obj {
                            match map.get(ident.as_str()) {
                                Some(val) => stack.push_val(val.clone()),
                                None => stack.push(CelStackValue::BoundCall {
                                    callable: self.callable_by_name(ident.as_str())?,
                                    value: obj,
                                }),
                            }
                        } else {
                            stack.push(CelStackValue::BoundCall {
                                callable: self.callable_by_name(ident.as_str())?,
                                value: obj,
                            });
                        }
                    } else {
                        let obj = stack.pop()?;
                        return Err(CelError::value(&format!(
                            "Index operator invalid between {:?} and {:?}",
                            index.into_value()?.as_type(),
                            obj.into_value()?.as_type()
                        )));
                    }
                }
                ByteCode::Call(n_args) => {
                    let mut args = Vec::new();

                    for _ in 0..*n_args {
                        args.push(stack.pop()?.into_value()?)
                    }

                    match stack.pop_noresolve()? {
                        CelStackValue::BoundCall { callable, value } => match callable {
                            RsCallable::Function(func) => {
                                let arg_values = self.resolve_args(args)?;
                                stack.push_val(func(value, &arg_values)?);
                            }
                            RsCallable::Macro(macro_) => {
                                stack.push_val(self.call_macro(&value, &args, macro_)?);
                            }
                        },
                        CelStackValue::Value(value) => match value {
                            CelValue::Ident(func_name) => {
                                if let Some(func) = self.get_func_by_name(&func_name) {
                                    let arg_values = self.resolve_args(args)?;
                                    stack.push_val(func(CelValue::from_null(), &arg_values)?);
                                } else if let Some(macro_) = self.get_macro_by_name(&func_name) {
                                    stack.push_val(self.call_macro(
                                        &CelValue::from_null(),
                                        &args,
                                        macro_,
                                    )?);
                                } else {
                                    return Err(CelError::runtime(&format!(
                                        "{} is not callable",
                                        func_name
                                    )));
                                }
                            }
                            _ => return Err(CelError::runtime("only idents are callable")),
                        },
                    };
                }
            };
        }

        let val = stack.pop_tryresolve();
        match val {
            Ok(val) => val.try_into(),
            Err(err) => Err(err),
        }
    }

    fn call_macro(
        &self,
        this: &CelValue,
        args: &Vec<CelValue>,
        macro_: &RsCelMacro,
    ) -> Result<CelValue, CelError> {
        let mut v = Vec::new();
        for arg in args.iter() {
            if let CelValue::ByteCode(bc) = arg {
                v.push(bc.as_slice());
            } else {
                return Err(CelError::internal("macro args must be bytecode"));
            }
        }
        let res = macro_(self, this.clone(), &v)?;
        Ok(res)
    }

    fn resolve_args(&self, args: Vec<CelValue>) -> Result<Vec<CelValue>, CelError> {
        let mut arg_values = Vec::new();
        for arg in args.into_iter() {
            if let CelValue::ByteCode(bc) = arg {
                arg_values.push(self.run_raw(&bc)?);
            } else {
                arg_values.push(arg)
            }
        }
        Ok(arg_values)
    }

    fn get_param_by_name(&self, name: &str) -> Option<&'a CelValue> {
        self.bindings?.get_param(name)
    }

    fn get_func_by_name(&self, name: &str) -> Option<&'a RsCelFunction> {
        self.bindings?.get_func(name)
    }

    fn get_macro_by_name(&self, name: &str) -> Option<&'a RsCelMacro> {
        self.bindings?.get_macro(name)
    }

    fn callable_by_name(&self, name: &str) -> CelResult<RsCallable> {
        if let Some(func) = self.get_func_by_name(name) {
            Ok(RsCallable::Function(func))
        } else if let Some(macro_) = self.get_macro_by_name(name) {
            Ok(RsCallable::Macro(macro_))
        } else {
            Err(CelError::value(&format!("{} is not callable", name)))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::CelValue;

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
    fn test_interp_ops(op: ByteCode, expected: CelValue) {
        let mut prog = vec![ByteCode::Push(4.into()), ByteCode::Push(3.into())];
        prog.push(op);
        let interp = Interpreter::empty();

        assert!(interp.run_raw(&prog).unwrap() == expected);
    }
}
