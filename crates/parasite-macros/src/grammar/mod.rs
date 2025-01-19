pub mod alternation;
pub mod factor;
pub mod production;

use alternation::{AlternationNode, AlternationsNode};
use factor::FactorNode;
use parasite_core::grammar::{Grammar, Id, Key, Symbol};
use production::ProductionNode;
use quote::ToTokens;
use std::collections::HashMap;
use syn::{parse::Parse, Data, DeriveInput, Ident, LitInt, Token};

pub type NodeIndex = Vec<usize>;

#[derive(Debug)]
pub enum Node<'a> {
    Production(&'a ProductionNode, NodeIndex),
    Alternations(&'a AlternationsNode, NodeIndex),
    Alternation(&'a AlternationNode, NodeIndex),
    Factor(&'a FactorNode, NodeIndex),
}

impl<'a> Node<'a> {
    pub fn index(&self) -> &NodeIndex {
        match self {
            Self::Production(_, idx)
            | Self::Alternation(_, idx)
            | Self::Alternations(_, idx)
            | Self::Factor(_, idx) => idx,
        }
    }
}

// Structure to represent grammar rules
#[derive(Debug)]
pub struct GrammarAst {
    productions: Vec<ProductionNode>,
    terminals: Vec<Ident>,
    derived: Vec<Ident>,
    start: Ident,
    token: DeriveInput,
    pub k: u16,
}

impl GrammarAst {
    // TODO also expand user defined recursive productions
    // Aim:
    // No expanded productions that have recursive productions or empty alternations without being annotated as such by ProductionKind
    pub fn expand(self) -> Grammar {
        let mut productions = HashMap::new();
        let mut table = HashMap::new();

        if let Node::Production(production, index) = self.start_production() {
            let key = Key::new(production.lhs.clone().into_token_stream().to_string());
            let mut rule = HashMap::new();
            rule.insert(Id(0), Vec::new());
            productions.insert(key.clone(), rule);
            table.insert(index, key);
        }

        for node in self.iter() {
            match node {
                Node::Production(_, ref index) => {
                    assert!(table.contains_key(index));
                }
                Node::Alternations(alternations, ref index) => {
                    let key = table[index].clone();
                    let alternations = alternations.alternations.as_slice();
                    let len = alternations.len();

                    if len == 1 {
                        let mut index = index.clone();
                        index.push(0);

                        productions[&key].insert(Id(0), Vec::new());
                        table.insert(index, key.clone());
                    } else {
                        for i in 0..len {
                            let mut index = index.clone();
                            index.push(i);

                            let prod_key = Key::new(productions.len().to_string());

                            let mut rule = HashMap::new();
                            rule.insert(Id(0), Vec::new());

                            productions.insert(prod_key.clone(), rule); // ProductionKind::Group
                            productions[&key]
                                .insert(Id(0), vec![Symbol::nonterminal(prod_key.clone())]);
                            table.insert(index, prod_key);
                        }
                    }
                }
                Node::Alternation(_, ref index) => {
                    // original production
                    assert!(table.contains_key(&index[0..index.len() - 1]));
                    // production from alternation
                    assert!(table.contains_key(index));
                }
                Node::Factor(factor, ref index) => {
                    // production from alternation
                    let key = table[&index[0..index.len() - 1]].clone();

                    match factor {
                        FactorNode::Group(_) => {
                            let mut index = index.clone();
                            index.push(0);

                            let prod_key = Key::new(productions.len().to_string());

                            productions.insert(prod_key.clone(), HashMap::new()); // ProductionKind::Group
                                                                                  // diverging alternations are only created in Node::Alternations and point directly to another production
                            productions[&key][&Id(0)].push(Symbol::nonterminal(prod_key.clone()));
                            table.insert(index, prod_key);
                        }
                        FactorNode::Repeat(_) => {
                            let mut inner_idx = index.clone();
                            inner_idx.push(0);

                            let inner_key = Key::new(productions.len().to_string());
                            productions.insert(inner_key.clone(), HashMap::new());
                            table.insert(inner_idx, inner_key.clone());

                            let prod_key = Key::new(productions.len().to_string());
                            let mut rule = HashMap::new();
                            rule.insert(
                                Id(0),
                                vec![
                                    Symbol::nonterminal(inner_key),
                                    Symbol::nonterminal(prod_key.clone()),
                                ],
                            );
                            rule.insert(Id(1), Vec::new());

                            productions.insert(prod_key.clone(), rule);
                            productions[&key][&Id(0)].push(Symbol::nonterminal(prod_key.clone()));
                            table.insert(index.clone(), prod_key);
                        }
                        FactorNode::Optional(_) => {
                            let mut inner_idx = index.clone();
                            inner_idx.push(0);

                            let inner_key = Key::new(productions.len().to_string());
                            productions.insert(inner_key.clone(), HashMap::new());
                            table.insert(inner_idx, inner_key.clone());

                            let prod_key = Key::new(productions.len().to_string());
                            let mut rule = HashMap::new();
                            rule.insert(Id(0), vec![Symbol::nonterminal(inner_key)]);
                            rule.insert(Id(1), Vec::new());

                            productions.insert(prod_key.clone(), rule);
                            productions[&key][&Id(0)].push(Symbol::nonterminal(prod_key.clone()));
                            table.insert(index.clone(), prod_key);
                        }
                        FactorNode::Symbol(ident) => {
                            let symbol_key = Key::new(ident.to_string());

                            if self.is_terminal(ident) {
                                productions[&key][&Id(0)].push(Symbol::terminal(symbol_key));
                            } else if productions.contains_key(&symbol_key) {
                                productions[&key][&Id(0)].push(Symbol::nonterminal(symbol_key))
                            } else if let Some(production) = self.find_production(ident) {
                                let prod_key = Key::new(productions.len().to_string());
                                productions.insert(prod_key.clone(), HashMap::new());
                                productions[&key][&Id(0)]
                                    .push(Symbol::nonterminal(prod_key.clone()));
                                table.insert(production.index().clone(), prod_key);
                            } else {
                                panic!("Identifier is no primitive terminal nor derivated: {ident}")
                            }
                        }
                    }
                }
            }
        }

        Grammar {
            productions,
            start: Key::new(self.start.to_token_stream().to_string()),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Node> {
        self.into_iter()
    }

    pub fn iter_symbols(&self) -> impl Iterator<Item = (&Ident, NodeIndex)> {
        self.iter().filter_map(|node| match node {
            Node::Factor(FactorNode::Symbol(ident), idx) => Some((ident, idx)),
            _ => None,
        })
    }

    pub fn get(&self, mut index: NodeIndex) -> Node {
        let id = index.remove(0);

        self.productions[id].get(index)
    }

    pub fn is_terminal(&self, ident: &Ident) -> bool {
        self.terminals.contains(ident)
    }

    pub fn is_derived(&self, ident: &Ident) -> bool {
        self.derived.contains(ident)
    }

    pub fn start_production(&self) -> Node {
        self.find_production(&self.start).unwrap()
    }

    pub fn find_production(&self, ident: &Ident) -> Option<Node> {
        self.productions.iter().enumerate().find_map(|(id, prod)| {
            if &prod.lhs == ident {
                Some(Node::Production(prod, vec![id]))
            } else {
                None
            }
        })
    }
}

impl<'a> IntoIterator for &'a GrammarAst {
    type IntoIter = GrammarAstIter<'a>;
    type Item = Node<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let stack = vec![self.start_production()];
        let visited = vec![false; self.productions.len()];

        GrammarAstIter {
            ast: self,
            stack,
            visited,
        }
    }
}

pub struct GrammarAstIter<'a> {
    ast: &'a GrammarAst,
    stack: Vec<Node<'a>>,
    visited: Vec<bool>,
}

impl<'a> Iterator for GrammarAstIter<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.stack.pop() {
            match node {
                Node::Production(production, ref idx) => {
                    self.stack
                        .push(Node::Alternations(&production.alternations, idx.clone()));
                    self.visited[idx[0]] = true;
                }
                Node::Alternations(alternations, ref idx) => {
                    for (id, alternation) in alternations.alternations.iter().enumerate() {
                        let mut index = idx.clone();
                        index.push(id);
                        self.stack.push(Node::Alternation(alternation, index))
                    }
                }
                Node::Alternation(alternation, ref idx) => {
                    for (id, factor) in alternation.factors.iter().enumerate().rev() {
                        let mut index = idx.clone();
                        index.push(id);
                        self.stack.push(Node::Factor(factor, index))
                    }
                }
                Node::Factor(factor, ref index) => match factor {
                    FactorNode::Group(alternations)
                    | FactorNode::Optional(alternations)
                    | FactorNode::Repeat(alternations) => {
                        let mut index = index.clone();
                        index.push(0);
                        self.stack.push(Node::Alternations(alternations, index))
                    }
                    FactorNode::Symbol(ident) => {
                        if let Some(production) = self.ast.find_production(ident)
                            && !self.visited[production.index()[0]]
                        {
                            self.stack.push(production)
                        }
                    }
                },
            }
            Some(node)
        } else {
            None
        }
    }
}

impl Parse for GrammarAst {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut start = None;
        let mut k = 3;

        let token = input.parse::<DeriveInput>()?;
        let terminals = match &token.data {
            Data::Enum(data) => data
                .variants
                .iter()
                .map(|variant| variant.ident.clone())
                .collect(),
            _ => return Err(syn::Error::new_spanned(token, "token must be an enum")),
        };

        while input.peek(Token![type]) {
            input.parse::<Token![type]>()?;
            let ident = input.parse::<Ident>()?.to_string();
            input.parse::<Token![=]>()?;

            if ident == "Start" {
                start = Some(input.parse()?);
            } else if ident == "K" {
                let lit = input.parse::<LitInt>()?;
                k = lit.base10_parse::<u16>()?;
            }

            input.parse::<Token![;]>()?;
        }

        let start = match start {
            Some(start) => start,
            None => panic!("A start symbol must be defined"),
        };

        let mut derived = Vec::new();
        let mut productions = Vec::new();
        while !input.is_empty() {
            let production = input.parse::<ProductionNode>()?;
            derived.push(production.lhs.clone());
            productions.push(production);
        }

        Ok(Self {
            productions,
            start,
            token,
            k,
            terminals,
            derived,
        })
    }
}
