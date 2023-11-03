#![feature(let_chains)]
#![feature(exact_size_is_empty)]
#![feature(slice_group_by)]
#![feature(extend_one)]

use crate::ast::GrammarAst;
use proc_macro::TokenStream;
use syn::parse_macro_input;

mod ast;
mod grammar;
// mod parser;

// #[proc_macro_attribute]
// pub fn parasite(args: TokenStream, input: TokenStream) -> TokenStream {
//     let arg = parse_macro_input!(args as syn::Ident);
//     match arg.to_string().as_str() {
//         "token" => {
//             let stream = input.clone();
//             let token = syn::parse_macro_input!(stream as syn::DeriveInput);

//             let terminals = match token.data {
//                 Data::Enum(token) => token
//                     .variants
//                     .into_iter()
//                     .map(|variant| variant.ident)
//                     .collect::<Vec<_>>(),
//                 _ => {
//                     return syn::Error::new_spanned(arg, "token must be an enum")
//                         .to_compile_error()
//                         .into()
//                 }
//             };

//             input
//         }
//         _ => syn::Error::new_spanned(arg, "invalid argument")
//             .to_compile_error()
//             .into(),
//     }
// }

// Macro to define grammar rules and generate the Grammar trait
#[proc_macro]
pub fn grammar(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as GrammarAst);

    let grammar = ast.expand();
    println!("{grammar}");

    assert!(grammar.k > 0);
    // TODO also detect indirect left recursion
    assert!(!grammar.contains_left_recursion());

    let first = grammar.first_sets();
    println!("{first}");

    let follow = grammar.follow_sets(&first);
    println!("{follow}");

    let table = grammar.table(&first, &follow);
    println!("{table}");

    let interface = grammar.interface();
    let token_variants = grammar.token_variants();

    let stream = grammar.generate(&interface, &token_variants, &table);
    println!("{stream}");

    stream.into()
}
