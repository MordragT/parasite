pub mod alternation;
pub mod factor;
pub mod production;

use crate::grammar::{Grammar, Production, ProductionKind, Token};
use alternation::{AlternationNode, AlternationsNode};
use factor::FactorNode;
use production::ProductionNode;
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
    k: u16,
}

impl GrammarAst {
    // TODO also expand user defined recursive productions
    // Aim:
    // No expanded productions that have recursive productions or empty alternations without being annotated as such by ProductionKind
    pub fn expand(self) -> Grammar {
        let mut productions = Vec::new();
        let mut table = HashMap::new();

        if let Node::Production(production, index) = self.start_production() {
            let ident = production.lhs.clone();
            productions.push(Production::new(
                ProductionKind::Instance(ident.clone()),
                Vec::new(),
                index.clone(),
            ));
            table.insert(index, 0);
        }

        for node in self.iter() {
            match node {
                Node::Production(_, ref index) => {
                    assert!(table.contains_key(index));
                }
                Node::Alternations(alternations, ref index) => {
                    let id = table[index];
                    let alternations = alternations.alternations.as_slice();
                    let len = alternations.len();

                    if len == 1 {
                        let mut index = index.clone();
                        index.push(0);

                        table.insert(index, id);
                        productions[id].alternations.push(Vec::new());
                    } else {
                        for i in 0..len {
                            let mut index = index.clone();
                            index.push(i);

                            let prod_id = productions.len();
                            let prod =
                                Production::new(ProductionKind::Group, vec![vec![]], index.clone());

                            table.insert(index, prod_id);
                            productions.push(prod);
                            productions[id]
                                .alternations
                                .push(vec![Token::Derived(prod_id)]);
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
                    let id = table[&index[0..index.len() - 1]];

                    match factor {
                        FactorNode::Group(_) => {
                            let mut index = index.clone();
                            index.push(0);
                            let prod_id = productions.len();
                            let prod =
                                Production::new(ProductionKind::Group, Vec::new(), index.clone());
                            table.insert(index, prod_id);
                            productions.push(prod);

                            // diverging alternations are only created in Node::Alternations and point directly to another production
                            productions[id].alternations[0].push(Token::Derived(prod_id));
                        }
                        FactorNode::Repeat(_) => {
                            let mut inner_idx = index.clone();
                            inner_idx.push(0);
                            let inner_id = productions.len();
                            let inner = Production::new(
                                ProductionKind::Group,
                                Vec::new(),
                                inner_idx.clone(),
                            );
                            table.insert(inner_idx, inner_id);
                            productions.push(inner);

                            let prod_id = productions.len();
                            let prod = Production::new(
                                ProductionKind::Repeat,
                                vec![
                                    vec![Token::Derived(inner_id), Token::Derived(prod_id)],
                                    vec![],
                                ],
                                index.clone(),
                            );
                            table.insert(index.clone(), prod_id);
                            productions.push(prod);

                            productions[id].alternations[0].push(Token::Derived(prod_id));
                        }
                        FactorNode::Optional(_) => {
                            let mut inner_idx = index.clone();
                            inner_idx.push(0);
                            let inner_id = productions.len();
                            let inner = Production::new(
                                ProductionKind::Group,
                                Vec::new(),
                                inner_idx.clone(),
                            );
                            table.insert(inner_idx, inner_id);
                            productions.push(inner);

                            let prod_id = productions.len();
                            let prod = Production::new(
                                ProductionKind::Optional,
                                vec![vec![Token::Derived(inner_id)], vec![]],
                                index.clone(),
                            );
                            table.insert(index.clone(), prod_id);
                            productions.push(prod);

                            productions[id].alternations[0].push(Token::Derived(prod_id));
                        }
                        FactorNode::Symbol(ident) => {
                            if self.is_terminal(ident) {
                                productions[id].alternations[0]
                                    .push(Token::Terminal(ident.clone()));
                            } else if let Some(prod_id) = productions.iter().position(|prod| {
                                prod.kind == ProductionKind::Instance(ident.clone())
                            }) {
                                productions[id].alternations[0].push(Token::Derived(prod_id))
                            } else if let Some(production) = self.find_production(ident) {
                                let prod_id = productions.len();
                                let prod = Production::new(
                                    ProductionKind::Instance(ident.clone()),
                                    Vec::new(),
                                    production.index().clone(),
                                );
                                table.insert(production.index().clone(), prod_id);
                                productions.push(prod);

                                productions[id].alternations[0].push(Token::Derived(prod_id))
                            } else {
                                panic!("Identifier is no primitive terminal nor derivated: {ident}")
                            }
                            // productions[id].expr.push(IdExpr::Single(ident.clone()));
                        }
                    }
                }
            }
        }

        let Self {
            productions: _,
            terminals,
            derived,
            start,
            token,
            k,
        } = self;

        Grammar {
            start,
            k: k as usize,
            productions,
            terminals,
            derived,
            token,
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
                    self
                    .stack
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
                        if let Some(production) = self.ast.find_production(ident) && !self.visited[production.index()[0]] {
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
