use super::{Grammar, Production};
use crate::grammar::{ProductionKind, Token};
use proc_macro2::{Ident, Span, TokenStream};
use std::collections::VecDeque;
use syn::{token::Semi, Block, ItemTrait, TraitItemFn};

#[derive(Debug)]
pub struct Interface {
    methods: Vec<Method>,
}

impl Interface {
    pub fn declaration(&self) -> ItemTrait {
        let methods = self
            .methods
            .iter()
            .enumerate()
            .map(|(id, method)| method.declaration(id));

        syn::parse_quote!(
            trait Grammar {
                type Error;

                #( #methods )*
            }

        )
    }
}

#[derive(Debug)]
pub struct Method {
    name: Option<Ident>,
    param: TokenStream,
    result: TokenStream,
    is_instance: bool,
}

impl Method {
    pub fn declaration(&self, id: usize) -> TraitItemFn {
        let Self {
            name,
            param,
            result,
            is_instance,
        } = &self;

        let (block, semi): (Option<Block>, Option<Semi>) = if !is_instance {
            (Some(syn::parse_quote!({ Ok(input) })), None)
        } else {
            (None, Some(Semi::default()))
        };

        let name = if let Some(name) = name {
            name.to_string().to_lowercase()
        } else {
            format!("production{id}")
        };
        let name = Ident::new(&name, Span::call_site());

        syn::parse_quote!(
            fn #name(&self, input: #param) -> Result<#result, Self::Error> #semi
            #block
        )
    }
}

impl Grammar {
    pub fn interface(&self) -> Interface {
        let mut results = self
            .productions
            .iter()
            .map(|production| {
                if let ProductionKind::Instance(ident) = &production.kind {
                    let result: TokenStream = syn::parse_quote!(#ident);
                    Some(result)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let mut queue = VecDeque::from_iter(0..results.len());

        while let Some(id) = queue.pop_front() {
            if results[id].is_none() {
                let production = &self.productions[id];

                if production
                    .alternations
                    .iter()
                    .flatten()
                    .all(|token| match token {
                        Token::Derived(other_id) => results[*other_id].is_some() || id == *other_id,
                        Token::Terminal(_) => true,
                    })
                {
                    results[id] = Some(self.production_parameter(production, &results));
                } else {
                    queue.push_back(id);
                }
            }
        }

        let mut parameters = self
            .productions
            .iter()
            .enumerate()
            .map(|(id, production)| {
                if !production.kind.is_instance() {
                    Some(results[id].clone().unwrap())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let mut queue = VecDeque::from_iter(0..parameters.len());

        while let Some(id) = queue.pop_front() {
            if parameters[id].is_none() {
                let production = &self.productions[id];

                if production
                    .alternations
                    .iter()
                    .flatten()
                    .all(|token| match token {
                        Token::Derived(other_id) => {
                            parameters[*other_id].is_some() || id == *other_id
                        }
                        Token::Terminal(_) => true,
                    })
                {
                    parameters[id] = Some(self.production_parameter(production, &results));
                } else {
                    queue.push_back(id);
                }
            }
        }

        let methods = self
            .productions
            .iter()
            .zip(parameters.into_iter().zip(results))
            .map(|(production, (param, result))| {
                let param = param.unwrap();
                let result = result.unwrap();

                Method {
                    name: production.kind.to_instance().cloned(),
                    param,
                    result,
                    is_instance: production.kind.is_instance(),
                }
            })
            .collect();

        Interface { methods }
    }

    fn production_parameter(
        &self,
        production: &Production,
        results: &Vec<Option<TokenStream>>,
    ) -> TokenStream {
        match production.kind {
            ProductionKind::Group | ProductionKind::Instance(_) => {
                self.group_production_parameter(production, results)
            }
            ProductionKind::Optional => self.optional_production_parameter(production, results),
            ProductionKind::Repeat => self.repeat_production_parameter(production, results),
        }
    }

    fn optional_production_parameter(
        &self,
        production: &Production,
        results: &Vec<Option<TokenStream>>,
    ) -> TokenStream {
        let param = match production.alternations[0][0] {
            Token::Derived(other_id) => results[other_id].clone().unwrap(),
            Token::Terminal(_) => unreachable!(),
        };

        syn::parse_quote!(Option<#param>)
    }

    fn repeat_production_parameter(
        &self,
        production: &Production,
        results: &Vec<Option<TokenStream>>,
    ) -> TokenStream {
        let param = match production.alternations[0][0] {
            Token::Derived(other_id) => results[other_id].clone().unwrap(),
            Token::Terminal(_) => unreachable!(),
        };

        syn::parse_quote!(Vec<#param>)
    }

    fn group_production_parameter(
        &self,
        production: &Production,
        results: &Vec<Option<TokenStream>>,
    ) -> TokenStream {
        if let [alternation] = production.alternations.as_slice() {
            self.group_alternation_parameter(alternation, &results)
        } else {
            let len = production.alternations.len();
            let variant_name = Ident::new(&format!("Sum{len}"), Span::call_site());

            let params = production
                .alternations
                .iter()
                .map(|alternation| self.group_alternation_parameter(alternation, &results));

            syn::parse_quote!(#variant_name<#(#params),*>)
        }
    }

    fn group_alternation_parameter(
        &self,
        alternation: &Vec<Token>,
        results: &Vec<Option<TokenStream>>,
    ) -> TokenStream {
        if let [token] = alternation.as_slice() {
            match token {
                Token::Terminal(ident) => syn::parse_quote!(#ident),
                Token::Derived(other_id) => results[*other_id].as_ref().unwrap().clone(),
            }
        } else {
            let params = alternation.iter().map(|token| match token {
                Token::Terminal(ident) => syn::parse_quote!(#ident),
                Token::Derived(other_id) => results[*other_id].as_ref().unwrap().clone(),
            });
            syn::parse_quote!(
                (#(#params),*)
            )
        }
    }
}
