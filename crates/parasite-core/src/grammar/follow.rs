use std::{
    collections::{HashMap, VecDeque},
    ops::IndexMut,
};

use super::{first::FirstTable, Grammar, Terminal, TypeName};
use crate::grammar::Symbol;

impl Grammar {
    pub fn follow_sets(&self, first_sets: &FirstTable) -> FollowSets {
        let mut sets = FollowSets::from_iter(self.keys().map(|key| (key, FollowSet::new())));

        for (key, rule) in &self.productions {
            for (_, symbols) in rule {
                let mut invocation = None;
                let mut terminals = Vec::new();

                for symbol in symbols {
                    match symbol {
                        Symbol::Epsilon => todo!(),
                        Symbol::Nonterminal(nonterminal) => {
                            if let Some(invoked) = invocation.replace(nonterminal.0) {
                                let terminals = std::mem::replace(&mut terminals, Vec::new());
                                sets[&invoked].push(FollowItem::first(nonterminal.0, terminals));
                            }
                        }
                        Symbol::Terminal(terminal) => terminals.push(*terminal),
                    }
                }

                if let Some(invoked) = invocation {
                    if terminals.is_empty() && &invoked != key {
                        // invoked is last element and not recursive therefore the follow set of it must be added
                        // to the production symbol on the lhs of the production containing this symbol
                        // A -> B
                        // follow(A) += follow(B)
                        sets[&key].push(FollowItem::follow(invoked, Vec::new()));
                    } else {
                        sets[&invoked].push(FollowItem::new(terminals));
                    }
                }
            }
        }

        let mut queue = VecDeque::from_iter(self.keys());

        while let Some(key) = queue.pop_front() {
            for (pos, item) in sets[&key].clone().into_iter().enumerate() {
                match item.reference {
                    Reference::Follow(invoked) => {
                        let mut following = sets[&invoked].clone();
                        sets[&key].append(&mut following);
                        sets[&key].swap_remove(pos);
                        queue.push_back(key);
                    }
                    Reference::First(invoked) => {
                        for (_, first_set) in &first_sets[&invoked] {
                            for first_item in first_set {
                                let mut terminals = item.terminals.clone();
                                terminals.extend(first_item);
                                sets[&key].push(FollowItem::new(terminals));
                            }
                        }
                        sets[&key].swap_remove(pos);
                    }
                    Reference::None => (),
                }
            }
        }

        sets
    }
}

pub type FollowSets = HashMap<TypeName, FollowSet>;
pub type FollowSet = Vec<FollowItem>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FollowItem {
    reference: Reference,
    pub terminals: Vec<Terminal>,
}

impl FollowItem {
    pub fn new(terminals: Vec<Terminal>) -> Self {
        Self {
            reference: Reference::None,
            terminals,
        }
    }

    pub fn first(reference: TypeName, terminals: Vec<Terminal>) -> Self {
        Self {
            reference: Reference::First(reference),
            terminals,
        }
    }

    pub fn follow(reference: TypeName, terminals: Vec<Terminal>) -> Self {
        Self {
            reference: Reference::Follow(reference),
            terminals,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Reference {
    Follow(TypeName),
    First(TypeName),
    None,
}
