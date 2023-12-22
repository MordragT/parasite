#![feature(extend_one)]

use cst::Data;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, ItemImpl};

use crate::cst::{Enum, Field, Struct, Terminal, Variant};

mod cst;

#[proc_macro_derive(Node, attributes(Start, Terminal))]
pub fn node(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as syn::DeriveInput);
    let ident = parsed.ident;

    if parsed
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("Terminal"))
        .is_some()
    {
        let terminal = Terminal { ident };
        let node_impl = terminal.node_impl();

        return node_impl.into_token_stream().into();
    }

    let start_impl = if let Some(start) = parsed
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("Start"))
    {
        // let expr: ExprAssign = start.parse_args()
        let start_impl: ItemImpl = syn::parse_quote!(
            impl parasite_core::ast::nodes::Start for #ident {}
        );
        start_impl.into_token_stream()
    } else {
        proc_macro2::TokenStream::new()
    };

    let data = match parsed.data {
        syn::Data::Enum(data) => {
            let variants = data
                .variants
                .into_iter()
                .map(|variant| {
                    let fields = variant
                        .fields
                        .into_iter()
                        .map(|field| Field { ty: field.ty })
                        .collect();
                    Variant {
                        fields,
                        ident: variant.ident,
                    }
                })
                .collect();
            Data::Enum(Enum { ident, variants })
        }
        syn::Data::Struct(data) => {
            let fields = data
                .fields
                .into_iter()
                .map(|field| Field { ty: field.ty })
                .collect();
            Data::Struct(Struct { ident, fields })
        }
        _ => unimplemented!(),
    };

    let node_impl = data.node_impl();

    quote!(
        #node_impl
        #start_impl
    )
    .into_token_stream()
    .into()
}
