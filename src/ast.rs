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
        type StackItem<'a> = (Node<'a>, usize);

        let mut expanded = Grammar::new(self.start.clone(), self.k as usize);

        let start_ident = self.start.clone();
        let start_id = expanded.insert(Production::new(
            ProductionObject::Single(start_ident.clone()),
            Vec::new(),
        ));

        let mut stack: Vec<StackItem> = Vec::new();
        if let Some(start) = self.start_production() {
            stack.push((Node::Alternations(&start.alternations), start_id));
        }

        while let Some((node, id)) = stack.pop() {
            match node {
                Node::Production(production) => {
                    stack.push((Node::Alternations(&production.alternations), id));
                }
                Node::Alternations(alternations) => {
                    let alternations = alternations.alternations.as_slice();

                    if let [alternation] = alternations {
                        stack.push((Node::Alternation(alternation, 0), id));

                        expanded.get_mut(id).alternations.push(Vec::new());
                    } else {
                        for alternation in alternations {
                            let inner_id = expanded.insert(Production::new(
                                ProductionObject::Group(Vec::new()),
                                vec![vec![]],
                            ));

                            stack.push((Node::Alternation(alternation, 0), inner_id));
                            expanded
                                .get_mut(id)
                                .alternations
                                .push(vec![Token::Derived(inner_id)]);
                        }
                    }
                }
                Node::Alternation(alternation, alternation_id) => {
                    for factor in alternation.factors.iter().rev() {
                        stack.push((Node::Factor(factor, alternation_id), id));
                    }
                }
                Node::Factor(factor, alternation_id) => match factor {
                    FactorNode::Group(alternations) => {
                        let group_id = expanded.insert({
                            Production::new(ProductionObject::Group(Vec::new()), Vec::new())
                        });
                        stack.push((Node::Alternations(alternations), group_id));
                        expanded
                            .get_mut(id)
                            .alternation_mut(alternation_id)
                            .push(Token::Derived(group_id));
                    }
                    FactorNode::Repeat(alternations) => {
                        let inner_repeat_id = expanded.insert_empty();
                        let repeat_id = expanded.insert_with(|repeat_id| {
                            Production::new(
                                ProductionObject::Repeat(Vec::new()),
                                vec![
                                    vec![
                                        Token::Derived(inner_repeat_id),
                                        Token::Derived(repeat_id),
                                    ],
                                    vec![],
                                ],
                            )
                        });

                        stack.push((Node::Alternations(alternations), inner_repeat_id));
                        expanded
                            .get_mut(id)
                            .alternation_mut(alternation_id)
                            .push(Token::Derived(repeat_id));
                    }
                    FactorNode::Optional(alternations) => {
                        let inner_optional_id = expanded.insert_empty();
                        let optional_id = expanded.insert({
                            Production::new(
                                ProductionObject::Optional(Vec::new()),
                                vec![vec![Token::Derived(inner_optional_id)], vec![]],
                            )
                        });

                        stack.push((Node::Alternations(alternations), inner_optional_id));
                        expanded
                            .get_mut(id)
                            .alternation_mut(alternation_id)
                            .push(Token::Derived(optional_id));
                    }
                    FactorNode::Symbol(ident) => {
                        if self.is_terminal(ident) {
                            expanded
                                .get_mut(id)
                                .alternation_mut(alternation_id)
                                .push(Token::Terminal(ident.clone()));
                        } else if let Some(production_id) = expanded.find_id(ident) {
                            expanded
                                .get_mut(id)
                                .alternation_mut(alternation_id)
                                .push(Token::Derived(production_id));
                        } else {
                            let production = self.find_production(ident).unwrap();
                            let production_id = expanded.insert(Production::new(
                                ProductionObject::Single(ident.clone()),
                                Vec::new(),
                            ));

                            stack.push((Node::Production(production), production_id));
                            expanded
                                .get_mut(id)
                                .alternation_mut(alternation_id)
                                .push(Token::Derived(production_id));
                        }
                        expanded
                            .get_mut(id)
                            .lhs
                            .push(ProductionObject::Single(ident.clone()));
                    }
                },
            }
        }

        dbg!(&expanded);

        expanded
    }

    fn iter(&self) -> impl Iterator<Item = Node> {
        let mut stack = vec![Node::Production(self.start_production().unwrap())];

        std::iter::from_fn(move || {
            while let Some(node) = stack.pop() {
                match node {
                    Node::Production(production) => {
                        stack.push(Node::Alternations(&production.alternations))
                    }
                    Node::Alternations(alternations) => {
                        for alternation in &alternations.alternations {
                            stack.push(Node::Alternation(alternation))
                        }
                    }
                    Node::Alternation(alternation) => {
                        for factor in alternation.factors.iter().rev() {
                            stack.push(Node::Factor(factor))
                        }
                    }
                    Node::Factor(factor) => match factor {
                        FactorNode::Group(group) => stack.push(Node::Alternations(group)),
                        FactorNode::Optional(optional) => stack.push(Node::Alternations(optional)),
                        FactorNode::Repeat(repeat) => stack.push(Node::Alternations(repeat)),
                        FactorNode::Symbol(_) => (),
                    },
                }
                return Some(node);
            }
            None
        })
    }

    fn is_terminal(&self, ident: &Ident) -> bool {
        self.terminals.contains(ident)
    }

    fn is_derived(&self, ident: &Ident) -> bool {
        self.derived.contains(ident)
    }

    fn start_production(&self) -> Option<&ProductionNode> {
        self.productions.iter().find(|prod| prod.lhs == self.start)
    }

    fn find_production(&self, ident: &Ident) -> Option<&ProductionNode> {
        self.productions.iter().find(|prod| prod.lhs == *ident)
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

#[derive(Debug)]
enum Node<'a> {
    Production(&'a ProductionNode),
    Alternations(&'a AlternationsNode),
    Alternation(&'a AlternationNode),
    Factor(&'a FactorNode),
}

#[derive(Debug)]
struct ProductionNode {
    lhs: Ident,
    /// split by |
    alternations: AlternationsNode,
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
struct AlternationsNode {
    alternations: Vec<AlternationNode>,
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
struct AlternationNode {
    /// split by ' '
    factors: Vec<FactorNode>,
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
enum FactorNode {
    // '(' Alternations ')'
    Group(AlternationsNode),
    // '{' Alternations '}'
    Repeat(AlternationsNode),
    // '[' Alternations ']'
    Optional(AlternationsNode),
    Symbol(Ident),
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
