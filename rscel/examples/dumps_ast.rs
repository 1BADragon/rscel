use rscel::{
    Addition, AstNode, ConditionalAnd, ConditionalOr, Expr, ExprList, Ident, Member, MemberPrime,
    Multiplication, NegList, NotList, ObjInit, ObjInits, Primary, Relation, Unary,
};

fn main() {
    let args: Vec<_> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <prog>", args[0]);
        return;
    }

    let prog = rscel::Program::from_source(&args[1]).expect("Failed to compile");
    let ast = prog.details().ast().unwrap();

    AstDumper::with_source(&args[1]).dump_expr_node(ast, 0);
}

struct AstDumper<'a> {
    source: &'a str,
}

impl<'a> AstDumper<'a> {
    fn with_source(s: &'a str) -> Self {
        AstDumper { source: s }
    }

    fn dump_expr_node(&self, node: &AstNode<Expr>, depth: usize) {
        self.format_output(node, depth);
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
            Expr::Match { .. } => todo!(),
        }
    }

    fn dump_or_node(&self, node: &AstNode<ConditionalOr>, depth: usize) {
        self.format_output(node, depth);
        match node.node() {
            ConditionalOr::Unary(nxt) => self.dump_and_node(nxt, depth + 1),
            ConditionalOr::Binary { lhs, rhs } => {
                self.dump_or_node(lhs, depth + 1);
                self.dump_and_node(rhs, depth + 1)
            }
        }
    }

    fn dump_and_node(&self, node: &AstNode<ConditionalAnd>, depth: usize) {
        self.format_output(node, depth);
        match node.node() {
            ConditionalAnd::Unary(nxt) => self.dump_relation(nxt, depth + 1),
            ConditionalAnd::Binary { lhs, rhs } => {
                self.dump_and_node(lhs, depth + 1);
                self.dump_relation(rhs, depth + 1);
            }
        }
    }

    fn dump_relation(&self, node: &AstNode<Relation>, depth: usize) {
        self.format_output(node, depth);
        match node.node() {
            Relation::Unary(nxt) => self.dump_addition(nxt, depth + 1),
            Relation::Binary { lhs, op: _op, rhs } => {
                self.dump_relation(lhs, depth + 1);
                self.dump_addition(rhs, depth + 1);
            }
        }
    }

    fn dump_addition(&self, node: &AstNode<Addition>, depth: usize) {
        self.format_output(node, depth);
        match node.node() {
            Addition::Unary(nxt) => self.dump_multiplication(nxt, depth + 1),
            Addition::Binary { lhs, op: _op, rhs } => {
                self.dump_addition(lhs, depth + 1);
                self.dump_multiplication(rhs, depth + 1);
            }
        }
    }

    fn dump_multiplication(&self, node: &AstNode<Multiplication>, depth: usize) {
        self.format_output(node, depth);
        match node.node() {
            Multiplication::Unary(nxt) => self.dump_uniary(nxt, depth + 1),
            Multiplication::Binary { lhs, op: _op, rhs } => {
                self.dump_multiplication(lhs, depth + 1);
                self.dump_uniary(rhs, depth + 1);
            }
        }
    }

    fn dump_uniary(&self, node: &AstNode<Unary>, depth: usize) {
        self.format_output(node, depth);
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

    fn dump_neg(&self, node: &AstNode<NegList>, depth: usize) {
        self.format_output(node, depth);
        match node.node() {
            NegList::List { tail } => self.dump_neg(tail, depth + 1),
            NegList::EmptyList => {}
        }
    }

    fn dump_not(&self, node: &AstNode<NotList>, depth: usize) {
        self.format_output(node, depth);
        match node.node() {
            NotList::List { tail } => self.dump_not(tail, depth + 1),
            NotList::EmptyList => {}
        }
    }

    fn dump_member(&self, node: &AstNode<Member>, depth: usize) {
        self.format_output(node, depth);
        self.dump_primary(&node.node().primary, depth + 1);
        for member in node.node().member.iter() {
            self.dump_member_prime(member, depth + 1);
        }
    }

    fn dump_primary(&self, node: &AstNode<Primary>, depth: usize) {
        self.format_output(node, depth);
        match node.node() {
            Primary::Type => {}
            Primary::Ident(_) => {}
            Primary::Parens(expr) => self.dump_expr_node(expr, depth + 1),
            Primary::ListConstruction(exprs) => self.dump_expr_list(exprs, depth + 1),
            Primary::ObjectInit(objinits) => self.dump_obj_inits(objinits, depth + 1),
            Primary::Literal(_) => {}
        }
    }

    fn dump_member_prime(&self, node: &AstNode<MemberPrime>, depth: usize) {
        self.format_output(node, depth);
        match node.node() {
            MemberPrime::MemberAccess { ident } => {
                self.dump_ident(&ident, depth + 1);
            }
            MemberPrime::Call { call } => self.dump_expr_list(call, depth + 1),
            MemberPrime::ArrayAccess { access } => self.dump_expr_node(access, depth + 1),
            MemberPrime::Empty => {}
        }
    }

    fn dump_ident(&self, node: &AstNode<Ident>, depth: usize) {
        self.format_output(node, depth);
    }

    fn dump_expr_list(&self, node: &AstNode<ExprList>, depth: usize) {
        self.format_output(node, depth);
        for expr in node.node().exprs.iter() {
            self.dump_expr_node(expr, depth + 1);
        }
    }

    fn dump_obj_inits(&self, node: &AstNode<ObjInits>, depth: usize) {
        self.format_output(node, depth);
        for init in node.node().inits.iter() {
            self.dump_obj_init(init, depth + 1);
        }
    }

    fn dump_obj_init(&self, node: &AstNode<ObjInit>, depth: usize) {
        self.format_output(node, depth);
        self.dump_expr_node(&node.node().key, depth + 1);
        self.dump_expr_node(&node.node().value, depth + 1);
    }

    fn format_output<T>(&self, ast: &AstNode<T>, depth: usize) {
        let spacing: String = std::iter::repeat(' ').take(depth).collect();

        println!(
            "{} {} -- '{}'::{},{}",
            spacing,
            std::any::type_name::<T>().split("::").last().unwrap(),
            &self.source.to_string()[ast.start().col()..ast.end().col()],
            ast.start().col(),
            ast.end().col()
        );
    }
}
