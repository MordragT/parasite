use crate::expansion::{ExpandedGrammar, Token};
use proc_macro2::Ident;
use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
    fmt,
};

use super::first::FirstSets;

#[derive(Debug, Clone, PartialEq)]
struct Item<'a> {
    units: VecDeque<Unit<'a>>,
    item: FollowItem<'a>,
}

#[derive(Debug, Clone, PartialEq)]
enum Unit<'a> {
    First(&'a Token),
    Follow(usize),
}

type FollowItem<'a> = Vec<&'a Ident>;
type FollowSet<'a> = Vec<FollowItem<'a>>;

#[derive(Debug, Clone)]
pub struct FollowSets<'a> {
    pub(crate) sets: HashMap<usize, FollowSet<'a>>,
}

impl<'a> fmt::Display for FollowSets<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Follow Sets\n")?;
        write!(f, "==============\n")?;

        for (id, set) in &self.sets {
            let mut output = format!("{id}\t: ");
            for item in set {
                for ident in item {
                    output.push_str(&ident.to_string());
                    output.push(' ');
                }
                output.pop();
                output.push_str("\n\t, ");
            }
            output.pop();
            output.pop();
            write!(f, "{output}\n")?;
        }
        Ok(())
    }
}

impl<'a> FollowSets<'a> {
    pub fn build(grammar: &'a ExpandedGrammar, first_sets: &FirstSets<'a>) -> Self {
        let mut derivations = vec![vec![]; first_sets.sets.len()];

        // TODO remove iter_productions in favor of simple productions getter
        for production in grammar.iter_productions() {
            for tokens in production.alternations().iter() {
                let mut to_process = tokens.into_iter();

                while let Some(token) = to_process.next() {
                    if let Token::Nonterminal(id) = token {
                        let units = if to_process.is_empty() {
                            let mut units = VecDeque::new();
                            units.push_back(Unit::Follow(production.id));
                            units
                        } else {
                            to_process
                                .clone()
                                .take(grammar.k())
                                .map(Unit::First)
                                .collect()
                        };
                        let item = Item {
                            units,
                            item: Vec::new(),
                        };
                        derivations[*id].push(item);
                    }
                }
            }
        }

        let mut queue = (0..derivations.len()).collect::<VecDeque<_>>();

        while let Some(id) = queue.pop_front() {
            let to_process = derivations[id]
                .iter_mut()
                .map(|item| item.units.pop_front())
                .collect::<Vec<_>>();

            if to_process.iter().all(|token| token.is_none()) {
                continue;
            }

            for (item_id, unit) in to_process.into_iter().enumerate() {
                let todo = match grammar.k().checked_sub(derivations[id][item_id].item.len()) {
                    Some(t) => t,
                    None => continue,
                };

                match unit {
                    Some(Unit::First(Token::Terminal(terminal))) => {
                        derivations[id][item_id].item.push(terminal)
                    }
                    Some(Unit::First(Token::Nonterminal(other_id))) => {
                        let mut to_push = Vec::new();
                        for other_item in &first_sets.sets[other_id] {
                            let mut item = derivations[id][item_id].clone();
                            item.item.extend(other_item.iter().take(todo));
                            to_push.push(item);
                        }
                        to_push.dedup_by(|a, b| a.eq(&b));

                        if let Some(item) = to_push.pop() {
                            derivations[id][item_id] = item;
                        }

                        for item in to_push {
                            derivations[id].push(item);
                        }
                    }
                    Some(Unit::Follow(other_id)) => {
                        let mut to_push = Vec::new();
                        for other_item in &derivations[other_id] {
                            let mut item = derivations[id][item_id].clone();
                            item.item.extend(other_item.item.iter().take(todo));

                            if let Some(todo) = grammar.k().checked_sub(item.item.len()) {
                                for unit in other_item.units.iter().take(todo).rev() {
                                    item.units.push_front(unit.clone());
                                }
                                queue.push_back(id);
                            }
                            to_push.push(item);
                        }
                        to_push.dedup_by(|a, b| a.eq(&b));

                        if let Some(item) = to_push.pop() {
                            derivations[id][item_id] = item;
                        }

                        for item in to_push {
                            derivations[id].push(item);
                        }
                    }
                    None => continue,
                }
            }
        }

        let sets = derivations
            .into_iter()
            .enumerate()
            .map(|(id, set)| (id, set.into_iter().map(|item| item.item).collect()))
            .collect();

        Self { sets }
    }
}
