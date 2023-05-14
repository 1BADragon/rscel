use crate::parser::{
    Addition, ConditionalAnd, ConditionalOr, Expr, MemberPrime, Multiplication, Primary, Relation,
    Unary,
};

pub fn extract_ident(expr: &Expr) -> Option<String> {
    if expr.ternary.as_prefix().is_some() {
        return None;
    }

    let member = if let ConditionalOr::Rhs(ConditionalAnd::Rhs(Relation::Rhs(Addition::Rhs(
        Multiplication::Rhs(Unary::Member(member)),
    )))) = &expr.cond_or
    {
        member
    } else {
        return None;
    };

    let ident = if let Primary::Ident(ident) = &member.primary {
        ident.to_string()
    } else {
        return None;
    };

    if let MemberPrime::Empty(_) = &member.member {
    } else {
        return None;
    }

    Some(ident)
}
