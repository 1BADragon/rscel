use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    punctuated::Punctuated, token, AngleBracketedGenericArguments, Arm, BinOp, Block, Expr,
    ExprBinary, ExprCall, ExprClosure, ExprIf, ExprMatch, ExprReturn, ExprTuple, FnArg,
    GenericArgument, Generics, Ident, Item, ItemFn, ItemMod, Lit, LitStr, Local, LocalInit, Pat,
    PatIdent, PatSlice, PatType, PatWild, PathSegment, Signature, Stmt, Type, TypePath,
};

use super::dispatch_func::DispatchFunc;

enum DispatchItem {
    Func(DispatchFunc),
    Other(Item),
}

impl DispatchItem {
    pub fn into_dispatch_item(self) -> Item {
        match self {
            DispatchItem::Func(dispatch_func) => Item::Fn(dispatch_func.into_dispatch_fn()),
            DispatchItem::Other(item) => item,
        }
    }
}

pub struct DispatchMod {
    parsed: ItemMod,
    items: Vec<DispatchItem>,
}

impl DispatchMod {
    pub fn from_mod(m: ItemMod) -> Self {
        DispatchMod {
            items: match m.content {
                Some((_, ref items)) => items
                    .iter()
                    .map(|item| match item {
                        Item::Fn(f) => DispatchItem::Func(DispatchFunc::from_item(f.clone())),
                        other => DispatchItem::Other(other.clone()),
                    })
                    .collect(),
                None => Vec::new(),
            },
            parsed: m,
        }
    }

    pub fn into_token_stream(self) -> TokenStream {
        let mut mod_tokens = self.parsed;

        let dispatch = DispatchMod::build_dispatch_func_item(
            self.items
                .iter()
                .filter(|i| matches!(*i, DispatchItem::Func(_)))
                .map(|i| match i {
                    DispatchItem::Func(f) => f,
                    _ => unreachable!(),
                }),
            &mod_tokens.ident.to_string(),
        );

        let mut dispatch_items: Vec<Item> = self
            .items
            .into_iter()
            .map(|i| i.into_dispatch_item())
            .collect();

        dispatch_items.push(Item::Fn(dispatch));

        let brace = if let Some((old, _items)) = mod_tokens.content {
            old
        } else {
            panic!("No content in dispatch")
        };

        mod_tokens.content = Some((brace, dispatch_items));

        return mod_tokens.to_token_stream().into();
    }

    fn build_dispatch_func_item<'a>(
        items: impl Iterator<Item = &'a DispatchFunc>,
        mod_name: &str,
    ) -> ItemFn {
        let item_vec: Vec<_> = items.collect();

        let mut max_args = 0;
        for item in item_vec.iter() {
            if item.args.len() > max_args {
                max_args = item.args.len();
            }
        }

        ItemFn {
            attrs: Vec::new(),
            vis: syn::Visibility::Public(token::Pub::default()),
            sig: Signature {
                constness: None,
                asyncness: None,
                unsafety: None,
                abi: None,
                fn_token: token::Fn::default(),
                ident: Ident::new("dispatch", Span::call_site()),
                generics: Generics {
                    lt_token: None,
                    params: Punctuated::new(),
                    gt_token: None,
                    where_clause: None,
                },
                paren_token: token::Paren::default(),
                inputs: [
                    FnArg::Typed(PatType {
                        attrs: Vec::new(),
                        pat: Box::new(Pat::Ident(PatIdent {
                            attrs: Vec::new(),
                            by_ref: None,
                            mutability: None,
                            ident: Ident::new("this", Span::call_site()),
                            subpat: None,
                        })),
                        colon_token: token::Colon::default(),
                        ty: DispatchMod::cel_value_type(),
                    }),
                    FnArg::Typed(PatType {
                        attrs: Vec::new(),
                        pat: Box::new(Pat::Ident(PatIdent {
                            attrs: Vec::new(),
                            by_ref: None,
                            mutability: Some(token::Mut::default()),
                            ident: Ident::new("args", Span::call_site()),
                            subpat: None,
                        })),
                        colon_token: token::Colon::default(),
                        ty: Box::new(Type::Path(TypePath {
                            qself: None,
                            path: syn::Path {
                                leading_colon: None,
                                segments: [PathSegment {
                                    ident: Ident::new("Vec", Span::call_site()),
                                    arguments: syn::PathArguments::AngleBracketed(
                                        AngleBracketedGenericArguments {
                                            colon2_token: None,
                                            lt_token: token::Lt::default(),
                                            args: [GenericArgument::Type(syn::Type::Path(
                                                TypePath {
                                                    qself: None,
                                                    path: syn::Path {
                                                        leading_colon: None,
                                                        segments: [Into::<PathSegment>::into(
                                                            Ident::new(
                                                                "CelValue",
                                                                Span::call_site(),
                                                            ),
                                                        )]
                                                        .into_iter()
                                                        .collect(),
                                                    },
                                                },
                                            ))]
                                            .into_iter()
                                            .collect(),
                                            gt_token: token::Gt::default(),
                                        },
                                    ),
                                }]
                                .into_iter()
                                .collect(),
                            },
                        })),
                    }),
                ]
                .into_iter()
                .collect(),
                variadic: None,
                output: syn::ReturnType::Type(
                    token::RArrow::default(),
                    DispatchMod::cel_value_type(),
                ),
            },
            block: Box::new(Block {
                brace_token: token::Brace::default(),
                stmts: vec![
                    // if args.len() < max_args
                    Stmt::Expr(
                        syn::Expr::If(ExprIf {
                            attrs: Vec::new(),
                            if_token: token::If::default(),
                            cond: Box::new(syn::Expr::Binary(ExprBinary {
                                attrs: Vec::new(),
                                left: Box::new(syn::Expr::MethodCall(syn::ExprMethodCall {
                                    attrs: Vec::new(),
                                    receiver: Box::new(syn::Expr::Path(syn::ExprPath {
                                        attrs: Vec::new(),
                                        qself: None,
                                        path: syn::Path {
                                            leading_colon: None,
                                            segments: [PathSegment {
                                                ident: Ident::new("args", Span::call_site()),
                                                arguments: syn::PathArguments::None,
                                            }]
                                            .into_iter()
                                            .collect(),
                                        },
                                    })),
                                    dot_token: token::Dot::default(),
                                    method: Ident::new("len", Span::call_site()),
                                    turbofish: None,
                                    paren_token: token::Paren::default(),
                                    args: Punctuated::new(),
                                })),
                                op: BinOp::Lt(token::Lt::default()),
                                right: Box::new(syn::Expr::Lit(syn::ExprLit {
                                    attrs: Vec::new(),
                                    lit: syn::Lit::Int(
                                        proc_macro2::Literal::usize_suffixed(max_args).into(),
                                    ),
                                })),
                            })),
                            then_branch: Block {
                                brace_token: token::Brace::default(),
                                stmts: vec![Stmt::Expr(
                                syn::Expr::MethodCall(syn::ExprMethodCall {
                                    attrs: Vec::new(),
                                    receiver: Box::new(syn::Expr::Path(syn::ExprPath {
                                        attrs: Vec::new(),
                                        qself: None,
                                        path: Ident::new("args", Span::call_site()).into(),
                                    })),
                                    dot_token: token::Dot::default(),
                                    method: Ident::new("extend", Span::call_site()).into(),
                                    turbofish: None,
                                    paren_token: token::Paren::default(),
                                    args: [syn::Expr::MethodCall(syn::ExprMethodCall {
                                        attrs: Vec::new(),
                                        receiver: Box::new(syn::Expr::Range(syn::ExprRange {
                                            attrs: Vec::new(),
                                            start: Some(Box::new(syn::Expr::Lit(syn::ExprLit {
                                                attrs: Vec::new(),
                                                lit: syn::Lit::Int(
                                                    proc_macro2::Literal::usize_suffixed(0).into(),
                                                ),
                                            }))),
                                            limits: syn::RangeLimits::HalfOpen(
                                                token::DotDot::default(),
                                            ),
                                            end: Some(Box::new(syn::Expr::Binary(ExprBinary {
                                                attrs: Vec::new(),
                                                left: Box::new(syn::Expr::Lit(syn::ExprLit {
                                                    attrs: Vec::new(),
                                                    lit: syn::Lit::Int(
                                                        proc_macro2::Literal::usize_suffixed(
                                                            max_args,
                                                        )
                                                        .into(),
                                                    ),
                                                })),
                                                op: BinOp::Sub(token::Minus::default()),
                                                right: Box::new(syn::Expr::MethodCall(
                                                    syn::ExprMethodCall {
                                                        attrs: Vec::new(),
                                                        receiver: Box::new(syn::Expr::Path(
                                                            syn::ExprPath {
                                                                attrs: Vec::new(),
                                                                qself: None,
                                                                path: Ident::new(
                                                                    "args",
                                                                    Span::call_site(),
                                                                )
                                                                .into(),
                                                            },
                                                        )),
                                                        dot_token: token::Dot::default(),
                                                        method: Ident::new(
                                                            "len",
                                                            Span::call_site(),
                                                        ),
                                                        turbofish: None,
                                                        paren_token: token::Paren::default(),
                                                        args: Punctuated::new(),
                                                    },
                                                )),
                                            }))),
                                        })),
                                        dot_token: token::Dot::default(),
                                        method: Ident::new("map", Span::call_site()),
                                        turbofish: None,
                                        paren_token: token::Paren::default(),
                                        args: [syn::Expr::Closure(ExprClosure {
                                            attrs: Vec::new(),
                                            lifetimes: None,
                                            constness: None,
                                            movability: None,
                                            asyncness: None,
                                            capture: None,
                                            or1_token: token::Or::default(),
                                            inputs: [syn::Pat::Ident(PatIdent {
                                                attrs: Vec::new(),
                                                by_ref: None,
                                                mutability: None,
                                                ident: Ident::new("_", Span::call_site()),
                                                subpat: None,
                                            })]
                                            .into_iter()
                                            .collect(),
                                            or2_token: token::Or::default(),
                                            output: syn::ReturnType::Default,
                                            body: Box::new(syn::Expr::Path(syn::ExprPath {
                                                attrs: Vec::new(),
                                                qself: None,
                                                path: syn::Path {
                                                    leading_colon: None,
                                                    segments: [
                                                        PathSegment {
                                                            arguments: syn::PathArguments::None,
                                                            ident: Ident::new(
                                                                "CelValue",
                                                                Span::call_site(),
                                                            ),
                                                        },
                                                        PathSegment {
                                                            arguments: syn::PathArguments::None,
                                                            ident: Ident::new(
                                                                "Null",
                                                                Span::call_site(),
                                                            ),
                                                        },
                                                    ]
                                                    .into_iter()
                                                    .collect(),
                                                },
                                            })),
                                        })]
                                        .into_iter()
                                        .collect(),
                                    })]
                                    .into_iter()
                                    .collect(),
                                }),
                                Some(token::Semi::default()),
                            )],
                            },
                            else_branch: None,
                        }),
                        None,
                    ),
                    // if args.len() > max_args
                    Stmt::Expr(
                        syn::Expr::If(ExprIf {
                            attrs: Vec::new(),
                            if_token: token::If::default(),
                            cond: Box::new(syn::Expr::Binary(ExprBinary {
                                attrs: Vec::new(),
                                left: Box::new(syn::Expr::MethodCall(syn::ExprMethodCall {
                                    attrs: Vec::new(),
                                    receiver: Box::new(syn::Expr::Path(syn::ExprPath {
                                        attrs: Vec::new(),
                                        qself: None,
                                        path: syn::Path {
                                            leading_colon: None,
                                            segments: [PathSegment {
                                                ident: Ident::new("args", Span::call_site()),
                                                arguments: syn::PathArguments::None,
                                            }]
                                            .into_iter()
                                            .collect(),
                                        },
                                    })),
                                    dot_token: token::Dot::default(),
                                    method: Ident::new("len", Span::call_site()),
                                    turbofish: None,
                                    paren_token: token::Paren::default(),
                                    args: Punctuated::new(),
                                })),
                                op: BinOp::Gt(token::Gt::default()),
                                right: Box::new(syn::Expr::Lit(syn::ExprLit {
                                    attrs: Vec::new(),
                                    lit: syn::Lit::Int(
                                        proc_macro2::Literal::usize_suffixed(max_args).into(),
                                    ),
                                })),
                            })),
                            then_branch: Block {
                                brace_token: token::Brace::default(),
                                stmts: vec![Stmt::Expr(
                                    syn::Expr::Return(ExprReturn {
                                        attrs: Vec::new(),
                                        return_token: token::Return::default(),
                                        expr: Some(Box::new(syn::Expr::Call(syn::ExprCall {
                                            attrs: Vec::new(),
                                            func: Box::new(syn::Expr::Path(syn::ExprPath {
                                                attrs: Vec::new(),
                                                qself: None,
                                                path: syn::Path {
                                                    leading_colon: None,
                                                    segments: [
                                                        PathSegment {
                                                            ident: Ident::new(
                                                                "CelValue",
                                                                Span::call_site(),
                                                            ),
                                                            arguments: syn::PathArguments::None,
                                                        },
                                                        PathSegment {
                                                            ident: Ident::new(
                                                                "argument_error",
                                                                Span::call_site(),
                                                            ),
                                                            arguments: syn::PathArguments::None,
                                                        },
                                                    ]
                                                    .into_iter()
                                                    .collect(),
                                                },
                                            })),
                                            paren_token: token::Paren::default(),
                                            args: [syn::Expr::Lit(syn::ExprLit {
                                                attrs: Vec::new(),
                                                lit: syn::Lit::Str(LitStr::new(
                                                    &format!(
                                                        "Too many arguments passed to {}",
                                                        mod_name
                                                    ),
                                                    Span::call_site(),
                                                )),
                                            })]
                                            .into_iter()
                                            .collect(),
                                        }))),
                                    }),
                                    Some(token::Semi::default()),
                                )],
                            },
                            else_branch: None,
                        }),
                        None,
                    ),
                    Stmt::Local(Local {
                        attrs: Vec::new(),
                        let_token: token::Let::default(),
                        pat: Pat::Slice(PatSlice {
                            attrs: Vec::new(),
                            bracket_token: token::Bracket::default(),
                            elems: (0..max_args)
                                .map(|i| {
                                    Pat::Ident(PatIdent {
                                        attrs: Vec::new(),
                                        by_ref: None,
                                        mutability: None,
                                        ident: Ident::new(&format!("a{}", i), Span::call_site()),
                                        subpat: None,
                                    })
                                })
                                .collect(),
                        }),
                        init: Some(LocalInit {
                            eq_token: token::Eq::default(),
                            expr: Box::new(syn::Expr::MethodCall(syn::ExprMethodCall {
                                attrs: Vec::new(),
                                receiver: Box::new(syn::Expr::MethodCall(syn::ExprMethodCall {
                                    attrs: Vec::new(),
                                    receiver: Box::new(syn::Expr::Path(syn::ExprPath {
                                        attrs: Vec::new(),
                                        qself: None,
                                        path: Ident::new("args", Span::call_site()).into(),
                                    })),
                                    dot_token: token::Dot::default(),
                                    method: Ident::new("try_into", Span::call_site()).into(),
                                    turbofish: None,
                                    paren_token: token::Paren::default(),
                                    args: Punctuated::new(),
                                })),
                                dot_token: token::Dot::default(),
                                method: Ident::new("unwrap", Span::call_site()).into(),
                                turbofish: None,
                                paren_token: token::Paren::default(),
                                args: Punctuated::new(),
                            })),
                            diverge: None,
                        }),
                        semi_token: token::Semi::default(),
                    }),
                    Stmt::Expr(
                        syn::Expr::Match(ExprMatch {
                            attrs: Vec::new(),
                            match_token: token::Match::default(),
                            expr: Box::new(syn::Expr::Tuple(ExprTuple {
                                attrs: Vec::new(),
                                paren_token: token::Paren::default(),
                                elems: (-1..(max_args as i32))
                                    .map(|i| {
                                        if i == -1 {
                                            syn::Expr::Path(syn::ExprPath {
                                                attrs: Vec::new(),
                                                qself: None,
                                                path: Ident::new("this", Span::call_site()).into(),
                                            })
                                        } else {
                                            syn::Expr::Path(syn::ExprPath {
                                                attrs: Vec::new(),
                                                qself: None,
                                                path: Ident::new(
                                                    &format!("a{}", i),
                                                    Span::call_site(),
                                                )
                                                .into(),
                                            })
                                        }
                                    })
                                    .collect(),
                            })),
                            brace_token: token::Brace::default(),
                            arms: item_vec
                                .into_iter()
                                .map(|f| f.as_arm(max_args))
                                .chain(
                                    [Arm {
                                        attrs: Vec::new(),
                                        pat: Pat::Wild(PatWild {
                                            attrs: Vec::new(),
                                            underscore_token: token::Underscore::default(),
                                        }),
                                        guard: None,
                                        fat_arrow_token: token::FatArrow::default(),
                                        body: Box::new(syn::Expr::Call(ExprCall {
                                            attrs: Vec::new(),
                                            func: Box::new(Expr::Path(syn::ExprPath {
                                                attrs: Vec::new(),
                                                qself: None,
                                                path: syn::Path {
                                                    leading_colon: None,
                                                    segments: [
                                                        PathSegment {
                                                            ident: Ident::new(
                                                                "CelValue",
                                                                Span::call_site(),
                                                            ),
                                                            arguments: syn::PathArguments::None,
                                                        },
                                                        PathSegment {
                                                            ident: Ident::new(
                                                                "argument_error",
                                                                Span::call_site(),
                                                            ),
                                                            arguments: syn::PathArguments::None,
                                                        },
                                                    ]
                                                    .into_iter()
                                                    .collect(),
                                                },
                                            })),
                                            paren_token: token::Paren::default(),
                                            args: [syn::Expr::Lit(syn::ExprLit {
                                                attrs: Vec::new(),
                                                lit: Lit::Str(LitStr::new(
                                                    "Invalid arguments passed to func",
                                                    Span::call_site(),
                                                )),
                                            })]
                                            .into_iter()
                                            .collect(),
                                        })),
                                        comma: None,
                                    }]
                                    .into_iter(),
                                )
                                .collect(),
                        }),
                        None,
                    ),
                ],
            }),
        }
    }

    fn cel_value_type() -> Box<Type> {
        Box::new(syn::Type::Path(TypePath {
            qself: None,
            path: syn::Path {
                leading_colon: None,
                segments: [Into::<PathSegment>::into(Ident::new(
                    "CelValue",
                    Span::call_site(),
                ))]
                .into_iter()
                .collect(),
            },
        }))
    }
}
