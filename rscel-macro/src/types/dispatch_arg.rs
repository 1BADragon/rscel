use std::str::FromStr;

use proc_macro2::Span;
use syn::{token, FnArg, Ident, Pat, PatIdent, PatTupleStruct, PathSegment};

use super::dispatch_arg_type::DispatchArgType;

#[derive(Clone, Debug)]
pub struct DispatchArg {
    this: bool,
    arg_type: DispatchArgType,
}

impl DispatchArg {
    pub fn from_fnarg(arg: FnArg) -> Self {
        match &arg {
            FnArg::Receiver(_) => panic!("Receiver args not allowed"),
            FnArg::Typed(t) => {
                let ident = match t.pat.as_ref() {
                    Pat::Ident(i) => i.ident.to_string(),
                    _ => panic!("Only basic args allowed"),
                };

                DispatchArg {
                    this: ident == "this",
                    arg_type: DispatchArgType::from_type(&t.ty),
                }
            }
        }
    }

    pub fn mangle_sym(&self) -> String {
        let mut s = if self.this {
            String::from_str("z").unwrap()
        } else {
            String::new()
        };

        s.push(self.arg_type.mangle_sym());

        s
    }

    pub fn is_this(&self) -> bool {
        self.this
    }

    pub fn as_pat(&self, ident: &str) -> Pat {
        match self.arg_type {
            DispatchArgType::CelValue => Pat::Ident(PatIdent {
                attrs: Vec::new(),
                by_ref: None,
                mutability: None,
                ident: Ident::new(ident, Span::call_site()),
                subpat: None,
            }),
            _ => Pat::TupleStruct(PatTupleStruct {
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
                            ident: Ident::new(self.arg_type.celvalue_enum(), Span::call_site()),
                            arguments: syn::PathArguments::None,
                        },
                    ]
                    .into_iter()
                    .collect(),
                },
                paren_token: token::Paren::default(),
                elems: [Pat::Ident(PatIdent {
                    attrs: Vec::new(),
                    by_ref: None,
                    mutability: None,
                    ident: Ident::new(ident, Span::call_site()),
                    subpat: None,
                })]
                .into_iter()
                .collect(),
            }),
        }
    }
}
