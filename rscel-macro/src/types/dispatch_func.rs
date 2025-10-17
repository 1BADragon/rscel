use proc_macro2::Span;
use syn::{punctuated::Punctuated, token, Arm, Ident, ItemFn, Pat, PatPath, PatTuple, PathSegment};

use super::{dispatch_arg::DispatchArg, dispatch_arg_type::DispatchArgType};

pub struct DispatchFunc {
    pub func: ItemFn,
    pub args: Vec<DispatchArg>,
    pub return_type: DispatchArgType,
}

impl DispatchFunc {
    pub fn from_item(f: ItemFn) -> Self {
        DispatchFunc {
            func: f.clone(),
            args: f
                .sig
                .inputs
                .iter()
                .map(|input| DispatchArg::from_fnarg(input.clone()))
                .collect(),
            return_type: DispatchArgType::from_type(match f.sig.output {
                syn::ReturnType::Default => panic!("Dispatch functions must have return type"),
                syn::ReturnType::Type(_rarrow, ref t) => &t,
            }),
        }
    }

    pub fn into_dispatch_fn(self) -> ItemFn {
        let mangled = self.mangled_name();
        let mut func = self.func;

        let s = func.sig.ident.span();
        func.sig.ident = Ident::new(&mangled, s);

        func
    }

    pub fn mangled_name(&self) -> String {
        let mut dispatch_name = self.func.sig.ident.to_string();
        dispatch_name.push('_');
        dispatch_name.extend(self.args.iter().map(|a| a.mangle_sym()));
        dispatch_name.push(self.return_type.mangle_sym());

        dispatch_name
    }

    pub fn as_arm(&self, max_args: usize) -> Arm {
        let mut elems = Vec::new();
        let mut args: Vec<syn::Expr> = Vec::new();
        let mut arg_index = 0usize;

        if self.args.len() > arg_index && self.args[arg_index].is_this() {
            elems.push(self.args[0].as_pat("this"));
            args.push(syn::Expr::Path(syn::ExprPath {
                attrs: Vec::new(),
                qself: None,
                path: Ident::new("this", Span::call_site()).into(),
            }));
            arg_index += 1;
        } else {
            elems.push(Pat::Path(PatPath {
                attrs: Vec::new(),
                qself: None,
                path: syn::Path {
                    leading_colon: None,
                    segments: [
                        PathSegment {
                            ident: Ident::new("CelValue", Span::call_site()),
                            arguments: syn::PathArguments::None,
                        },
                        PathSegment {
                            ident: Ident::new("Null", Span::call_site()),
                            arguments: syn::PathArguments::None,
                        },
                    ]
                    .into_iter()
                    .collect(),
                },
            }));
        }

        for i in 0..(max_args) {
            if arg_index >= self.args.len() {
                elems.push(Pat::Path(PatPath {
                    attrs: Vec::new(),
                    qself: None,
                    path: syn::Path {
                        leading_colon: None,
                        segments: [
                            PathSegment {
                                ident: Ident::new("CelValue", Span::call_site()),
                                arguments: syn::PathArguments::None,
                            },
                            PathSegment {
                                ident: Ident::new("Null", Span::call_site()),
                                arguments: syn::PathArguments::None,
                            },
                        ]
                        .into_iter()
                        .collect(),
                    },
                }));
            } else {
                let a = format!("a{}", i);
                elems.push(self.args[arg_index].as_pat(&a));
                args.push(syn::Expr::Path(syn::ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path: Ident::new(&a, Span::call_site()).into(),
                }));
                arg_index += 1;
            }
        }

        Arm {
            attrs: Vec::new(),
            pat: Pat::Tuple(PatTuple {
                attrs: Vec::new(),
                paren_token: token::Paren::default(),
                elems: elems.into_iter().collect(),
            }),
            guard: None,
            fat_arrow_token: token::FatArrow::default(),
            body: Box::new(syn::Expr::MethodCall(syn::ExprMethodCall {
                attrs: Vec::new(),
                receiver: Box::new(syn::Expr::Call(syn::ExprCall {
                    attrs: Vec::new(),
                    func: Box::new(syn::Expr::Path(syn::ExprPath {
                        attrs: Vec::new(),
                        qself: None,
                        path: Ident::new(&self.mangled_name(), Span::call_site()).into(),
                    })),
                    paren_token: token::Paren::default(),
                    args: args.into_iter().collect(),
                })),
                dot_token: token::Dot::default(),
                method: Ident::new("into", Span::call_site()),
                turbofish: None,
                paren_token: token::Paren::default(),
                args: Punctuated::new(),
            })),
            comma: Some(token::Comma::default()),
        }
    }
}
