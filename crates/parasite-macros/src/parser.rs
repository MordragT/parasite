use syn::{Item, ItemMod};

pub fn parser_impl(input: ItemMod) -> ItemMod {
    // if let Some((_, items)) = input.content {
    //     let data = items.into_iter().filter_map(|item| match item {
    //         Item::Enum(item) => {
    //             let x: i32 = 10;
    //             todo!()
    //         }
    //         Item::Struct(item) => {
    //             let x: i32 = 10;
    //             todo!()
    //         }
    //         _ => None,
    //     });
    // }
    todo!()
}

// use parasite_core::grammar::{table::Table, Grammar, TypeName};
// use proc_macro2::{Ident, Span, TokenStream};
// use quote::ToTokens;
// use syn::{ItemFn, ItemImpl};

// mod node;

// pub struct Generator {
//     table: Table,
//     grammar: Grammar,
//     k: usize,
//     token_ty: TypeName,
// }

// impl Generator {
//     pub fn new(grammar: Grammar, k: usize, token_ty: TypeName) -> Self {
//         let table = grammar.table(k);

//         Self {
//             table,
//             grammar,
//             k,
//             token_ty,
//         }
//     }

//     pub fn parser_impl(&self) -> TokenStream {
//         let token = Ident::new(self.token_ty.as_str(), Span::call_site());
//         let ast = Ident::new(self.grammar.start.as_str(), Span::call_site());

//         let fun: ItemFn = syn::parse_quote!(
//             pub fn parse(input: logos::Lexer<#token>) -> Result<#ast, Box<dyn std::error::Error>> {

//             }
//         );

//         fun.into_token_stream()
//     }

//     pub fn parse_symbols_impl(&self, symbols: &Vec<Symbol>) {

//     }

//     pub fn parse_rule_impl(&self) -> TokenStream {
//         let token = Ident::new(self.token_ty.as_str(), Span::call_site());

//         self.grammar.productions.iter().map(|(key, rule)| {
//             let ident = Ident::new(key.as_str(), Span::call_site());

//             rule.iter().map(|(id, symbols)| {
//                 let method_name = format!("parse_{}", id.0);
//                 let parse_impl: ItemImpl = syn::parse_quote!(
//                     impl #ident {
//                         pub fn #method_name(input: logos::Lexer<#token>) -> Result<#ident, Box<dyn std::error::Error>> {

//                         }
//                     }
//                 );
//                 parse_impl
//             });
//             todo!()
//         })

//         todo!()
//     }

// }
