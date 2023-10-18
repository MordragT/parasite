use core::panic;
use proc_macro2::Ident;
use std::collections::{HashMap, VecDeque};

use crate::expansion::{ExpandedGrammar, Token};

#[derive(Debug)]
struct StackItem<'a> {
    tokens: VecDeque<&'a Token>,
    item: Vec<&'a Ident>,
    id: usize,
    pos: usize,
}

#[derive(Debug, Clone)]
pub struct FirstItem<'a> {
    item: Vec<&'a Ident>,
    finished: bool,
}

impl<'a> FirstItem<'a> {
    fn done(&self) -> usize {
        self.item.len()
    }
}

// one vec per alternation
type FirstSet<'a> = Vec<FirstItem<'a>>;

#[derive(Debug, Clone)]
pub struct FirstSets<'a> {
    sets: HashMap<usize, FirstSet<'a>>,
}

impl<'a> FirstSets<'a> {
    fn alloc(&mut self, id: usize) -> usize {
        let set = self.sets.get_mut(&id).unwrap();
        let pos = set.len();
        set.push(FirstItem {
            item: Vec::new(),
            finished: false,
        });
        pos
    }
}

// TODO rules can be dependent on each other.
// Meaning that there needs to be some way to partially update one production and then check from another if the required k is already computed
impl<'a> FirstSets<'a> {
    pub fn build(grammar: &'a ExpandedGrammar) -> Self {
        let mut sets = HashMap::new();
        let mut stack = VecDeque::new();

        // populate left terminals of productions
        for (id, production) in grammar.iter_productions().enumerate() {
            let mut set = Vec::new();

            for (pos, tokens) in production.alternations().iter().enumerate() {
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

                let finished = tokens.len() == 0;
                if !finished {
                    stack.push_back(StackItem {
                        tokens,
                        item: terminals.clone(),
                        id,
                        pos,
                    });
                }
                set.push(FirstItem {
                    item: terminals,
                    finished,
                });
            }

            sets.insert(id, set);
        }

        while let Some(StackItem {
            mut tokens,
            mut item,
            id,
            pos,
        }) = stack.pop_front()
        {
            // dbg!(&grammar);
            // dbg!(&sets);
            // dbg!(&stack);

            let mut alloc = Vec::new();

            while let Some(token) = tokens.pop_front() && item.len() < grammar.k() {
                let todo = grammar.k() - item.len();

                match token {
                    Token::Terminal(ident) => {
                        item.push(ident);
                    }
                    Token::Nonterminal(prod_id) => {
                        let set = sets[prod_id].as_slice();

                        if set.iter().all(|item| item.finished || item.done() >= todo) {
                            let mut set_items = set.into_iter();
                            let first_item = set_items.next();

                            while let Some(set_item) = set_items.next() {
                                let mut item = item.clone();
                                item.extend(set_item.item.iter().take(todo));

                                let finished = tokens.is_empty() || item.len() >= grammar.k();
                                alloc.push(FirstItem {
                                    item: item.clone(),
                                    finished,
                                });

                                let pos = sets[&id].len() + alloc.len();
                                if !finished {
                                    stack.push_front(StackItem {
                                        tokens: tokens.clone(),
                                        item,
                                        id,
                                        pos
                                    });
                                }

                            }
                            if let Some(set_item) = first_item {
                                dbg!("test");
                                item.extend(set_item.item.iter().take(todo));
                            }
                        } else {
                            // push current on stack, the unfinished should already be on the stack
                            tokens.push_front(token);
                            stack.push_back(StackItem {
                                tokens: tokens.clone(),
                                item: item.clone(),
                                id,
                                pos,
                            });
                            break;
                        }
                    }
                }

            }

            let finished = tokens.is_empty() || item.len() >= grammar.k();
            let set = sets.get_mut(&id).unwrap();
            set[pos] = FirstItem {
                item: item.clone(),
                finished,
            };
            for item in alloc {
                set.push(item)
            }

            if !finished {
                stack.push_front(StackItem {
                    tokens,
                    item,
                    id,
                    pos,
                });
            }
        }

        Self { sets }
    }
}
