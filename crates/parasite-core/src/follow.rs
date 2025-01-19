use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
};

use super::first::FirstTable;
use crate::grammar::{Grammar, Key, Symbol, Terminals};

pub type FollowSets = HashMap<Key, FollowSet>;
pub type FollowSet = HashSet<Terminals>;

impl Grammar {
    pub fn follow_k(&self, k: usize, first_table: &FirstTable) -> FollowSets {
        let mut sets: HashMap<Key, Vec<FollowItem>> =
            HashMap::from_iter(self.keys().map(|key| (key, Vec::new())));

        for (key, rule) in &self.productions {
            for (_, symbols) in rule {
                let mut invocation = None;
                let mut terminals = Vec::new();

                for symbol in symbols {
                    match symbol {
                        Symbol::Epsilon => (),
                        Symbol::Nonterminal(nonterminal) => {
                            if let Some(invoked) = invocation.replace(nonterminal.0.clone()) {
                                let mut terminals = std::mem::replace(&mut terminals, Vec::new());
                                terminals.truncate(k);
                                // sets.entry(invoked).and_modify(|entry| {
                                //     entry.push(FollowItem::first(nonterminal.0.clone(), terminals))
                                // });
                                sets.get_mut(&invoked)
                                    .unwrap()
                                    .push(FollowItem::first(nonterminal.0.clone(), terminals))
                            } else {
                                terminals.clear();
                            }
                        }
                        Symbol::Terminal(terminal) => terminals.push(terminal.clone()),
                    }
                }

                if let Some(invoked) = invocation {
                    if terminals.is_empty() && &invoked != key {
                        // invoked is last element and not recursive therefore the follow set of it must be added
                        // to the production symbol on the lhs of the production containing this symbol
                        // A -> B
                        // follow(A) += follow(B)
                        sets.get_mut(&key)
                            .unwrap()
                            .push(FollowItem::follow(invoked, terminals));
                    } else {
                        terminals.truncate(k);
                        sets.get_mut(&invoked)
                            .unwrap()
                            .push(FollowItem::new(terminals));
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
                        sets.get_mut(&key).unwrap().append(&mut following);
                        sets.get_mut(&key).unwrap().swap_remove(pos);
                        queue.push_back(key.clone());
                    }
                    Reference::First(invoked) => {
                        for (_, first_set) in &first_table[&invoked] {
                            for first_item in first_set {
                                let mut terminals = item.terminals.clone();
                                terminals.append(&mut first_item.clone());
                                sets.get_mut(&key).unwrap().push(FollowItem::new(terminals));
                            }
                        }
                        sets.get_mut(&key).unwrap().swap_remove(pos);
                    }
                    Reference::None => (),
                }
            }
        }

        sets.into_iter()
            .map(|(key, set)| {
                let set = set.into_iter().map(|item| item.terminals).collect();
                (key, set)
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct FollowItem {
    reference: Reference,
    terminals: Terminals,
}

impl FollowItem {
    fn new(terminals: Terminals) -> Self {
        Self {
            reference: Reference::None,
            terminals,
        }
    }

    fn first(reference: Key, terminals: Terminals) -> Self {
        Self {
            reference: Reference::First(reference),
            terminals,
        }
    }

    fn follow(reference: Key, terminals: Terminals) -> Self {
        Self {
            reference: Reference::Follow(reference),
            terminals,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Reference {
    Follow(Key),
    First(Key),
    None,
}

#[cfg(test)]
mod test {

    use crate::{
        builder::Syntactical,
        grammar::{Grammar, Id, Key, Rule, Symbol, Terminal},
    };

    #[allow(dead_code)]
    enum S {
        A((u8, A, u8)),
    }

    #[allow(dead_code)]
    enum A {
        S((bool, Box<S>, bool)),
        End,
    }

    impl Syntactical for S {
        fn generate(grammar: &mut Grammar, stack: &mut Vec<Key>) -> Symbol {
            let key = Key::of::<Self>();

            if !Self::visited(grammar, stack) {
                stack.push(key.clone());

                let mut rule = Rule::new();
                rule.insert(
                    Id(0),
                    vec![
                        u8::generate(grammar, stack),
                        A::generate(grammar, stack),
                        u8::generate(grammar, stack),
                    ],
                );

                grammar.insert(key.clone(), rule);
            }

            Symbol::nonterminal(key)
        }
    }

    impl Syntactical for A {
        fn generate(grammar: &mut Grammar, stack: &mut Vec<Key>) -> Symbol {
            let key = Key::of::<Self>();

            if !Self::visited(grammar, stack) {
                stack.push(key.clone());

                let mut rule = Rule::new();
                rule.insert(
                    Id(0),
                    vec![
                        bool::generate(grammar, stack),
                        S::generate(grammar, stack),
                        bool::generate(grammar, stack),
                    ],
                );
                rule.insert(Id(1), vec![Symbol::Epsilon]);

                grammar.insert(key.clone(), rule);
            }

            Symbol::nonterminal(key)
        }
    }

    #[test]
    fn follow_1() {
        let mut grammar = Grammar::new(Key::of::<S>());
        let mut stack = Vec::new();

        S::generate(&mut grammar, &mut stack);

        let k = 1;
        let first_table = grammar.first_k(k);
        let follow_sets = grammar.follow_k(k, &first_table);

        let a = &follow_sets[&Key::of::<A>()];
        assert_eq!(a.len(), 1);
        assert!(a.contains(&vec![Terminal::from(Key::of::<u8>())]));
    }

    #[test]
    fn follow_2() {
        let mut grammar = Grammar::new(Key::of::<S>());
        let mut stack = Vec::new();

        S::generate(&mut grammar, &mut stack);

        let k = 2;
        let first_table = grammar.first_k(k);
        let follow_sets = grammar.follow_k(k, &first_table);

        let a = &follow_sets[&Key::of::<A>()];
        assert_eq!(a.len(), 1);
        assert!(a.contains(&vec![Terminal::from(Key::of::<u8>())]));
    }

    #[test]
    fn follow_3() {
        let mut grammar = Grammar::new(Key::of::<S>());
        let mut stack = Vec::new();

        S::generate(&mut grammar, &mut stack);

        let k = 3;
        let first_table = grammar.first_k(k);
        let follow_sets = grammar.follow_k(k, &first_table);

        let a = &follow_sets[&Key::of::<A>()];
        assert_eq!(a.len(), 1);
        assert!(a.contains(&vec![Terminal::from(Key::of::<u8>())]));
    }
}
