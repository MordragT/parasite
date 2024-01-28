#![feature(extend_one)]

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::parse_macro_input;
use syntactical::{syntactical_impl, terminal_impl};

use crate::grammar::grammar_impl;

// mod builder;
mod grammar;
mod syntactical;

#[proc_macro_derive(Syntactical)]
pub fn syntactical(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let item_impl = syntactical_impl(input);

    quote!(
        #item_impl
    )
    .into_token_stream()
    .into()
}

#[proc_macro_derive(Terminal)]
pub fn terminal(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let item_impl = terminal_impl(input.ident);

    quote!(
        #item_impl
    )
    .into_token_stream()
    .into()
}

#[proc_macro]
pub fn grammar(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::ItemMod);
    let item_impls = grammar_impl(input).into_iter();

    quote!(
        #(#item_impls)*
    )
    .into_token_stream()
    .into()
}
