use super::{Grammar, Terminal, TypeName};
use std::collections::{HashMap, HashSet};

impl Grammar {
    pub fn table(&self, look_ahead: usize) -> Table {
        let first_sets = self.first_sets(look_ahead);
        dbg!(&first_sets);

        let follow_sets = self.follow_sets(&first_sets);
        dbg!(&follow_sets);

        let mut table = Table::new();

        for key in self.keys() {
            let mut row = Row::new();
            let first_set = &first_sets[&key];

            for first in first_set {
                if first.terminals.is_empty() {
                    panic!("Expect atleast one episilon element inside first item");
                }
                if first.is_epsilon() {
                    for follow in &follow_sets[&key] {
                        let element = Element::new(first.index, follow.terminals.clone());
                        row.insert(element);
                    }
                } else {
                    let element = Element::new(first.index, first.terminals.clone());
                    row.insert(element);
                }
            }

            table.insert(key, row);
        }

        table
    }
}

pub type Table = HashMap<TypeName, Row>;
pub type Row = HashSet<Element>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Element {
    index: usize,
    terminals: Vec<Terminal>,
}

impl Element {
    pub fn new(index: usize, terminals: Vec<Terminal>) -> Self {
        Self { index, terminals }
    }
}
