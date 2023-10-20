use crate::expansion::{ExpandedGrammar, Token};
use proc_macro2::Ident;
use std::{
    collections::{HashMap, VecDeque},
    fmt,
};

#[derive(Debug, Clone, PartialEq)]
struct Item<'a> {
    tokens: VecDeque<&'a Token>,
    item: FirstItem<'a>,
}

type FirstItem<'a> = Vec<&'a Ident>;
type FirstSet<'a> = Vec<FirstItem<'a>>;

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

// TODO rules can be dependent on each other.
// Meaning that there needs to be some way to partially update one production and then check from another if the required k is already computed
impl<'a> FirstSets<'a> {
    pub fn build(grammar: &'a ExpandedGrammar) -> Self {
        // populate left terminals of productions
        let mut derivations = grammar
            .iter_productions()
            .map(|production| {
                production
                    .alternations()
                    .iter()
                    .map(|tokens| {
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

                        Item {
                            tokens,
                            item: terminals,
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
                    Some(Token::Nonterminal(other_id)) => {
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

        let sets = derivations
            .into_iter()
            .enumerate()
            .map(|(id, set)| (id, set.into_iter().map(|item| item.item).collect()))
            .collect();

        Self { sets }
    }
}
