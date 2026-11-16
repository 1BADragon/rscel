use crate::{interp::Interpreter, BindContext, CelContext, CelResult, CelValue, MacroArg};

pub mod helpers;

pub struct BoundPredicateExecutor<'a> {
    cel: CelContext,
    bindings: BindContext<'a>,
    ident_name: String,
    predicate: &'a MacroArg<'a>,
}

impl<'a> BoundPredicateExecutor<'a> {
    pub fn new(
        ident_name_param: &MacroArg,
        predicate: &'a MacroArg<'a>,
        interp: &'a Interpreter<'a>,
    ) -> CelResult<Self> {
        let ident_name = ident_name_param.as_binder()?.to_owned();

        let (cel, bindings) = helpers::child_scope(interp);

        Ok(BoundPredicateExecutor {
            cel,
            bindings,
            ident_name,
            predicate,
        })
    }

    pub fn eval(&mut self, value: CelValue) -> CelResult<CelValue> {
        self.bindings.bind_param(&self.ident_name, value);

        let interp = Interpreter::new(&self.cel, &self.bindings);

        self.predicate.eval(&interp)
    }
}
