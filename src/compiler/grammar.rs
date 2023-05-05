use super::types::{parser::ParserCallback, Lexicons};

const EXPR: &str = "Expr";
const CONDITIONAL_OR: &str = "ConditionalOr";
const CONDITIONAL_AND: &str = "ConditionalAnd";
const RELATION: &str = "Relation";
const RELOP: &str = "Relop";
const ADDITION: &str = "Addition";
const MULTIPLICATION: &str = "Multiplication";
const UNARY: &str = "Unary";
const MEMBER: &str = "Member";
const PRIMARY: &str = "Primary";
const EXPR_LIST: &str = "ExprList";
const FIELD_INITS: &str = "FieldInits";
const MAP_INITS: &str = "MapInits";
const LITERAL: &str = "Literal";

#[derive(Debug)]
enum ParserRuleComponent {
    Token(Lexicons),
    ParseRule(&'static str),
    RepeatedParseRule(&'static [ParserRuleComponent]),
    OptionalParseRule(&'static [ParserRuleComponent]),
    OneOfParseRule(&'static [ParserRuleComponent]),
}

#[derive(Debug)]
struct ParseRule {
    rule: &'static [ParserRuleComponent],
    callback: Option<ParserCallback>,
}

#[derive(Debug)]
struct ParseRuleGroup {
    name: &'static str,

    rules: &'static [ParseRule],
}

const PARSE_RULES: &'static [ParseRuleGroup] = &[
    ParseRuleGroup {
        name: EXPR,
        rules: &[ParseRule {
            rule: &[
                ParserRuleComponent::ParseRule(CONDITIONAL_OR),
                ParserRuleComponent::OptionalParseRule(&[
                    ParserRuleComponent::Token(Lexicons::Question),
                    ParserRuleComponent::ParseRule(CONDITIONAL_OR),
                    ParserRuleComponent::Token(Lexicons::Colon),
                    ParserRuleComponent::ParseRule(EXPR),
                ]),
            ],
            callback: None,
        }],
    },
    ParseRuleGroup {
        name: CONDITIONAL_OR,
        rules: &[ParseRule {
            rule: &[
                ParserRuleComponent::OptionalParseRule(&[
                    ParserRuleComponent::ParseRule(CONDITIONAL_OR),
                    ParserRuleComponent::Token(Lexicons::OrOp),
                ]),
                ParserRuleComponent::ParseRule(CONDITIONAL_AND),
            ],
            callback: None,
        }],
    },
    ParseRuleGroup {
        name: CONDITIONAL_AND,
        rules: &[ParseRule {
            rule: &[
                ParserRuleComponent::OptionalParseRule(&[
                    ParserRuleComponent::ParseRule(CONDITIONAL_AND),
                    ParserRuleComponent::Token(Lexicons::AndOp),
                ]),
                ParserRuleComponent::ParseRule(RELATION),
            ],
            callback: None,
        }],
    },
    ParseRuleGroup {
        name: RELATION,
        rules: &[ParseRule {
            rule: &[
                ParserRuleComponent::OptionalParseRule(&[
                    ParserRuleComponent::ParseRule(RELATION),
                    ParserRuleComponent::ParseRule(RELOP),
                ]),
                ParserRuleComponent::ParseRule(ADDITION),
            ],
            callback: None,
        }],
    },
    ParseRuleGroup {
        name: RELOP,
        rules: &[
            ParseRule {
                rule: &[ParserRuleComponent::Token(Lexicons::LtOp)],
                callback: None,
            },
            ParseRule {
                rule: &[ParserRuleComponent::Token(Lexicons::LeOp)],
                callback: None,
            },
            ParseRule {
                rule: &[ParserRuleComponent::Token(Lexicons::GeOp)],
                callback: None,
            },
            ParseRule {
                rule: &[ParserRuleComponent::Token(Lexicons::GtOp)],
                callback: None,
            },
            ParseRule {
                rule: &[ParserRuleComponent::Token(Lexicons::EqOp)],
                callback: None,
            },
            ParseRule {
                rule: &[ParserRuleComponent::Token(Lexicons::NeOp)],
                callback: None,
            },
            ParseRule {
                rule: &[ParserRuleComponent::Token(Lexicons::InOp)],
                callback: None,
            },
        ],
    },
    ParseRuleGroup {
        name: ADDITION,
        rules: &[ParseRule {
            rule: &[
                ParserRuleComponent::OptionalParseRule(&[
                    ParserRuleComponent::ParseRule(ADDITION),
                    ParserRuleComponent::OneOfParseRule(&[
                        ParserRuleComponent::Token(Lexicons::AddOp),
                        ParserRuleComponent::Token(Lexicons::SubOp),
                    ]),
                ]),
                ParserRuleComponent::ParseRule(MULTIPLICATION),
            ],
            callback: None,
        }],
    },
    ParseRuleGroup {
        name: MULTIPLICATION,
        rules: &[ParseRule {
            rule: &[
                ParserRuleComponent::OptionalParseRule(&[
                    ParserRuleComponent::ParseRule(MULTIPLICATION),
                    ParserRuleComponent::OneOfParseRule(&[
                        ParserRuleComponent::Token(Lexicons::MulOp),
                        ParserRuleComponent::Token(Lexicons::DivOp),
                        ParserRuleComponent::Token(Lexicons::ModOp),
                    ]),
                ]),
                ParserRuleComponent::ParseRule(UNARY),
            ],
            callback: None,
        }],
    },
    ParseRuleGroup {
        name: UNARY,
        rules: &[
            ParseRule {
                rule: &[ParserRuleComponent::ParseRule(MEMBER)],
                callback: None,
            },
            ParseRule {
                rule: &[
                    ParserRuleComponent::Token(Lexicons::BangOp),
                    ParserRuleComponent::RepeatedParseRule(&[ParserRuleComponent::Token(
                        Lexicons::BangOp,
                    )]),
                    ParserRuleComponent::ParseRule(MEMBER),
                ],
                callback: None,
            },
            ParseRule {
                rule: &[
                    ParserRuleComponent::Token(Lexicons::SubOp),
                    ParserRuleComponent::RepeatedParseRule(&[ParserRuleComponent::Token(
                        Lexicons::SubOp,
                    )]),
                    ParserRuleComponent::ParseRule(MEMBER),
                ],
                callback: None,
            },
        ],
    },
    ParseRuleGroup {
        name: MEMBER,
        rules: &[
            ParseRule {
                rule: &[ParserRuleComponent::ParseRule(PRIMARY)],
                callback: None,
            },
            ParseRule {
                rule: &[
                    ParserRuleComponent::ParseRule(MEMBER),
                    ParserRuleComponent::Token(Lexicons::Period),
                    ParserRuleComponent::Token(Lexicons::Ident),
                    ParserRuleComponent::OptionalParseRule(&[
                        ParserRuleComponent::Token(Lexicons::LParen),
                        ParserRuleComponent::OptionalParseRule(&[ParserRuleComponent::ParseRule(
                            EXPR_LIST,
                        )]),
                        ParserRuleComponent::Token(Lexicons::RParen),
                    ]),
                ],
                callback: None,
            },
            ParseRule {
                rule: &[
                    ParserRuleComponent::ParseRule(MEMBER),
                    ParserRuleComponent::Token(Lexicons::LParen),
                    ParserRuleComponent::ParseRule(EXPR),
                    ParserRuleComponent::Token(Lexicons::RParen),
                ],
                callback: None,
            },
        ],
    },
    ParseRuleGroup {
        name: PRIMARY,
        rules: &[
            ParseRule {
                rule: &[
                    ParserRuleComponent::OptionalParseRule(&[ParserRuleComponent::Token(
                        Lexicons::Period,
                    )]),
                    ParserRuleComponent::Token(Lexicons::Ident),
                    ParserRuleComponent::OptionalParseRule(&[
                        ParserRuleComponent::Token(Lexicons::LParen),
                        ParserRuleComponent::OptionalParseRule(&[ParserRuleComponent::ParseRule(
                            EXPR_LIST,
                        )]),
                        ParserRuleComponent::Token(Lexicons::RParen),
                    ]),
                ],
                callback: None,
            },
            ParseRule {
                rule: &[
                    ParserRuleComponent::Token(Lexicons::LParen),
                    ParserRuleComponent::ParseRule(EXPR),
                    ParserRuleComponent::Token(Lexicons::RParen),
                ],
                callback: None,
            },
            ParseRule {
                rule: &[
                    ParserRuleComponent::Token(Lexicons::LBracket),
                    ParserRuleComponent::OptionalParseRule(&[ParserRuleComponent::ParseRule(
                        EXPR_LIST,
                    )]),
                    ParserRuleComponent::OptionalParseRule(&[ParserRuleComponent::Token(
                        Lexicons::Comma,
                    )]),
                    ParserRuleComponent::Token(Lexicons::RBracket),
                ],
                callback: None,
            },
            ParseRule {
                rule: &[
                    ParserRuleComponent::Token(Lexicons::LBrace),
                    ParserRuleComponent::OptionalParseRule(&[ParserRuleComponent::ParseRule(
                        MAP_INITS,
                    )]),
                    ParserRuleComponent::OptionalParseRule(&[ParserRuleComponent::Token(
                        Lexicons::Comma,
                    )]),
                    ParserRuleComponent::Token(Lexicons::RBrace),
                ],
                callback: None,
            },
            ParseRule {
                rule: &[
                    ParserRuleComponent::OptionalParseRule(&[ParserRuleComponent::Token(
                        Lexicons::Period,
                    )]),
                    ParserRuleComponent::Token(Lexicons::Ident),
                    ParserRuleComponent::RepeatedParseRule(&[
                        ParserRuleComponent::Token(Lexicons::Period),
                        ParserRuleComponent::Token(Lexicons::Ident),
                    ]),
                    ParserRuleComponent::Token(Lexicons::LBrace),
                    ParserRuleComponent::OptionalParseRule(&[ParserRuleComponent::ParseRule(
                        FIELD_INITS,
                    )]),
                    ParserRuleComponent::OptionalParseRule(&[ParserRuleComponent::Token(
                        Lexicons::Comma,
                    )]),
                    ParserRuleComponent::Token(Lexicons::RBrace),
                ],
                callback: None,
            },
            ParseRule {
                rule: &[ParserRuleComponent::ParseRule(LITERAL)],
                callback: None,
            },
        ],
    },
    ParseRuleGroup {
        name: EXPR_LIST,
        rules: &[ParseRule {
            rule: &[
                ParserRuleComponent::ParseRule(EXPR),
                ParserRuleComponent::RepeatedParseRule(&[
                    ParserRuleComponent::Token(Lexicons::Comma),
                    ParserRuleComponent::ParseRule(EXPR),
                ]),
            ],
            callback: None,
        }],
    },
    ParseRuleGroup {
        name: FIELD_INITS,
        rules: &[ParseRule {
            rule: &[
                ParserRuleComponent::Token(Lexicons::Ident),
                ParserRuleComponent::Token(Lexicons::Colon),
                ParserRuleComponent::ParseRule(EXPR),
                ParserRuleComponent::RepeatedParseRule(&[
                    ParserRuleComponent::Token(Lexicons::Comma),
                    ParserRuleComponent::Token(Lexicons::Ident),
                    ParserRuleComponent::Token(Lexicons::Colon),
                    ParserRuleComponent::ParseRule(EXPR),
                ]),
            ],
            callback: None,
        }],
    },
    ParseRuleGroup {
        name: MAP_INITS,
        rules: &[ParseRule {
            rule: &[
                ParserRuleComponent::ParseRule(EXPR),
                ParserRuleComponent::Token(Lexicons::Colon),
                ParserRuleComponent::ParseRule(EXPR),
                ParserRuleComponent::RepeatedParseRule(&[
                    ParserRuleComponent::Token(Lexicons::Comma),
                    ParserRuleComponent::ParseRule(EXPR),
                    ParserRuleComponent::Token(Lexicons::Colon),
                    ParserRuleComponent::ParseRule(EXPR),
                ]),
            ],
            callback: None,
        }],
    },
    ParseRuleGroup {
        name: LITERAL,
        rules: &[
            ParseRule {
                rule: &[ParserRuleComponent::Token(Lexicons::IntLit)],
                callback: None,
            },
            ParseRule {
                rule: &[ParserRuleComponent::Token(Lexicons::UintLit)],
                callback: None,
            },
            ParseRule {
                rule: &[ParserRuleComponent::Token(Lexicons::FloatLit)],
                callback: None,
            },
            ParseRule {
                rule: &[ParserRuleComponent::Token(Lexicons::StringLit)],
                callback: None,
            },
            ParseRule {
                rule: &[ParserRuleComponent::Token(Lexicons::BytesLit)],
                callback: None,
            },
            ParseRule {
                rule: &[ParserRuleComponent::Token(Lexicons::BoolLit)],
                callback: None,
            },
            ParseRule {
                rule: &[ParserRuleComponent::Token(Lexicons::NullLit)],
                callback: None,
            },
        ],
    },
];
