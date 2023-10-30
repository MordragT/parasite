#![feature(let_chains)]
#![feature(exact_size_is_empty)]
#![feature(slice_group_by)]

use proc_macro::TokenStream;
use syn::parse_macro_input;

use crate::{
    analysis::{first::FirstSets, follow::FollowSets},
    ast::GrammarAst,
};

mod analysis;
mod ast;
mod generation;
mod grammar;
// mod parser;

// Macro to define grammar rules and generate the Grammar trait
#[proc_macro]
pub fn grammar(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as GrammarAst);

    let grammar = ast.expand();
    println!("{grammar}");

    assert!(grammar.k > 0);
    // TODO also detect indirect left recursion
    assert!(!grammar.contains_left_recursion());

    let interface = grammar.interface();

    let first = grammar.first_sets();
    println!("{first}");

    let follow = grammar.follow_sets(&first);
    println!("{follow}");

    interface.into()
}
