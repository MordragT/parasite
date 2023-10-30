use std::collections::HashMap;

use syn::{
    braced, bracketed, parenthesized,
    parse::Parse,
    token::{Brace, Bracket, Paren},
    Ident, LitInt, Token,
};

use crate::grammar::{Grammar, Production, ProductionObject, Token};

// Structure to represent grammar rules
#[derive(Debug)]
pub struct GrammarAst {
    productions: Vec<ProductionNode>,
    terminals: Vec<Ident>,
    derived: Vec<Ident>,
    start: Ident,
    k: u16,
}

impl GrammarAst {
    pub(crate) fn expand(self) -> Grammar {
        let mut productions = Vec::new();
        let mut table = HashMap::new();

        if let Node::Production(production, index) = self.start_production() {
            productions.push(Production::new(
                ProductionObject::Single(production.lhs.clone()),
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

                    if let [alternation] = alternations {
                        let mut index = index.clone();
                        index.push(0);

                        table.insert(index, id);
                        productions[id].alternations.push(Vec::new());
                    } else {
                        for (i, alternation) in alternations.iter().enumerate() {
                            let mut index = index.clone();
                            index.push(i);

                            let prod = Production::new(
                                ProductionObject::Group(Vec::new()),
                                vec![vec![]],
                                index.clone(),
                            );
                            let prod_id = productions.len();

                            table.insert(index, prod_id);
                            productions.push(prod);
                            productions[id]
                                .alternations
                                .push(vec![Token::Derived(prod_id)]);
                        }
                    }
                }
                Node::Alternation(alternation, ref index) => {
                    // original production
                    assert!(table.contains_key(&index[0..index.len() - 1]));
                    // production from alternation
                    assert!(table.contains_key(index));
                }
                Node::Factor(factor, ref index) => {
                    // production from alternation
                    let id = table[&index[0..index.len() - 1]];

                    match factor {
                        FactorNode::Group(alternations) => {
                            let mut index = index.clone();
                            index.push(0);
                            let prod_id = productions.len();
                            let prod = Production::new(
                                ProductionObject::Group(Vec::new()),
                                Vec::new(),
                                index.clone(),
                            );
                            table.insert(index, prod_id);
                            productions.push(prod);

                            // diverging alternations are only created in Node::Alternations and point directly to another production
                            productions[id].alternations[0].push(Token::Derived(prod_id));
                        }
                        FactorNode::Repeat(alternations) => {
                            let mut inner_idx = index.clone();
                            inner_idx.push(0);
                            let inner_id = productions.len();
                            let inner = Production::new(
                                ProductionObject::Group(Vec::new()),
                                Vec::new(),
                                inner_idx.clone(),
                            );
                            table.insert(inner_idx, inner_id);
                            productions.push(inner);

                            let prod_id = productions.len();
                            let prod = Production::new(
                                ProductionObject::Repeat(Vec::new()),
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
                        FactorNode::Optional(alternations) => {
                            let mut inner_idx = index.clone();
                            inner_idx.push(0);
                            let inner_id = productions.len();
                            let inner = Production::new(
                                ProductionObject::Optional(Vec::new()),
                                Vec::new(),
                                inner_idx.clone(),
                            );
                            table.insert(inner_idx, inner_id);
                            productions.push(inner);

                            let prod_id = productions.len();
                            let prod = Production::new(
                                ProductionObject::Repeat(Vec::new()),
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
                                prod.lhs == ProductionObject::Single(ident.clone())
                            }) {
                                productions[id].alternations[0].push(Token::Derived(prod_id))
                            } else if let Some(production) = self.find_production(ident) {
                                let prod = Production::new(
                                    ProductionObject::Single(ident.clone()),
                                    Vec::new(),
                                    production.index().clone(),
                                );
                                let prod_id = productions.len();
                                table.insert(production.index().clone(), prod_id);
                                productions.push(prod);

                                productions[id].alternations[0].push(Token::Derived(prod_id))
                            } else {
                                panic!("Identifier is no primitive terminal nor derivated: {ident}")
                            }
                            productions[id]
                                .lhs
                                .push(ProductionObject::Single(ident.clone()));
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
            k,
        } = self;

        Grammar {
            start,
            k: k as usize,
            productions,
            terminals,
            derived,
        }
    }

    fn iter(&self) -> impl Iterator<Item = Node> {
        self.into_iter()
    }

    fn iter_symbols(&self) -> impl Iterator<Item = (&Ident, NodeIndex)> {
        self.iter().filter_map(|node| match node {
            Node::Factor(FactorNode::Symbol(ident), idx) => Some((ident, idx)),
            _ => None,
        })
    }

    fn get(&self, mut index: NodeIndex) -> Node {
        let id = index.remove(0);

        self.productions[id].get(index)
    }

    fn is_terminal(&self, ident: &Ident) -> bool {
        self.terminals.contains(ident)
    }

    fn is_derived(&self, ident: &Ident) -> bool {
        self.derived.contains(ident)
    }

    fn start_production(&self) -> Node {
        self.find_production(&self.start).unwrap()
    }

    fn find_production(&self, ident: &Ident) -> Option<Node> {
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
        let mut terminals = Vec::new();

        while input.peek(Token![type]) {
            input.parse::<Token![type]>()?;
            let ident = input.parse::<Ident>()?.to_string();
            input.parse::<Token![=]>()?;

            if ident == "Terminals" {
                terminals.push(input.parse()?);
                while input.peek(Token!(|)) {
                    input.parse::<Token!(|)>()?;
                    let terminal = input.parse()?;
                    terminals.push(terminal);
                }
            } else if ident == "Start" {
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
            k,
            terminals,
            derived,
        })
    }
}

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

#[derive(Debug)]
pub struct ProductionNode {
    lhs: Ident,
    /// split by |
    alternations: AlternationsNode,
}

impl ProductionNode {
    fn get(&self, index: NodeIndex) -> Node {
        if !index.is_empty() {
            self.alternations.get(index)
        } else {
            Node::Production(self, Vec::new())
        }
    }
}

impl Parse for ProductionNode {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lhs = input.parse()?;
        let _colon = input.parse::<Token![:]>()?;
        let alternations = input.parse()?;
        let _semi = input.parse::<Token![;]>()?;

        Ok(Self { lhs, alternations })
    }
}

#[derive(Debug)]
pub struct AlternationsNode {
    alternations: Vec<AlternationNode>,
}

impl AlternationsNode {
    fn get(&self, mut index: NodeIndex) -> Node {
        if !index.is_empty() {
            let id = index.remove(0);
            self.alternations[id].get(index)
        } else {
            Node::Alternations(self, Vec::new())
        }
    }
}

impl Parse for AlternationsNode {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut alternations = vec![input.parse()?];
        while input.peek(Token!(|)) {
            let _pipe = input.parse::<Token!(|)>()?;
            let alternation = input.parse()?;
            alternations.push(alternation);
        }

        Ok(Self { alternations })
    }
}

#[derive(Debug)]
pub struct AlternationNode {
    /// split by ' '
    factors: Vec<FactorNode>,
}

impl AlternationNode {
    fn get(&self, mut index: NodeIndex) -> Node {
        if !index.is_empty() {
            let id = index.remove(0);
            self.factors[id].get(index)
        } else {
            Node::Alternation(self, Vec::new())
        }
    }
}

impl Parse for AlternationNode {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut factors = vec![input.parse()?];
        while !input.peek(Token![;])
        // && !input.peek(Paren)
        // && !input.peek(Brace)
        // && !input.peek(Bracket)
        && !input.peek(Token!(|))
        && !input.is_empty()
        {
            let factor = input.parse()?;
            factors.push(factor);
        }
        Ok(Self { factors })
    }
}

#[derive(Debug)]
pub enum FactorNode {
    // '(' Alternations ')'
    Group(AlternationsNode),
    // '{' Alternations '}'
    Repeat(AlternationsNode),
    // '[' Alternations ']'
    Optional(AlternationsNode),
    Symbol(Ident),
}

impl FactorNode {
    fn get(&self, mut index: NodeIndex) -> Node {
        if !index.is_empty() {
            let id = index.remove(0);
            match self {
                Self::Group(alternations)
                | Self::Optional(alternations)
                | Self::Repeat(alternations) => alternations.get(index),
                Self::Symbol(ident) => Node::Factor(self, Vec::new()),
            }
        } else {
            Node::Factor(self, Vec::new())
        }
    }
}

impl Parse for FactorNode {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;

        let factor = if input.peek(Paren) {
            parenthesized!(content in input);
            let alternations = content.parse()?;
            Self::Group(alternations)
        } else if input.peek(Brace) {
            braced!(content in input);
            let alternations = content.parse()?;
            Self::Repeat(alternations)
        } else if input.peek(Bracket) {
            bracketed!(content in input);
            let alternations = content.parse()?;
            Self::Optional(alternations)
        } else {
            Self::Symbol(input.parse()?)
        };

        Ok(factor)
    }
}
