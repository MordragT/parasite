//! A concrete syntax tree (parse tree) is the syntax tree that stores the full representation of the parsed document.
//! It's is a low level representation of the parsed source in the structure defined by a grammar description.
//! It should be possible to rewrite the original document for a concrete syntax tree.
//! It represents every detail (such as white-space in white-space insensitive languages)
//! This is the first tree build by a parser.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, ItemImpl, Type};

pub enum Data {
    Struct(Struct),
    Enum(Enum),
}

impl Data {
    pub fn syntactical_impl(self) -> ItemImpl {
        let node_impl = match self {
            Self::Struct(data) => data.syntactical_impl(),
            Self::Enum(data) => data.syntactical_impl(),
        };

        node_impl
    }
}

pub struct Struct {
    pub ident: Ident,
    pub fields: Vec<Field>,
}

impl Struct {
    pub fn syntactical_impl(self) -> ItemImpl {
        let Self { ident, fields } = self;

        let calls = fields.into_iter().map(Field::calls);

        syn::parse_quote!(
            impl parasite_core::builder::Syntactical for #ident {
                fn generate(grammar: &mut parasite_core::grammar::Grammar, stack: &mut Vec<parasite_core::grammar::TypeName>) -> parasite_core::grammar::Symbol {
                    let key = parasite_core::grammar::TypeName::of::<Self>();

                    if !Self::visited(grammar, stack) {
                        stack.push(key);

                        let mut rule = parasite_core::grammar::Rule::new();
                        rule.insert(parasite_core::grammar::Id(0), vec![#(#calls ,)*]);

                        grammar.insert(key, rule);
                    }

                    parasite_core::grammar::Symbol::nonterminal(key)
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
    pub fn syntactical_impl(self) -> ItemImpl {
        let Self { ident, variants } = self;

        let alternations = variants.into_iter().enumerate().map(|(id, variant)| {
            let Variant { ident: _, fields } = variant;
            let calls = fields.into_iter().map(Field::calls);
            quote!(rule.insert(parasite_core::grammar::Id(#id), vec![#(#calls ,)*]))
        });

        syn::parse_quote!(
            impl parasite_core::builder::Syntactical for #ident {
                fn generate(grammar: &mut parasite_core::grammar::Grammar, stack: &mut Vec<parasite_core::grammar::TypeName>) -> parasite_core::grammar::Symbol {
                    let key = parasite_core::grammar::TypeName::of::<Self>();

                    if !Self::visited(grammar, stack) {
                        stack.push(key);

                        let mut rule = parasite_core::grammar::Rule::new();
                        #(#alternations ;)*

                        grammar.insert(key, rule);
                    }

                    parasite_core::grammar::Symbol::nonterminal(key)
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

impl Field {
    pub fn calls(self) -> TokenStream {
        let ty = self.ty;
        quote!(<#ty as parasite_core::builder::Syntactical>::generate(grammar, stack))
    }
}

pub struct Terminal {
    pub ident: Ident,
}

impl Terminal {
    pub fn syntactical_impl(self) -> ItemImpl {
        let Self { ident } = self;

        syn::parse_quote!(
            impl parasite_core::builder::Syntactical for #ident {}
        )
    }
}
