use self::{first::FirstSets, follow::FollowSets};
use crate::ast::NodeIndex;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::fmt;
use syn::DeriveInput;

pub mod first;
pub mod follow;
mod interface;
mod parser;
mod token_variants;

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

    pub fn productions_count(&self) -> usize {
        self.productions.len()
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
        let interface = self.interface();
        let interface_decl = interface.declaration();

        let token_decl = &self.token;
        let token_variants = self.token_variants();
        let token_variants_decl = token_variants.declarations();
        let token_variants_try_from_impls = token_variants.try_from_impls();

        let stream = quote!(
            #interface_decl

            #token_decl
            #( #token_variants_decl )*
            #( #token_variants_try_from_impls )*
        );

        stream
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

    pub fn ident(&self) -> Option<&Ident> {
        match self {
            Self::Instance(ident) => Some(ident),
            _ => None,
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
