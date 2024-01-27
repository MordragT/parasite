#![feature(extend_one)]

use builder::syntactical_impl;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::parse_macro_input;

use crate::parser::parser_impl;

mod builder;
mod parser;

#[proc_macro_derive(Syntactical, attributes(Terminal))]
pub fn syntactical(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let syntactical_impl = syntactical_impl(input);

    quote!(
        #syntactical_impl
    )
    .into_token_stream()
    .into()
}

#[proc_macro]
pub fn grammar(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::ItemMod);
    let parser_impl = parser_impl(input);

    quote!(
        #parser_impl
    )
    .into_token_stream()
    .into()
}
