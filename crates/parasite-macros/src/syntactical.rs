use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{DeriveInput, Field, Ident, ItemImpl, Stmt, Variant};

pub fn terminal_impl(ident: Ident) -> ItemImpl {
    syn::parse_quote!(
        impl parasite_core::builder::Syntactical for #ident {}
    )
}

pub fn syntactical_impl(parsed: DeriveInput) -> ItemImpl {
    let ident = parsed.ident;

    let rule_stmts = match parsed.data {
        syn::Data::Enum(data) => {
            let stmts = enum_rule(Vec::from_iter(data.variants));
            TokenStream::from_iter(stmts.map(ToTokens::into_token_stream))
        }
        syn::Data::Struct(data) => struct_rule(Vec::from_iter(data.fields)).into_token_stream(),
        _ => unimplemented!(),
    };

    syn::parse_quote!(
        impl parasite_core::builder::Syntactical for #ident {
            fn generate(grammar: &mut parasite_core::grammar::Grammar, stack: &mut Vec<parasite_core::grammar::TypeName>) -> parasite_core::grammar::Symbol {
                let key = parasite_core::grammar::TypeName::of::<Self>();

                if !Self::visited(grammar, stack) {
                    stack.push(key);

                    let mut rule = parasite_core::grammar::Rule::new();
                    #rule_stmts

                    grammar.insert(key, rule);
                }

                parasite_core::grammar::Symbol::nonterminal(key)
            }
        }
    )
}

fn field_calls(field: Field) -> TokenStream {
    let ty = field.ty;
    quote!(<#ty as parasite_core::builder::Syntactical>::generate(grammar, stack))
}

pub fn struct_rule(fields: Vec<Field>) -> Stmt {
    let calls = fields.into_iter().map(field_calls);
    syn::parse_quote!(rule.insert(parasite_core::grammar::Id(0), vec![#(#calls ,)*]);)
}

pub fn enum_rule(variants: Vec<Variant>) -> impl Iterator<Item = Stmt> {
    variants.into_iter().enumerate().map(|(id, variant)| {
        let Variant { fields, .. } = variant;
        let calls = fields.into_iter().map(field_calls);
        syn::parse_quote!(rule.insert(parasite_core::grammar::Id(#id), vec![#(#calls ,)*]);)
    })
}
