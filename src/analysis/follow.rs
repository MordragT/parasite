use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
};

use proc_macro2::Ident;
use sum::Sum2;

use crate::{
    first::FirstSets, Alternation, Alternations, Factor, GrammarDefinition, IdentSet, Production,
};

#[derive(Debug, Clone)]
pub struct FollowSets<'a> {
    sets: HashMap<&'a Ident, IdentSet<'a>>,
    tails: VecDeque<(&'a Ident, &'a Ident)>,
}

impl<'a> FollowSets<'a> {
    pub fn new() -> Self {
        Self {
            sets: HashMap::new(),
            tails: VecDeque::new(),
        }
    }

    pub fn push_tail(&mut self, ident: &'a Ident, tail: &'a Ident) {
        self.tails.push_back((ident, tail));
    }

    pub fn insert(&mut self, ident: &'a Ident, set: IdentSet<'a>) {
        self.sets.insert(ident, set);
    }

    pub fn get(&'a self, ident: &'a Ident) -> Option<&'a IdentSet<'a>> {
        self.sets.get(ident)
    }

    pub fn get_mut(&'a mut self, ident: &'a Ident) -> Option<&'a mut IdentSet<'a>> {
        self.sets.get_mut(ident)
    }

    pub fn union(&mut self, ident: &'a Ident, set: IdentSet<'a>) {
        if let Some(ident_set) = self.sets.get_mut(ident) {
            ident_set.extend(set);
        } else {
            self.sets.insert(ident, set);
        }
    }
}

#[derive(Debug, Clone)]
pub struct FollowBuilder<'a> {
    grammar: &'a GrammarDefinition,
    first: FirstSets<'a>,
}

impl<'a> FollowBuilder<'a> {
    pub(crate) fn new(grammar: &'a GrammarDefinition, first: FirstSets<'a>) -> Self {
        Self { grammar, first }
    }

    pub(crate) fn build(&'a self) -> FollowSets<'a> {
        let mut cache = FollowSets::new();

        for nonterminal in &self.grammar.nonterminals {
            for production in &self.grammar.productions {
                self.normal(
                    &production.lhs,
                    &production.alternations,
                    nonterminal,
                    &mut cache,
                );
            }
        }

        while let Some((left, tail)) = cache.tails.pop_front() {
            if cache.tails.iter().find(|(l, _)| tail == *l).is_none() {
                let set = cache.sets.get(tail).unwrap().clone();
                cache.sets.get_mut(left).unwrap().extend(set);
            } else if left != tail {
                cache.tails.push_back((left, tail));
            }
        }

        cache
    }

    fn normal(
        &'a self,
        left: &'a Ident,
        alternations: &'a Alternations,
        nonterminal: &'a Ident,
        cache: &mut FollowSets<'a>,
    ) {
        for alternation in &alternations.alternations {
            let mut factor_iter = alternation.factors.iter();
            let mut success = false;

            while let Some(factor) = factor_iter.next() {
                match factor {
                    Factor::Group(alternations)
                    | Factor::Repeat(alternations)
                    | Factor::Optional(alternations) => {
                        self.normal(left, alternations, nonterminal, cache);
                    }
                    Factor::Symbol(ident) => {
                        if ident == nonterminal {
                            success = true;
                            break;
                        }
                    }
                }
            }

            let mut set = IdentSet::new();

            if factor_iter.len() == 0 && success {
                cache.push_tail(left, nonterminal);
            }

            for factor in factor_iter {
                set = self.factor(factor, set);
            }

            cache.union(nonterminal, set);
        }
    }

    fn factor(&'a self, factor: &'a Factor, mut set: IdentSet<'a>) -> IdentSet<'a> {
        match factor {
            Factor::Group(alternations)
            | Factor::Repeat(alternations)
            | Factor::Optional(alternations) => {
                let orig_set = set.clone();

                for alternation in &alternations.alternations {
                    let mut alternation_set = orig_set.clone();

                    for factor in &alternation.factors {
                        alternation_set = self.factor(factor, alternation_set);
                    }

                    set.extend(alternation_set);
                }
                set
            }
            Factor::Symbol(ident) => self.symbol(ident, set),
        }
    }

    fn symbol(&'a self, ident: &'a Ident, mut set: IdentSet<'a>) -> IdentSet<'a> {
        if self.grammar.is_terminal(ident) {
            if set.is_empty() {
                set.insert(vec![ident]);
                set
            } else {
                set.into_iter()
                    .map(|mut item| {
                        item.push(ident);
                        item
                    })
                    .collect()
            }
        } else {
            let first = self.first.get(ident).unwrap();

            if set.is_empty() {
                first.clone()
            } else {
                set.into_iter()
                    .map(|item| {
                        first.iter().map(move |first_item| {
                            let mut item = item.clone();
                            item.extend(first_item);
                            item
                        })
                    })
                    .flatten()
                    .collect()
            }
        }
    }
}
