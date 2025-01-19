#![feature(extend_one)]
#![feature(let_chains)]

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::parse_macro_input;

use crate::grammar::GrammarAst;
use crate::module::module_check;
use crate::syntactical::{syntactical_impl, terminal_impl};

mod grammar;
mod module;
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
pub fn module(input: TokenStream) -> TokenStream {
    let module = input.clone();
    let mut module = parse_macro_input!(module as syn::ItemMod);

    module_check(&mut module);

    module.into_token_stream().into()
}

// Macro to define grammar rules and generate the Grammar trait
#[proc_macro]
pub fn grammar(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as GrammarAst);

    let k = ast.k;

    assert!(k > 0);

    let grammar = ast.expand();
    println!("{grammar}");

    let table = grammar.table(k as usize);
    println!("{table}");

    TokenStream::new()
}
