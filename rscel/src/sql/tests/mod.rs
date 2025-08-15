use super::*;
use crate::{
    AddOp, Addition, AstNode, CelValue, ConditionalAnd, ConditionalOr, Expr, ExprList, Ident,
    LiteralsAndKeywords, Member, MemberPrime, MultOp, Multiplication, Primary, Relation,
    SourceLocation, SourceRange, Unary,
};

fn sr() -> SourceRange {
    SourceRange::new(SourceLocation::new(0, 0), SourceLocation::new(0, 0))
}

fn node<T>(v: T) -> AstNode<T> {
    AstNode::new(v, sr())
}

fn primary_int(v: i64) -> AstNode<Primary> {
    node(Primary::Literal(LiteralsAndKeywords::IntegerLit(v)))
}

fn primary_bool(v: bool) -> AstNode<Primary> {
    node(Primary::Literal(LiteralsAndKeywords::BooleanLit(v)))
}

fn primary_ident(name: &str) -> AstNode<Primary> {
    node(Primary::Ident(Ident(name.to_string())))
}

fn expr_from_member(m: AstNode<Member>) -> AstNode<Expr> {
    let unary = node(Unary::Member(m));
    let mul = node(Multiplication::Unary(unary));
    let add = node(Addition::Unary(mul));
    let rel = node(Relation::Unary(add));
    let cand = node(ConditionalAnd::Unary(rel));
    let cor = node(ConditionalOr::Unary(cand));
    node(Expr::Unary(Box::new(cor)))
}

fn expr_from_primary(p: AstNode<Primary>) -> AstNode<Expr> {
    expr_from_member(node(Member {
        primary: p,
        member: vec![],
    }))
}

fn or_from_primary(p: AstNode<Primary>) -> AstNode<ConditionalOr> {
    let member = node(Member {
        primary: p,
        member: vec![],
    });
    let unary = node(Unary::Member(member));
    let mul = node(Multiplication::Unary(unary));
    let add = node(Addition::Unary(mul));
    let rel = node(Relation::Unary(add));
    let cand = node(ConditionalAnd::Unary(rel));
    node(ConditionalOr::Unary(cand))
}

fn relation_from_primary(p: AstNode<Primary>) -> AstNode<Relation> {
    let member = node(Member {
        primary: p,
        member: vec![],
    });
    let unary = node(Unary::Member(member));
    let mul = node(Multiplication::Unary(unary));
    let add = node(Addition::Unary(mul));
    node(Relation::Unary(add))
}

#[test]
fn sql_arithmetic_from_ast() {
    let lit1 = primary_int(1);
    let lit2 = primary_int(2);
    let lit3 = primary_int(3);

    let mem1 = node(Member {
        primary: lit1,
        member: vec![],
    });
    let mem2 = node(Member {
        primary: lit2,
        member: vec![],
    });
    let mem3 = node(Member {
        primary: lit3,
        member: vec![],
    });

    let un1 = node(Unary::Member(mem1));
    let un2 = node(Unary::Member(mem2));
    let un3 = node(Unary::Member(mem3));

    let mul2 = node(Multiplication::Unary(un2));
    let mul_expr = node(Multiplication::Binary {
        lhs: Box::new(mul2),
        op: MultOp::Mult,
        rhs: un3,
    });

    let mul1 = node(Multiplication::Unary(un1));
    let add1 = node(Addition::Unary(mul1));
    let add_expr = node(Addition::Binary {
        lhs: Box::new(add1),
        op: AddOp::Add,
        rhs: mul_expr,
    });

    let rel = node(Relation::Unary(add_expr));
    let cand = node(ConditionalAnd::Unary(rel));
    let cor = node(ConditionalOr::Unary(cand));
    let expr = node(Expr::Unary(Box::new(cor)));

    let frag = SqlCompiler::compile(&expr).unwrap();
    assert_eq!(frag.sql, "($1 + ($2 * $3))");
    assert_eq!(
        frag.params,
        vec![
            CelValue::from_int(1),
            CelValue::from_int(2),
            CelValue::from_int(3),
        ],
    );
}

#[test]
fn sql_boolean_logic_from_ast() {
    let rel_true = relation_from_primary(primary_bool(true));
    let rel_false = relation_from_primary(primary_bool(false));
    let rel_true2 = relation_from_primary(primary_bool(true));

    let and_left = node(ConditionalAnd::Unary(rel_true));
    let and_expr = node(ConditionalAnd::Binary {
        lhs: Box::new(and_left),
        rhs: rel_false,
    });

    let lhs_or = node(ConditionalOr::Unary(and_expr));
    let rhs_or = node(ConditionalAnd::Unary(rel_true2));
    let or_expr = node(ConditionalOr::Binary {
        lhs: Box::new(lhs_or),
        rhs: rhs_or,
    });

    let expr = node(Expr::Unary(Box::new(or_expr)));
    let frag = SqlCompiler::compile(&expr).unwrap();
    assert_eq!(frag.sql, "(($1 AND $2) OR $3)");
    assert_eq!(
        frag.params,
        vec![
            CelValue::from_bool(true),
            CelValue::from_bool(false),
            CelValue::from_bool(true),
        ],
    );
}

#[test]
fn sql_ternary_from_ast() {
    let cond = or_from_primary(primary_bool(true));
    let true_clause = or_from_primary(primary_int(1));
    let false_clause = expr_from_primary(primary_int(2));

    let expr = node(Expr::Ternary {
        condition: Box::new(cond),
        true_clause: Box::new(true_clause),
        false_clause: Box::new(false_clause),
    });

    let frag = SqlCompiler::compile(&expr).unwrap();
    assert_eq!(frag.sql, "(CASE WHEN $1 THEN $2 ELSE $3 END)");
    assert_eq!(
        frag.params,
        vec![
            CelValue::from_bool(true),
            CelValue::from_int(1),
            CelValue::from_int(2),
        ],
    );
}

#[test]
fn sql_member_access_from_ast() {
    let primary = primary_ident("data");
    let member_access = node(MemberPrime::MemberAccess {
        ident: node(Ident("field".to_string())),
    });
    let member = node(Member {
        primary,
        member: vec![member_access],
    });
    let expr = expr_from_member(member);

    let frag = SqlCompiler::compile(&expr).unwrap();
    assert_eq!(frag.sql, "data ->> 'field'");
    assert!(frag.params.is_empty());
}

#[test]
fn sql_function_call_from_ast() {
    let arg_expr = expr_from_primary(primary_int(1));
    let call = node(ExprList {
        exprs: vec![arg_expr],
    });
    let member = node(Member {
        primary: primary_ident("abs"),
        member: vec![node(MemberPrime::Call { call })],
    });
    let expr = expr_from_member(member);

    let frag = SqlCompiler::compile(&expr).unwrap();
    assert_eq!(frag.sql, "ABS($1)");
    assert_eq!(frag.params, vec![CelValue::from_int(1)]);
}
