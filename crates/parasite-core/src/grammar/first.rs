use super::{Grammar, Symbol, Terminal, TypeName};
use std::{
    collections::{HashMap, VecDeque},
    num::NonZeroUsize,
    ops::IndexMut,
};

impl Grammar {
    pub fn first_sets(&self, look_ahead: usize) -> FirstSets {
        let mut sets = FirstSets::new();

        for (key, rule) in &self.productions {
            let mut set = FirstSet::new();

            let mut alternations = rule
                .iter()
                .enumerate()
                .map(|(index, symbols)| {
                    let to_process = symbols.iter().take(look_ahead).cloned().collect::<Vec<_>>();
                    (index, to_process)
                })
                .collect::<Vec<_>>();

            alternations.dedup_by(|a, b| a.1.eq(&b.1));

            for (index, symbols) in alternations {
                let mut symbols = symbols.into_iter();

                let mut terminals = Vec::new();
                let mut pending = VecDeque::new();

                while let Some(symbol) = symbols.next() {
                    match symbol {
                        Symbol::Terminal(terminal) => terminals.push(terminal),
                        Symbol::Nonterminal(_) => {
                            pending.push_back(symbol);
                            break;
                        }
                    }
                }
                pending.extend(symbols);
                set.push(FirstItem {
                    index,
                    terminals,
                    pending,
                });
            }

            sets.insert(*key, set);
        }

        let mut queue = VecDeque::from_iter(self.keys());

        while let Some(key) = queue.pop_front() {
            let next_symbols = sets[&key]
                .iter_mut()
                .enumerate()
                .map(|(pos, item)| {
                    let symbol = item.pending.pop_front();
                    (pos, symbol)
                })
                .collect::<Vec<_>>();

            if next_symbols.iter().all(|(_, sym)| sym.is_none()) {
                continue;
            }

            for (pos, symbol) in next_symbols {
                match symbol {
                    Some(Symbol::Terminal(terminal)) => sets[&key][pos].terminals.push(terminal),
                    Some(Symbol::Nonterminal(nonterminal)) => {
                        let item = &sets[&key][pos];

                        let pending_count = match look_ahead
                            .checked_sub(item.count())
                            .and_then(NonZeroUsize::new)
                        {
                            Some(count) => count.get(),
                            None => continue,
                        };

                        let mut to_push = Vec::new();

                        for other_item in &sets[&nonterminal.0] {
                            let mut item = item.clone();
                            item.terminals
                                .extend(other_item.terminals.iter().take(pending_count));

                            if let Some(still_pending) = look_ahead
                                .checked_sub(item.count())
                                .and_then(NonZeroUsize::new)
                            {
                                for symbol in other_item.pending.iter().take(still_pending.get()) {
                                    item.pending.push_back(*symbol);
                                }
                                queue.push_back(key);
                            }
                            to_push.push(item);
                        }
                        to_push.dedup();

                        if let Some(item) = to_push.pop() {
                            sets[&key][pos] = item;
                        }

                        for item in to_push {
                            sets[&key].push(item);
                        }
                    }
                    None => continue,
                }
            }
        }

        sets
    }
}

pub type FirstSets = HashMap<TypeName, FirstSet>;

impl IndexMut<&TypeName> for FirstSets {
    fn index_mut(&mut self, index: &TypeName) -> &mut Self::Output {
        self.get_mut(&index).unwrap()
    }
}

pub type FirstSet = Vec<FirstItem>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FirstItem {
    pub(crate) index: usize,
    pub(crate) terminals: Vec<Terminal>,
    pending: VecDeque<Symbol>,
}

impl FirstItem {
    pub fn count(&self) -> usize {
        self.terminals.len()
    }

    pub fn is_epsilon(&self) -> bool {
        if let [terminal] = self.terminals.as_slice() {
            terminal.is_epsilon()
        } else {
            false
        }
    }
}
