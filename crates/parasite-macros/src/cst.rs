//! A concrete syntax tree (parse tree) is the syntax tree that stores the full representation of the parsed document.
//! It's is a low level representation of the parsed source in the structure defined by a grammar description.
//! It should be possible to rewrite the original document for a concrete syntax tree.
//! It represents every detail (such as white-space in white-space insensitive languages)
//! This is the first tree build by a parser. 

use proc_macro2::Span;
use quote::quote;
use syn::{ExprStruct, Ident, ItemImpl, Type};

pub enum Data {
    Struct(Struct),
    Enum(Enum),
}

impl Data {
    pub fn node_impl(self) -> ItemImpl {
        let node_impl = match self {
            Self::Struct(data) => data.node_impl(),
            Self::Enum(data) => data.node_impl(),
        };

        node_impl
    }
}

pub struct Struct {
    pub ident: Ident,
    pub fields: Vec<Field>,
}

impl Struct {
    pub fn node_impl(self) -> ItemImpl {
        let Self { ident, fields } = self;

        let ident_str = ident.to_string().to_lowercase();
        let (calls, vars): (Vec<_>, Vec<_>) = fields.into_iter().enumerate().map(|(i, field)| {
            let ty = field.ty;
            let var = Ident::new(&format!("n{i}"), Span::call_site());

            let call = quote!(
                let #var = <#ty as parasite_core::ast::nodes::Node>::item(ast, &nonterminal);
            );

            (call, var)
        }).unzip();

        syn::parse_quote!(
            impl parasite_core::ast::nodes::Node for #ident {
                fn item(ast: &mut parasite_core::ast::Ast, current: &parasite_core::ast::Nonterminal) -> parasite_core::ast::Item {
                    let nonterminal = #ident_str.into();

                    if &nonterminal == current {
                        parasite_core::ast::Item::Symbol(parasite_core::ast::Symbol::Recursive(nonterminal))
                    } else if let Some(idx) = ast.find_production_idx(&nonterminal) {
                        parasite_core::ast::Item::Symbol(parasite_core::ast::Symbol::Nonterminal(idx))
                    } else {
                        #(#calls)*

                        let idx = ast.insert_production(nonterminal.clone(), parasite_core::ast::Rhs::Items(vec![#(#vars ,)*]));
                        parasite_core::ast::Item::Symbol(parasite_core::ast::Symbol::Nonterminal(idx))
                    }
                }
            }
        )
    }
}

pub struct Enum {
    pub ident: Ident,
    pub variants: Vec<Variant>,
}

impl Enum {
    pub fn node_impl(self) -> ItemImpl {
        let Self { ident, variants } = self;
        let ident_str = ident.to_string().to_lowercase();

        let (variants, calls): (Vec<_>, Vec<_>) = variants.into_iter().map(|variant| {
            let Variant {
                ident: variant_ident,
                fields,
            } = variant;
            let variant_ident_str = variant_ident.to_string().to_lowercase();

            let (calls, vars): (Vec<_>, Vec<_>) = fields.into_iter().enumerate().map(|(i, field)| {
                let ty = field.ty;
                let var = Ident::new(&format!("{variant_ident_str}{i}"), Span::call_site());
    
                let call = quote!(
                    let #var = <#ty as parasite_core::ast::nodes::Node>::item(ast, &nonterminal);
                );
    
                (call, var)
            }).unzip();

            let expr: ExprStruct = syn::parse_quote!(
                parasite_core::ast::Alternation {
                    ident: #variant_ident_str.into(),
                    items: vec![#(#vars,)*]
                }
            );
            (expr, calls)
        }).unzip();

        let calls = calls.into_iter().flatten();

        syn::parse_quote!(
            impl parasite_core::ast::nodes::Node for #ident {
                fn item(ast: &mut parasite_core::ast::Ast, current: &parasite_core::ast::Nonterminal) -> parasite_core::ast::Item {
                    let nonterminal = #ident_str.into();

                    if &nonterminal == current {
                        parasite_core::ast::Item::Symbol(parasite_core::ast::Symbol::Recursive(nonterminal))
                    } else if let Some(idx) = ast.find_production_idx(&nonterminal) {
                        parasite_core::ast::Item::Symbol(parasite_core::ast::Symbol::Nonterminal(idx))
                    } else {
                        #(#calls)*

                        let idx = ast.insert_production(nonterminal.clone(), parasite_core::ast::Rhs::Alternations(vec![#(#variants,)*]));
                        parasite_core::ast::Item::Symbol(parasite_core::ast::Symbol::Nonterminal(idx))
                    }
                }
            }
        )
    }
}

pub struct Variant {
    pub ident: Ident,
    pub fields: Vec<Field>,
}
pub struct Field {
    pub ty: Type,
}

pub struct Terminal {
    pub ident: Ident,
}

impl Terminal {
    pub fn node_impl(self) -> ItemImpl {
        let Self { ident } = self;

        let terminal = ident.to_string().to_lowercase();

        syn::parse_quote!(
            impl parasite_core::ast::nodes::Node for #ident {
                fn item(ast: &mut parasite_core::ast::Ast, current: &parasite_core::ast::Nonterminal) -> parasite_core::ast::Item {
                    let terminal = #terminal.into();
                    let idx = ast.insert_terminal(terminal);
                    parasite_core::ast::Item::Symbol(parasite_core::ast::Symbol::Terminal(idx))
                }
            }
        )
    }
}