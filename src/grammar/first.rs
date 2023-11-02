use crate::grammar::{Grammar, Token};
use proc_macro2::Ident;
use std::{
    collections::{HashMap, VecDeque},
    fmt,
};

type FirstItem<'a> = Vec<&'a Ident>;
type FirstSet<'a> = Vec<FirstUnit<'a>>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Derivation<'a> {
    tokens: VecDeque<&'a Token>,
    item: FirstItem<'a>,
    alternation: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FirstUnit<'a> {
    pub(crate) item: FirstItem<'a>,
    pub(crate) alternation: usize,
}

impl<'a> From<Derivation<'a>> for FirstUnit<'a> {
    fn from(derivation: Derivation<'a>) -> Self {
        Self {
            item: derivation.item,
            alternation: derivation.alternation,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FirstSets<'a> {
    pub(crate) sets: HashMap<usize, FirstSet<'a>>,
}

impl<'a> fmt::Display for FirstSets<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "First Sets\n")?;
        write!(f, "==============\n")?;

        for (id, set) in &self.sets {
            let mut output = format!("{id}\t: ");
            for unit in set {
                for ident in &unit.item {
                    output.push_str(&ident.to_string());
                    output.push(' ');
                }
                output.push_str(&format!("({})\n\t, ", unit.alternation));
            }
            output.pop();
            output.pop();
            write!(f, "{output}\n")?;
        }
        Ok(())
    }
}

// TODO rules can be dependent on each other.
// Meaning that there needs to be some way to partially update one production and then check from another if the required k is already computed
impl<'a> FirstSets<'a> {
    pub fn build(grammar: &'a Grammar) -> Self {
        // populate left terminals of productions
        let mut derivations = grammar
            .productions
            .iter()
            .map(|production| {
                production
                    .alternations()
                    .iter()
                    .enumerate()
                    .map(|(alternation, tokens)| {
                        let mut to_process = tokens.into_iter().take(grammar.k());

                        let mut terminals = Vec::new();
                        let mut tokens = VecDeque::new();

                        while let Some(token) = to_process.next() {
                            match token {
                                Token::Terminal(ident) => terminals.push(ident),
                                _ => {
                                    tokens.push_back(token);
                                    break;
                                }
                            }
                        }

                        tokens.extend(to_process);

                        Derivation {
                            tokens,
                            item: terminals,
                            alternation,
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let mut queue = (0..derivations.len()).collect::<VecDeque<_>>();

        while let Some(id) = queue.pop_front() {
            let to_process = derivations[id]
                .iter_mut()
                .map(|item| item.tokens.pop_front())
                .collect::<Vec<_>>();

            if to_process.iter().all(|token| token.is_none()) {
                continue;
            }

            for (item_id, token) in to_process.into_iter().enumerate() {
                match token {
                    Some(Token::Terminal(terminal)) => derivations[id][item_id].item.push(terminal),
                    Some(Token::Derived(other_id)) => {
                        let todo =
                            match grammar.k().checked_sub(derivations[id][item_id].item.len()) {
                                Some(t) => t,
                                None => continue,
                            };

                        let mut to_push = Vec::new();
                        for other_item in &derivations[*other_id] {
                            let mut item = derivations[id][item_id].clone();
                            item.item.extend(other_item.item.iter().take(todo));

                            if let Some(todo) = grammar.k().checked_sub(item.item.len()) {
                                for token in other_item.tokens.iter().take(todo).rev() {
                                    item.tokens.push_front(token);
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

        // TODO remove unnecessary first set items by checking what amount of k items is necessary to uphold:
        // 1. every unit of the same alternation has minimal items while still including differing starting items for the parsing table
        // 2. every unit of different alternations have enough items to differentiate their units

        // for set in derivations {
        //     let groups = set
        //         .group_by(|a, b| a.alternation == b.alternation)
        //         .collect::<Vec<_>>();

        //     for l in 1..grammar.k() {
        //         for a in &groups {
        //             for b in &groups {
        //                 if a != b {
        //                     if a.iter()
        //                         .map(|derivation| derivation.item.iter().take(l))
        //                         .all(|item| {
        //                             b.iter()
        //                                 .map(|derivation| derivation.item.iter().take(l))
        //                                 .all(|other| item != other)
        //                         })
        //                     {
        //                         todo!()
        //                     }
        //                 }
        //             }
        //         }
        //     }
        // }

        // Current approach is very complex and not very efficient maybe it it is acceptable to create more items than necessary for k > 1
        // Only need to keep that in mind for parsing table creation and follow sets

        let sets = derivations
            .into_iter()
            .enumerate()
            .map(|(id, set)| (id, set.into_iter().map(Derivation::into).collect()))
            .collect();

        Self { sets }
    }
}
