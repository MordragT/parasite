use std::{collections::VecDeque, fmt};

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    token::{Brace, For, Impl, Struct},
    Data, DeriveInput, Generics, Item, ItemImpl, ItemStruct, Path, Type, TypePath, Fields,
};

use crate::{
    analysis::{first::FirstSets, follow::FollowSets},
    ast::NodeIndex,
};

#[derive(Debug, Clone)]
pub struct Grammar {
    pub(crate) start: Ident,
    pub(crate) k: usize,
    pub(crate) productions: Vec<Production>,
    pub(crate) derived: Vec<Ident>,
    pub(crate) terminals: Vec<Ident>,
    pub(crate) token: DeriveInput,
}

impl fmt::Display for Grammar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "K = {}\nStart = {}\n\n", self.k, &self.start)?;
        write!(f, "Productions\n")?;
        write!(f, "===============\n")?;

        for (id, production) in self.productions.iter().enumerate() {
            let mut output = format!("{}({})\t: ", id, production.kind);
            for tokens in &production.alternations {
                for token in tokens {
                    match token {
                        Token::Terminal(terminal) => {
                            output.push('"');
                            output.push_str(&terminal.to_string());
                            output.push('"');
                        }
                        Token::Derived(id) => {
                            if let ProductionKind::Instance(name) = &self.productions[*id].kind {
                                output.push_str(&name.to_string())
                            } else {
                                output.push_str(&id.to_string())
                            }
                        }
                    }
                    output.push(' ');
                }
                output.push_str("\n\t| ");
            }
            output.pop();
            output.pop();

            write!(f, "{output}\n")?;
        }
        Ok(())
    }
}

impl Grammar {
    pub fn k(&self) -> usize {
        self.k
    }

    pub fn insert(&mut self, production: Production) -> usize {
        let id = self.productions.len();

        self.productions.push(production);

        id
    }

    pub fn insert_with<F>(&mut self, f: F) -> usize
    where
        F: FnOnce(usize) -> Production,
    {
        let id = self.productions.len();
        let production = f(id);
        self.productions.push(production);
        id
    }

    // pub fn insert_empty(&mut self) -> usize {
    //     let id = self.productions.len();

    //     self.productions.push(Production::empty());

    //     id
    // }

    pub fn find_id(&self, ident: &Ident) -> Option<usize> {
        self.productions
            .iter()
            .position(|production| match &production.kind {
                ProductionKind::Instance(name) => name == ident,
                _ => false,
            })
    }

    pub fn find_start(&self) -> (usize, &Production) {
        let id = self.find_id(&self.start).unwrap();
        (id, &self.productions[id])
    }

    pub fn get(&self, id: usize) -> &Production {
        &self.productions[id]
    }

    pub fn get_mut(&mut self, id: usize) -> &mut Production {
        &mut self.productions[id]
    }

    pub fn iter_productions(&self) -> impl Iterator<Item = &Production> {
        self.productions.iter()
    }

    pub fn contains_left_recursion(&self) -> bool {
        !self
            .productions
            .iter()
            .enumerate()
            .all(|(id, production)| !production.is_left_recursive(id))
    }

    pub fn first_sets(&self) -> FirstSets {
        FirstSets::build(self)
    }

    pub fn follow_sets<'a>(&'a self, first_sets: &'a FirstSets<'a>) -> FollowSets<'a> {
        FollowSets::build(self, first_sets)
    }

    pub fn generate(&self) -> TokenStream {
        let mut results = self
            .productions
            .iter()
            .map(|production| {
                if let ProductionKind::Instance(ident) = &production.kind {
                    Some(quote!(#ident))
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

        let functions = self
            .productions
            .iter()
            .enumerate()
            .zip(parameters.into_iter().zip(results))
            .map(|((id, production), (param, ret))| {
                let param = param.unwrap();
                let ret = ret.unwrap();
                let name = if let ProductionKind::Instance(ident) = &production.kind {
                    ident.to_string().to_lowercase()
                } else {
                    format!("production{id}")
                };
                let name = Ident::new(&name, Span::call_site());

                if production.kind.is_instance() {
                    quote! { fn #name(&self, input: #param) -> Result<#ret, Self::Error>; }
                } else {
                    quote! { fn #name(&self, input: #param) -> Result<#ret, Self::Error> {
                        Ok(input)
                    } }
                }
            });

        let token = &self.token;

        let variant_types = match token.data.clone() {
            Data::Enum(data) => {
                data.variants
                    .into_iter()
                    .fold(TokenStream::new(), |mut akku, variant| {
                        let structure: Item = ItemStruct {
                            attrs: Vec::new(),
                            vis: syn::Visibility::Inherited,
                            struct_token: Struct::default(),
                            ident: variant.ident.clone(),
                            generics: Generics::default(),
                            fields: variant.fields.clone(),
                            semi_token: None,
                        }
                        .into();
                        
                        let token_ident = token.ident.clone();
                        let variant_ident = variant.ident;
                        let variant_fields = match &variant.fields {
                            Fields::Unnamed(fields) => {
                                let fields = (0..fields.unnamed.len()).into_iter().map(|n| Ident::new(&format!("n{n}"), Span::call_site()));
                                let elems = syn::parse_quote!(#(#fields),*);

                                let tuple = syn::ExprTuple {
                                    attrs: Vec::new(),
                                    paren_token: syn::token::Paren::default(),
                                    elems
                                };
                                tuple.to_token_stream()
                            }
                            _ => variant.fields.to_token_stream()
                        };

                        let arm: syn::Arm = syn::parse_quote!(
                            #token_ident::#variant_ident #variant_fields => Ok(#variant_ident #variant_fields)
                        );

                        let try_from: ItemImpl = syn::parse_quote!(
                            impl TryFrom<#token_ident> for #variant_ident {
                                type Error = String;

                                fn try_from(token: #token_ident) -> Result<#variant_ident, Self::Error> {
                                    match token {
                                        #arm,
                                        _ => Err("Token not of variant kind".to_owned())
                                    }
                                }
                            }
                        );

                        akku.extend_one(structure.to_token_stream());
                        akku.extend_one(try_from.to_token_stream());
                        akku
                    })
            }
            _ => unreachable!(),
        };

        quote! {
            trait Grammar {
                type Error;

                #( #functions )*
            }

            #token
            #variant_types
        }
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

        quote!(Option<#param>)
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

        quote!(Vec<#param>)
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

            quote!(#variant_name<#(#params),*>)
        }
    }

    fn group_alternation_parameter(
        &self,
        alternation: &Vec<Token>,
        results: &Vec<Option<TokenStream>>,
    ) -> TokenStream {
        if let [token] = alternation.as_slice() {
            match token {
                Token::Terminal(ident) => quote!(#ident),
                Token::Derived(other_id) => results[*other_id].as_ref().unwrap().clone(),
            }
        } else {
            let params = alternation.iter().map(|token| match token {
                Token::Terminal(ident) => quote!(#ident),
                Token::Derived(other_id) => results[*other_id].as_ref().unwrap().clone(),
            });
            quote!(
                (#(#params),*)
            )
        }
    }
}

#[derive(Debug, Clone)]
pub struct Production {
    pub(crate) kind: ProductionKind,
    pub(crate) alternations: Vec<Vec<Token>>,
    pub(crate) index: NodeIndex,
}

impl Production {
    pub fn new(kind: ProductionKind, alternations: Vec<Vec<Token>>, index: NodeIndex) -> Self {
        Self {
            kind,
            alternations,
            index,
        }
    }

    pub fn alternations_count(&self) -> usize {
        self.alternations.len()
    }

    pub fn alternation_mut(&mut self, id: usize) -> &mut Vec<Token> {
        &mut self.alternations[id]
    }

    pub fn alternations(&self) -> &Vec<Vec<Token>> {
        &self.alternations
    }

    pub fn is_left_recursive(&self, id: usize) -> bool {
        !self
            .alternations
            .iter()
            .all(|tokens| tokens.first() != Some(&Token::Derived(id)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProductionKind {
    Repeat,
    Optional,
    Group,
    Instance(Ident),
}

impl ProductionKind {
    pub fn is_instance(&self) -> bool {
        match self {
            Self::Instance(_) => true,
            _ => false,
        }
    }
}

impl fmt::Display for ProductionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Group => write!(f, "group"),
            Self::Optional => write!(f, "optional"),
            Self::Repeat => write!(f, "repeat"),
            Self::Instance(ident) => write!(f, "{ident}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Token {
    Terminal(Ident),
    Derived(usize),
}

impl Token {
    pub fn is_terminal(&self) -> bool {
        match self {
            Self::Terminal(_) => true,
            _ => false,
        }
    }
}
