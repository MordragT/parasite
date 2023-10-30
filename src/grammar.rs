use std::fmt;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

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
}

impl fmt::Display for Grammar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "K = {}\nStart = {}\n\n", self.k, &self.start)?;
        write!(f, "Productions\n")?;
        write!(f, "===============\n")?;

        for (id, production) in self.productions.iter().enumerate() {
            let mut output = format!("{}({:?})\t: ", id, production.lhs);
            for tokens in &production.alternations {
                for token in tokens {
                    match token {
                        Token::Terminal(terminal) => {
                            output.push('"');
                            output.push_str(&terminal.to_string());
                            output.push('"');
                        }
                        Token::Derived(id) => {
                            if let ProductionObject::Single(name) = &self.productions[*id].lhs {
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
            .position(|production| match &production.lhs {
                ProductionObject::Single(a) => a == ident,
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

    pub fn interface(&self) -> TokenStream {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct Production {
    pub(crate) lhs: ProductionObject,
    pub(crate) alternations: Vec<Vec<Token>>,
    pub(crate) index: NodeIndex,
}

impl Production {
    pub fn new(lhs: ProductionObject, alternations: Vec<Vec<Token>>, index: NodeIndex) -> Self {
        Self {
            lhs,
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
pub enum ProductionObject {
    Repeat(Vec<ProductionObject>),
    Group(Vec<ProductionObject>),
    Optional(Vec<ProductionObject>),
    Single(Ident),
}

impl ProductionObject {
    pub(crate) fn push(&mut self, object: ProductionObject) {
        match self {
            Self::Repeat(list) | Self::Group(list) | Self::Optional(list) => list.push(object),
            _ => (),
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
