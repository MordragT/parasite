use super::first::FirstSets;
use crate::grammar::{Grammar, Token};
use proc_macro2::Ident;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt,
};

type FollowItem<'a> = Vec<&'a Ident>;
type FollowSet<'a> = HashSet<FollowItem<'a>>;

#[derive(Debug, Clone)]
pub struct FollowSets<'a> {
    pub(crate) sets: HashMap<usize, FollowSet<'a>>,
}
impl<'a> FollowSets<'a> {
    // similar to the earley algo
    pub fn build(grammar: &'a Grammar, first_sets: &FirstSets<'a>) -> Self {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
        struct Derivation<'a> {
            // symbols: VecDeque<Symbol<'a>>,
            /// A follow item containing a list of terminals
            item: FollowItem<'a>,
            /// An referenced production at the end of the derivation
            reference: Option<Symbol>,
        }

        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy)]
        enum Symbol {
            First(usize),
            Follow(usize),
        }

        let mut chart = vec![vec![]; grammar.productions_count()];

        for (pid, production) in grammar.productions.iter().enumerate() {
            for tokens in production.alternations().iter() {
                let mut current_item = Vec::new();
                let mut current: Option<usize> = None;

                for token in tokens {
                    match token {
                        Token::Derived(rhs_pid) => {
                            let item = std::mem::replace(&mut current_item, Vec::new());
                            if let Some(current_pid) = current {
                                chart[current_pid].push(Derivation {
                                    item,
                                    reference: Some(Symbol::First(*rhs_pid)),
                                });
                            }
                            current = Some(*rhs_pid);
                        }
                        Token::Terminal(terminal) => current_item.push(terminal),
                    }
                }

                if let Some(current_pid) = current {
                    if current_item.is_empty() && pid != current_pid {
                        // rhs_pid is the last symbol therefore the follow set of it must be added
                        // to the production symbol on the lhs of the production containing this symbol
                        // A -> B
                        // follow(A) += follow(B)
                        chart[current_pid].push(Derivation {
                            item: Vec::new(),
                            reference: Some(Symbol::Follow(pid)),
                        });
                    } else {
                        chart[current_pid].push(Derivation {
                            item: current_item,
                            reference: None,
                        });
                    }
                }
            }
        }

        let mut queue = VecDeque::from_iter(0..grammar.productions_count());

        while let Some(pid) = queue.pop_front() {
            for (id, derivation) in chart[pid].clone().into_iter().enumerate() {
                match derivation.reference {
                    Some(Symbol::Follow(lhs_pid)) => {
                        let mut set = chart[lhs_pid].clone();
                        chart[pid].append(&mut set);
                        chart[pid].swap_remove(id);
                        queue.push_back(pid);
                    }
                    Some(Symbol::First(end_pid)) => {
                        for first_unit in &first_sets.sets[&end_pid] {
                            let mut item = derivation.item.clone();
                            item.extend(&first_unit.item);
                            chart[pid].push(Derivation {
                                item,
                                reference: None,
                            });
                        }
                        chart[pid].swap_remove(id);
                    }
                    None => (),
                }
            }
        }

        let sets = chart
            .into_iter()
            .enumerate()
            .map(|(id, set)| {
                (
                    id,
                    set.into_iter().map(|derivation| derivation.item).collect(),
                )
            })
            .collect();

        Self { sets }
    }
}

impl<'a> fmt::Display for FollowSets<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Follow Sets\n")?;
        write!(f, "==============\n")?;

        for (id, set) in &self.sets {
            let mut output = format!("{id}\t: ");
            for unit in set {
                for ident in unit {
                    output.push_str(&ident.to_string());
                    output.push(' ');
                }
                output.push_str(&format!("\n\t, "));
            }
            output.pop();
            output.pop();
            write!(f, "{output}\n")?;
        }
        Ok(())
    }
}
