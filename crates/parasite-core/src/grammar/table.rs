use super::{Grammar, Id, Terminals, TypeName};
use std::collections::{HashMap, HashSet};

impl Grammar {
    pub fn table(&self, look_ahead: usize) -> Table {
        let first_table = self.first_k(look_ahead);
        dbg!(&first_table);

        let follow_sets = self.follow_sets(&first_table);
        dbg!(&follow_sets);

        let mut table = Table::new();

        for key in self.keys() {
            let mut row = Row::new();
            let first_sets = &first_table[&key];

            for (id, first_set) in first_sets {
                let mut set = Set::new();

                for first_item in first_set {
                    if first_item.is_empty() {
                        for follow in &follow_sets[&key] {
                            set.insert(follow.terminals.clone());
                        }
                    } else {
                        set.insert(first_item.clone());
                    }
                }

                row.insert(*id, set);
            }
            table.insert(key, row);
        }

        table
    }
}

pub type Table = HashMap<TypeName, Row>;
pub type Row = HashMap<Id, Set>;
pub type Set = HashSet<Terminals>;
