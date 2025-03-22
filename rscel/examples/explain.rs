use rscel::{
    Addition, AstNode, BindContext, CelContext, CelResult, CelValue, ConditionalAnd, ConditionalOr,
    Expr, ExprList, Ident, Member, MemberPrime, Multiplication, NegList, NotList, ObjInit,
    ObjInits, Primary, Relation, SourceRange, Unary,
};

struct RangeNode {
    pub range: SourceRange,

    pub children: Vec<Box<RangeNode>>,
}

impl RangeNode {
    fn empty(range: SourceRange) -> Self {
        RangeNode {
            range,
            children: Vec::new(),
        }
    }

    fn with_children(range: SourceRange, children: impl Iterator<Item = Box<RangeNode>>) -> Self {
        RangeNode {
            range,
            children: children.collect(),
        }
    }

    fn range(&self) -> SourceRange {
        self.range
    }

    fn into_children(self) -> Vec<Box<RangeNode>> {
        self.children
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <prog> [bindings]", args[0]);
        return;
    }

    let bindings: serde_json::Value = if args.len() == 3 {
        serde_json::from_str(&args[2]).expect("Failed to parse bindings")
    } else {
        serde_json::from_str::<serde_json::Value>("{}").unwrap()
    };

    let prog = rscel::Program::from_source(&args[1]).expect("Failed to compile");
    let ast = prog.details().ast().unwrap();

    let dumper = AstDumper::new();
    let tree = dumper.dump_expr_node(ast);

    let mut queue = Vec::new();
    queue.push((tree, 0));

    while !queue.is_empty() {
        let (curr_node, depth) = queue.pop().expect("wut");

        // for now only handle one line
        let range = curr_node.range();

        let start = range.start().col();
        let end = range.end().col();

        let src = &args[1][start..end];

        print!("{}{} => ", " ".to_owned().repeat(depth), src);
        match eval(src, &bindings) {
            Ok(val) => println!("{}", val),
            Err(err) => println!("err: {}", err),
        }

        for node in curr_node.into_children().into_iter().rev() {
            if node.range() != range {
                queue.push((node, depth + 1));
            }
        }
    }
}

fn eval(source: &str, bindings: &serde_json::Value) -> CelResult<CelValue> {
    let mut ctx = CelContext::new();

    ctx.add_program_str("main", source)?;

    let mut bind = BindContext::new();
    bind.bind_params_from_json_obj(bindings.clone())?;

    Ok(ctx.exec("main", &bind)?)
}

struct AstDumper;

impl AstDumper {
    fn new() -> Self {
        AstDumper {}
    }

    fn dump_expr_node(&self, node: &AstNode<Expr>) -> Box<RangeNode> {
        match node.node() {
            Expr::Unary(nxt) => self.dump_or_node(nxt),
            Expr::Ternary {
                condition,
                true_clause,
                false_clause,
            } => Box::new(RangeNode::with_children(
                node.range(),
                [
                    self.dump_or_node(condition),
                    self.dump_or_node(true_clause),
                    self.dump_expr_node(false_clause),
                ]
                .into_iter(),
            )),
            Expr::Match { .. } => todo!(),
        }
    }

    fn dump_or_node(&self, node: &AstNode<ConditionalOr>) -> Box<RangeNode> {
        match node.node() {
            ConditionalOr::Unary(nxt) => self.dump_and_node(nxt),
            ConditionalOr::Binary { lhs, rhs } => Box::new(RangeNode::with_children(
                node.range(),
                [self.dump_or_node(lhs), self.dump_and_node(rhs)].into_iter(),
            )),
        }
    }

    fn dump_and_node(&self, node: &AstNode<ConditionalAnd>) -> Box<RangeNode> {
        match node.node() {
            ConditionalAnd::Unary(nxt) => self.dump_relation(nxt),
            ConditionalAnd::Binary { lhs, rhs } => Box::new(RangeNode::with_children(
                node.range(),
                [self.dump_and_node(lhs), self.dump_relation(rhs)].into_iter(),
            )),
        }
    }

    fn dump_relation(&self, node: &AstNode<Relation>) -> Box<RangeNode> {
        match node.node() {
            Relation::Unary(nxt) => self.dump_addition(nxt),
            Relation::Binary { lhs, op: _op, rhs } => Box::new(RangeNode::with_children(
                node.range(),
                [self.dump_relation(lhs), self.dump_addition(rhs)].into_iter(),
            )),
        }
    }

    fn dump_addition(&self, node: &AstNode<Addition>) -> Box<RangeNode> {
        match node.node() {
            Addition::Unary(nxt) => self.dump_multiplication(nxt),
            Addition::Binary { lhs, op: _op, rhs } => Box::new(RangeNode::with_children(
                node.range(),
                [self.dump_addition(lhs), self.dump_multiplication(rhs)].into_iter(),
            )),
        }
    }

    fn dump_multiplication(&self, node: &AstNode<Multiplication>) -> Box<RangeNode> {
        match node.node() {
            Multiplication::Unary(nxt) => self.dump_uniary(nxt),
            Multiplication::Binary { lhs, op: _op, rhs } => Box::new(RangeNode::with_children(
                node.range(),
                [self.dump_multiplication(lhs), self.dump_uniary(rhs)].into_iter(),
            )),
        }
    }

    fn dump_uniary(&self, node: &AstNode<Unary>) -> Box<RangeNode> {
        match node.node() {
            Unary::Member(nxt) => self.dump_member(nxt),
            Unary::NegMember { negs, member } => {
                self.dump_neg(negs);
                let mem = self.dump_member(member);
                Box::new(RangeNode::with_children(node.range(), [mem].into_iter()))
            }
            Unary::NotMember { nots, member } => {
                self.dump_not(nots);
                let mem = self.dump_member(member);
                Box::new(RangeNode::with_children(node.range(), [mem].into_iter()))
            }
        }
    }

    fn dump_neg(&self, node: &AstNode<NegList>) {
        match node.node() {
            NegList::List { tail } => self.dump_neg(tail),
            NegList::EmptyList => {}
        }
    }

    fn dump_not(&self, node: &AstNode<NotList>) {
        match node.node() {
            NotList::List { tail } => self.dump_not(tail),
            NotList::EmptyList => {}
        }
    }

    fn dump_member(&self, node: &AstNode<Member>) -> Box<RangeNode> {
        let mut children = vec![self.dump_primary(&node.node().primary)];
        for member in node.node().member.iter() {
            children.push(self.dump_member_prime(member));
        }

        Box::new(RangeNode::with_children(node.range(), children.into_iter()))
    }

    fn dump_primary(&self, node: &AstNode<Primary>) -> Box<RangeNode> {
        match node.node() {
            Primary::Type => Box::new(RangeNode::empty(node.range())),
            Primary::Ident(_) => Box::new(RangeNode::empty(node.range())),
            Primary::Parens(expr) => Box::new(RangeNode::with_children(
                node.range(),
                [self.dump_expr_node(expr)].into_iter(),
            )),
            Primary::ListConstruction(exprs) => self.dump_expr_list(exprs),
            Primary::ObjectInit(objinits) => self.dump_obj_inits(objinits),
            Primary::Literal(_) => Box::new(RangeNode::empty(node.range())),
        }
    }

    fn dump_member_prime(&self, node: &AstNode<MemberPrime>) -> Box<RangeNode> {
        match node.node() {
            MemberPrime::MemberAccess { ident } => self.dump_ident(&ident),
            MemberPrime::Call { call } => self.dump_expr_list(call),
            MemberPrime::ArrayAccess { access } => self.dump_expr_node(access),
            MemberPrime::Empty => Box::new(RangeNode::empty(node.range())),
        }
    }

    fn dump_ident(&self, node: &AstNode<Ident>) -> Box<RangeNode> {
        Box::new(RangeNode::empty(node.range()))
    }

    fn dump_expr_list(&self, node: &AstNode<ExprList>) -> Box<RangeNode> {
        Box::new(RangeNode::with_children(
            node.range(),
            node.node()
                .exprs
                .clone()
                .into_iter()
                .map(|x| self.dump_expr_node(&x)),
        ))
    }

    fn dump_obj_inits(&self, node: &AstNode<ObjInits>) -> Box<RangeNode> {
        Box::new(RangeNode::with_children(
            node.range(),
            node.node()
                .inits
                .clone()
                .into_iter()
                .map(|x| self.dump_obj_init(&x)),
        ))
    }

    fn dump_obj_init(&self, node: &AstNode<ObjInit>) -> Box<RangeNode> {
        self.dump_expr_node(&node.node().key);
        self.dump_expr_node(&node.node().value);
        Box::new(RangeNode::empty(node.range()))
    }
}
