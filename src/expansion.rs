use std::fmt;

use crate::{Alternation, Alternations, Factor, GrammarDefinition, IdentSet, Production};
use proc_macro2::Ident;

#[derive(Debug, Clone)]
pub struct ExpandedGrammar {
    start: Ident,
    k: usize,
    production_ids: Vec<Option<Ident>>,
    productions: Vec<ExpandedProduction>,
}

impl fmt::Display for ExpandedGrammar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "K = {}\nStart = {}\n\n", self.k, &self.start)?;
        write!(f, "Productions\n")?;
        write!(f, "===============\n")?;

        for production in &self.productions {
            let mut output = if let Some(ident) = &self.production_ids[production.id] {
                format!("{ident}\t: ")
            } else {
                format!("{}\t: ", production.id)
            };
            for tokens in &production.alternations {
                for token in tokens {
                    match token {
                        Token::Terminal(terminal) => {
                            output.push('"');
                            output.push_str(&terminal.to_string());
                            output.push('"');
                        }
                        Token::Nonterminal(id) => {
                            if let Some(ident) = &self.production_ids[*id] {
                                output.push_str(&ident.to_string())
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

impl ExpandedGrammar {
    pub fn new(start: Ident, k: usize) -> Self {
        Self {
            start,
            k,
            production_ids: Vec::new(),
            productions: Vec::new(),
        }
    }

    pub fn k(&self) -> usize {
        self.k
    }

    pub fn alloc(&mut self) -> usize {
        let id = self.productions.len();

        self.productions.push(ExpandedProduction::new(id));
        self.production_ids.push(None);

        id
    }

    pub fn find_id(&self, ident: &Ident) -> Option<usize> {
        self.production_ids
            .iter()
            .position(|id| id.as_ref() == Some(ident))
    }

    pub fn find_start(&self) -> (usize, &ExpandedProduction) {
        let id = self.find_id(&self.start).unwrap();
        (id, &self.productions[id])
    }

    pub fn get(&self, id: usize) -> (Option<&Ident>, &ExpandedProduction) {
        assert!(self.productions.len() > id);
        assert!(self.production_ids.len() > id);

        let ident = self.production_ids[id].as_ref();
        let production = &self.productions[id];

        (ident, production)
    }

    pub fn get_mut(&mut self, id: usize) -> (&mut Option<Ident>, &mut ExpandedProduction) {
        assert!(self.productions.len() > id);
        assert!(self.production_ids.len() > id);

        let ident = &mut self.production_ids[id];
        let production = &mut self.productions[id];

        (ident, production)
    }

    pub fn get_production(&self, id: usize) -> &ExpandedProduction {
        assert!(self.productions.len() > id);

        &self.productions[id]
    }

    pub fn get_mut_production(&mut self, id: usize) -> &mut ExpandedProduction {
        assert!(self.productions.len() > id);

        &mut self.productions[id]
    }

    pub fn get_mut_id(&mut self, id: usize) -> &mut Option<Ident> {
        assert!(self.production_ids.len() > id);

        &mut self.production_ids[id]
    }

    pub fn iter_productions(&self) -> impl Iterator<Item = &ExpandedProduction> {
        self.productions.iter()
    }

    pub fn contains_left_recursion(&self) -> bool {
        !self
            .productions
            .iter()
            .all(|production| !production.is_left_recursive())
    }
}

#[derive(Debug, Clone)]
pub struct ExpandedProduction {
    pub(crate) id: usize,
    alternations: Vec<Vec<Token>>,
}

impl ExpandedProduction {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            alternations: Vec::new(),
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

    pub fn is_left_recursive(&self) -> bool {
        !self
            .alternations
            .iter()
            .all(|tokens| tokens.first() != Some(&Token::Nonterminal(self.id)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Terminal(Ident),
    Nonterminal(usize),
}

impl Token {
    pub fn is_terminal(&self) -> bool {
        match self {
            Self::Terminal(_) => true,
            _ => false,
        }
    }
}

enum Node<'a> {
    Production(&'a Production),
    Alternations(&'a Alternations),
    Alternation(&'a Alternation, usize),
    Factor(&'a Factor, usize),
}

type StackItem<'a> = (Node<'a>, usize);

pub struct Expander<'a> {
    grammar: &'a GrammarDefinition,
}

impl<'a> Expander<'a> {
    pub(crate) fn new(grammar: &'a GrammarDefinition) -> Self {
        Self { grammar }
    }

    pub(crate) fn expand(self) -> ExpandedGrammar {
        let mut expanded =
            ExpandedGrammar::new(self.grammar.start.clone(), self.grammar.k as usize);

        let start_id = expanded.alloc();
        let start_ident = expanded.get_mut_id(start_id);
        *start_ident = Some(self.grammar.start.clone());

        let mut stack: Vec<StackItem> = Vec::new();
        if let Some(start) = self.grammar.start() {
            stack.push((Node::Alternations(dbg!(&start.alternations)), start_id));
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

                        expanded
                            .get_mut_production(id)
                            .alternations
                            .push(Vec::new());
                    } else {
                        for alternation in alternations {
                            let inner_id = expanded.alloc();

                            stack.push((Node::Alternation(alternation, 0), inner_id));
                            expanded
                                .get_mut_production(inner_id)
                                .alternations
                                .push(Vec::new());

                            expanded
                                .get_mut_production(id)
                                .alternations
                                .push(vec![Token::Nonterminal(inner_id)]);
                        }
                    }
                }
                Node::Alternation(alternation, alternation_id) => {
                    for factor in alternation.factors.iter().rev() {
                        stack.push((Node::Factor(factor, alternation_id), id));
                    }
                }
                Node::Factor(factor, alternation_id) => match factor {
                    Factor::Group(alternations) => {
                        let group_id = expanded.alloc();
                        stack.push((Node::Alternations(alternations), group_id));
                        expanded
                            .get_mut_production(id)
                            .alternation_mut(alternation_id)
                            .push(Token::Nonterminal(group_id));
                    }
                    Factor::Repeat(alternations) => {
                        let repeat_id = expanded.alloc();
                        let inner_repeat_id = expanded.alloc();

                        let repeat_production = expanded.get_mut_production(repeat_id);
                        repeat_production.alternations.push(vec![
                            Token::Nonterminal(inner_repeat_id),
                            Token::Nonterminal(repeat_id),
                        ]);
                        repeat_production.alternations.push(Vec::new());

                        stack.push((Node::Alternations(alternations), inner_repeat_id));
                        expanded
                            .get_mut_production(id)
                            .alternation_mut(alternation_id)
                            .push(Token::Nonterminal(repeat_id));
                    }
                    Factor::Optional(alternations) => {
                        let optional_id = expanded.alloc();
                        let inner_optional_id = expanded.alloc();

                        let optional_production = expanded.get_mut_production(optional_id);
                        optional_production
                            .alternations
                            .push(vec![Token::Nonterminal(inner_optional_id)]);
                        optional_production.alternations.push(Vec::new());

                        stack.push((Node::Alternations(alternations), inner_optional_id));
                        expanded
                            .get_mut_production(id)
                            .alternation_mut(alternation_id)
                            .push(Token::Nonterminal(optional_id));
                    }
                    Factor::Symbol(ident) => {
                        if self.grammar.is_terminal(ident) {
                            expanded
                                .get_mut_production(id)
                                .alternation_mut(alternation_id)
                                .push(Token::Terminal(ident.clone()));
                        } else if let Some(production_id) = expanded.find_id(ident) {
                            expanded
                                .get_mut_production(id)
                                .alternation_mut(alternation_id)
                                .push(Token::Nonterminal(production_id));
                        } else {
                            let production = self.grammar.find_production(ident).unwrap();
                            let production_id = expanded.alloc();

                            *expanded.get_mut_id(production_id) = Some(ident.clone());

                            stack.push((Node::Production(production), production_id));
                            expanded
                                .get_mut_production(id)
                                .alternation_mut(alternation_id)
                                .push(Token::Nonterminal(production_id));
                        }
                    }
                },
            }
        }

        expanded
    }
}
