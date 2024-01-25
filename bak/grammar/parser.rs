use core::fmt;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{Block, ItemFn, ItemImpl, ItemStruct, ItemTrait};

use crate::grammar::{token_variants, Token};

use super::{
    first::FirstSets, follow::FollowSets, token_variants::TokenVariants, Grammar, Production,
};

type Row<'a> = Vec<(Vec<&'a Ident>, usize)>;

pub struct Table<'a> {
    // (lhs id, alternation id)
    table: Vec<Row<'a>>,
}

// match tokens.as_slice {
//     for (terminals, aid) in table[pid]
//     quote!(&terminals => .. &terminals1 =>)
//         let tokens = self.productions[pid][aid];
//         for token in tokens {
//             match token {
//                 Terminal(t) => quote!{
//                     if let Some(token) = tokens.pop_front() {
//                         assert!(token.kind() == #t.kind());
//                         let node: #t = token.try_into();
//                     } else {
//                         panic!()
//                     }
//                 }
//                 Derived(pid) => self.state = pid,
//             }
//         }
//     }
// }

impl<'a> fmt::Display for Table<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parsing Table\n")?;
        write!(f, "==============\n")?;

        for (id, set) in self.table.iter().enumerate() {
            let mut output = format!("{id}\t: ");
            for (terminals, aid) in set {
                for ident in terminals {
                    output.push_str(&ident.to_string());
                    output.push(' ');
                }
                output.push_str(&format!("({})\n\t, ", aid));
            }
            output.pop();
            output.pop();
            write!(f, "{output}\n")?;
        }
        Ok(())
    }
}

impl Grammar {
    pub fn table<'a>(&self, first: &'a FirstSets, follow: &'a FollowSets) -> Table<'a> {
        let table = (0..self.productions_count())
            .into_iter()
            .map(|pid| {
                let mut rules = Vec::new();

                let first_set = &first.sets[&pid];
                let follow_set = &follow.sets[&pid];

                for unit in first_set {
                    if unit.item.is_empty() {
                        for item in follow_set {
                            rules.push((item.clone(), unit.aid));
                        }
                    } else {
                        rules.push((unit.item.clone(), unit.aid));
                    }
                }
                rules
            })
            .collect();

        Table { table }
    }

    pub fn parser_decl(&self) -> ItemStruct {
        let token_ident = &self.token.ident;

        syn::parse_quote!(
            pub struct Parser {
                tokens: Vec<#token_ident>,
                stack: Vec<(usize, Option<usize>, usize)>,
            }
        )
    }

    pub fn parser_impl(&self, table: &Table, variants: &TokenVariants) -> ItemImpl {
        let matchings = self
            .productions
            .iter()
            .enumerate()
            .map(|(pid, production)| {
                let row = &table.table[pid];
                let block = Self::parse_production_impl(row, production, variants);
                quote!(
                    #pid => { #block }
                )
            });
        let token_ident = &self.token.ident;
        let start_pid = self.find_start().0;

        syn::parse_quote!(
            impl Parser {
                pub fn new(mut tokens: Vec<#token_ident>) -> Self {
                    Self {
                        tokens,
                        stack: vec![(#start_pid, None, 0)]
                    }
                }

                pub fn parse(&mut self) {
                    while let Some((pid, mut aid, index)) = self.stack.pop() && !self.tokens.is_empty() {
                        match pid {
                            #(#matchings)*
                            _ => panic!("PID unknown")
                        }
                    }
                }
            }
        )
    }

    fn parse_production_impl(
        row: &Row,
        production: &Production,
        variants: &TokenVariants,
    ) -> Block {
        let token_ident = &variants.token_ident;
        let matchings = row.iter().map(|(set, aid)| {
            let tokens = set.iter().map(|ident| {
                let fields = &variants.variants[*ident];
                let pat = TokenVariants::pattern(*ident, fields);
                quote!(#token_ident::#pat)
            });

            quote!(
                &[#(#tokens,)* ..] => {
                    #aid
                }
            )
        });

        let parse = production
            .alternations
            .iter()
            .enumerate()
            .map(|(aid, tokens)| {
                let end = tokens.len();

                let mut block = tokens
                    .iter()
                    .enumerate()
                    .map(|(tid, token)| match token {
                        Token::Terminal(terminal) => quote!(
                            #tid => {
                                let token = self.tokens.remove(0);
                                let node: #terminal = token.try_into().unwrap();
                                dbg!(node);
                                self.stack.push((pid, aid, index + 1));
                            },
                        ),
                        Token::Derived(pid) => quote!(
                            #tid => {
                                self.stack.push((pid, aid, index + 1));
                                self.stack.push((#pid, None, 0));
                                println!("{} {} {}", #pid, alternation_id, index);
                            },
                        ),
                    })
                    .fold(TokenStream::new(), |mut akku, part| {
                        akku.extend_one(part);
                        akku
                    });

                block.extend_one(quote!(
                    #end => (),
                    _ => panic!("INDEX for rule out of bounds"),
                ));
                quote!(
                    #aid => match index {
                        #block
                    }
                )
            });

        syn::parse_quote!({
                let alternation_id = if let Some(aid) = aid {
                    aid
                } else {
                    match self.tokens.as_slice() {
                        #(#matchings)*
                        _ => panic!("MATCHING not found, {:?}", self.tokens),
                    }
                };
                aid = Some(alternation_id);
                match alternation_id {
                    #(#parse)*
                    _ => panic!("ALTERNATION not found, {}", alternation_id),
                }
        })
    }
}
