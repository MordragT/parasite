use std::collections::VecDeque;

use proc_macro2::TokenStream;
use quote::quote;

use crate::grammar::{Grammar, Production, Token};

pub struct Derivation<'a> {
    alternation: VecDeque<&'a Token>,
    token_stream: Vec<TokenStream>,
}

pub fn interface(grammar: &Grammar) -> TokenStream {
    let mut derivations = grammar
        .iter_productions()
        .map(|production| {
            if let Some(name) = &production.name {
                production
                    .alternations
                    .iter()
                    .map(|_| Derivation {
                        alternation: VecDeque::new(),
                        token_stream: vec![quote!(#name)],
                    })
                    .collect::<Vec<_>>()
            } else {
                production
                    .alternations
                    .iter()
                    .map(|alternation| Derivation {
                        alternation: VecDeque::from_iter(alternation),
                        token_stream: Vec::new(),
                    })
                    .collect::<Vec<_>>()
            }
        })
        .collect::<Vec<_>>();

    let mut queue = (0..derivations.len()).collect::<VecDeque<_>>();

    while let Some(id) = queue.pop_front() {
        let to_process = derivations[id]
            .iter_mut()
            .map(|item| item.alternation.pop_front())
            .collect::<Vec<_>>();

        if to_process.iter().all(|token| token.is_none()) {
            continue;
        }

        for (alternation_id, token) in to_process.into_iter().enumerate() {
            match token {
                Some(Token::Terminal(terminal)) => derivations[id][alternation_id]
                    .token_stream
                    .push(quote!(#terminal)),
                Some(Token::Derived(other_id)) => {
                    let other = &derivations[*other_id];

                    if other
                        .iter()
                        .all(|derivation| derivation.alternation.is_empty())
                    {}
                }
                None => continue,
            }
        }
    }

    todo!()
}

// pub fn generate_trait(&self) -> proc_macro2::TokenStream {
//     let functions = self.productions.iter().enumerate().map(|(id, production)| {
//         let name = if let Some(name) = production.name {
//             name.to_string().to_lowercase()
//         } else {
//             format!("production{}", id)
//         };
//         let name = Ident::new(&name, Span::call_site());
//         let result = production.result;
//         let parameter = self.alternations.generate_parameter();

//         quote! { fn #name(&self, input: #parameter) -> Result<#result, Self::Error>; }
//     });

//     quote! {
//         trait Grammar {
//             type Error;

//             #( #functions )*
//         }
//     }
// }

// fn generate_function(&self) -> proc_macro2::TokenStream {
//     let name = if let Some(name) = self.name {
//         name.to_lowercase()
//     } else {
//         format!("production{}", self.id)
//     };
//     let name = Ident::new(&name, Span::call_site());
//     let result = self.result;
//     let parameter = self.alternations.generate_parameter();

//     quote! { fn #name(&self, input: #parameter) -> Result<#result, Self::Error>; }
// }

// impl FactorNode {
//     fn generate_parameter(&self) -> proc_macro2::TokenStream {
//         match self {
//             Self::Group(alternations) => {
//                 let inner = alternations.generate_parameter();
//                 quote!(#inner)
//             }
//             Self::Repeat(alternations) => {
//                 let inner = alternations.generate_parameter();

//                 quote!(Vec<#inner>)
//             }
//             Self::Optional(alternations) => {
//                 let inner = alternations.generate_parameter();

//                 quote!(Option<#inner>)
//             }
//             Self::Symbol(ident) => {
//                 quote!(#ident)
//             }
//         }
//     }
// }

// impl AlternationNode {
//     fn generate_parameter(&self) -> proc_macro2::TokenStream {
//         if self.factors.len() == 1 {
//             self.factors[0].generate_parameter()
//         } else {
//             let inner = self
//                 .factors
//                 .iter()
//                 .map(|factor| factor.generate_parameter());
//             quote!(
//                 (#(#inner),*)
//             )
//         }
//     }
// }

// impl AlternationsNode {
//     fn generate_parameter(&self) -> proc_macro2::TokenStream {
//         let len = self.alternations.len();

//         if len == 1 {
//             self.alternations[0].generate_parameter()
//         } else {
//             let inner = self
//                 .alternations
//                 .iter()
//                 .map(|alternation| alternation.generate_parameter());

//             let variant_name = Ident::new(&format!("Sum{len}"), Span::call_site());

//             quote!(#variant_name<#(#inner),*>)
//             // match self.alternations.len() {
//             //     2 => ,
//             //     _ => panic!("More than 16 alternations are not supported"),
//             // }
//         }
//     }
// }
