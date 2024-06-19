use std::collections::HashSet;

use rscel::{
    Addition, AstNode, BindContext, CelContext, CelResult, CelValue, ConditionalAnd, ConditionalOr,
    Expr, ExprList, Ident, Member, MemberPrime, Multiplication, NegList, NotList, ObjInit,
    ObjInits, Primary, Relation, Unary,
};

fn main() {
    let args: Vec<_> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <prog> [bindings]", args[0]);
        return;
    }

    let bindings: serde_json::Value = if args.len() == 3 {
        serde_json::from_str(&args[2]).expect("Failed to parse bindings")
    } else {
        serde_json::Value::default()
    };

    let prog = rscel::Program::from_source(&args[1]).expect("Failed to compile");
    let ast = prog.details().ast().unwrap();

    let mut dumper = AstDumper::with_source(&args[1]);
    dumper.dump_expr_node(ast, 0);
    let ranges = dumper.into_seen();

    let prog_str: &str = &args[1];
    for (start, end) in ranges.iter() {
        let sub_str: String = prog_str.chars().skip(*start).take(end - start).collect();
        let res = eval(&sub_str, &bindings).expect("Failed to eval");

        println!("{} => {}", sub_str, res);
    }
}

fn eval(source: &str, bindings: &serde_json::Value) -> CelResult<CelValue> {
    let mut ctx = CelContext::new();

    ctx.add_program_str("main", source)?;

    let mut bind = BindContext::new();
    bind.bind_params_from_json_obj(bindings.clone())?;

    Ok(ctx.exec("main", &bind)?)
}

struct AstDumper<'a> {
    source: &'a str,
    seen: HashSet<(usize, usize)>,
}

impl<'a> AstDumper<'a> {
    fn with_source(s: &'a str) -> Self {
        AstDumper {
            source: s,
            seen: HashSet::new(),
        }
    }

    fn into_seen(self) -> HashSet<(usize, usize)> {
        self.seen
    }

    fn dump_expr_node(&mut self, node: &AstNode<Expr>, depth: usize) {
        self.record_range(node, depth);
        match node.node() {
            Expr::Unary(nxt) => self.dump_or_node(nxt, depth + 1),
            Expr::Ternary {
                condition,
                true_clause,
                false_clause,
            } => {
                self.dump_or_node(condition, depth + 1);
                self.dump_or_node(true_clause, depth + 1);
                self.dump_expr_node(false_clause, depth + 1);
            }
        }
    }

    fn dump_or_node(&mut self, node: &AstNode<ConditionalOr>, depth: usize) {
        self.record_range(node, depth);
        match node.node() {
            ConditionalOr::Unary(nxt) => self.dump_and_node(nxt, depth + 1),
            ConditionalOr::Binary { lhs, rhs } => {
                self.dump_or_node(lhs, depth + 1);
                self.dump_and_node(rhs, depth + 1)
            }
        }
    }

    fn dump_and_node(&mut self, node: &AstNode<ConditionalAnd>, depth: usize) {
        self.record_range(node, depth);
        match node.node() {
            ConditionalAnd::Unary(nxt) => self.dump_relation(nxt, depth + 1),
            ConditionalAnd::Binary { lhs, rhs } => {
                self.dump_and_node(lhs, depth + 1);
                self.dump_relation(rhs, depth + 1);
            }
        }
    }

    fn dump_relation(&mut self, node: &AstNode<Relation>, depth: usize) {
        self.record_range(node, depth);
        match node.node() {
            Relation::Unary(nxt) => self.dump_addition(nxt, depth + 1),
            Relation::Binary { lhs, op: _op, rhs } => {
                self.dump_relation(lhs, depth + 1);
                self.dump_addition(rhs, depth + 1);
            }
        }
    }

    fn dump_addition(&mut self, node: &AstNode<Addition>, depth: usize) {
        self.record_range(node, depth);
        match node.node() {
            Addition::Unary(nxt) => self.dump_multiplication(nxt, depth + 1),
            Addition::Binary { lhs, op: _op, rhs } => {
                self.dump_addition(lhs, depth + 1);
                self.dump_multiplication(rhs, depth + 1);
            }
        }
    }

    fn dump_multiplication(&mut self, node: &AstNode<Multiplication>, depth: usize) {
        self.record_range(node, depth);
        match node.node() {
            Multiplication::Unary(nxt) => self.dump_uniary(nxt, depth + 1),
            Multiplication::Binary { lhs, op: _op, rhs } => {
                self.dump_multiplication(lhs, depth + 1);
                self.dump_uniary(rhs, depth + 1);
            }
        }
    }

    fn dump_uniary(&mut self, node: &AstNode<Unary>, depth: usize) {
        self.record_range(node, depth);
        match node.node() {
            Unary::Member(nxt) => self.dump_member(nxt, depth + 1),
            Unary::NegMember { negs, member } => {
                self.dump_neg(negs, depth + 1);
                self.dump_member(member, depth + 1);
            }
            Unary::NotMember { nots, member } => {
                self.dump_not(nots, depth + 1);
                self.dump_member(member, depth + 1);
            }
        }
    }

    fn dump_neg(&mut self, node: &AstNode<NegList>, depth: usize) {
        self.record_range(node, depth);
        match node.node() {
            NegList::List { tail } => self.dump_neg(tail, depth + 1),
            NegList::EmptyList => {}
        }
    }

    fn dump_not(&mut self, node: &AstNode<NotList>, depth: usize) {
        self.record_range(node, depth);
        match node.node() {
            NotList::List { tail } => self.dump_not(tail, depth + 1),
            NotList::EmptyList => {}
        }
    }

    fn dump_member(&mut self, node: &AstNode<Member>, depth: usize) {
        self.record_range(node, depth);
        self.dump_primary(&node.node().primary, depth + 1);
        for member in node.node().member.iter() {
            self.dump_member_prime(member, depth + 1);
        }
    }

    fn dump_primary(&mut self, node: &AstNode<Primary>, depth: usize) {
        self.record_range(node, depth);
        match node.node() {
            Primary::Type => {}
            Primary::Ident(_) => {}
            Primary::Parens(expr) => self.dump_expr_node(expr, depth + 1),
            Primary::ListConstruction(exprs) => self.dump_expr_list(exprs, depth + 1),
            Primary::ObjectInit(objinits) => self.dump_obj_inits(objinits, depth + 1),
            Primary::Literal(_) => {}
        }
    }

    fn dump_member_prime(&mut self, node: &AstNode<MemberPrime>, depth: usize) {
        self.record_range(node, depth);
        match node.node() {
            MemberPrime::MemberAccess { ident } => {
                self.dump_ident(&ident, depth + 1);
            }
            MemberPrime::Call { call } => self.dump_expr_list(call, depth + 1),
            MemberPrime::ArrayAccess { access } => self.dump_expr_node(access, depth + 1),
            MemberPrime::Empty => {}
        }
    }

    fn dump_ident(&mut self, node: &AstNode<Ident>, depth: usize) {
        self.record_range(node, depth);
    }

    fn dump_expr_list(&mut self, node: &AstNode<ExprList>, depth: usize) {
        self.record_range(node, depth);
        for expr in node.node().exprs.iter() {
            self.dump_expr_node(expr, depth + 1);
        }
    }

    fn dump_obj_inits(&mut self, node: &AstNode<ObjInits>, depth: usize) {
        self.record_range(node, depth);
        for init in node.node().inits.iter() {
            self.dump_obj_init(init, depth + 1);
        }
    }

    fn dump_obj_init(&mut self, node: &AstNode<ObjInit>, depth: usize) {
        self.record_range(node, depth);
        self.dump_expr_node(&node.node().key, depth + 1);
        self.dump_expr_node(&node.node().value, depth + 1);
    }

    fn record_range<T>(&mut self, ast: &AstNode<T>, _depth: usize) {
        let loc = (ast.start().col(), ast.end().col());

        self.seen.insert(loc);
    }
}
