use owo_colors::OwoColorize;

use crate::grammar::{Grammar, Id, Key, Terminals};
use core::fmt;
use std::{collections::HashMap, ops::Index};

impl Grammar {
    pub fn table(&self, k: usize) -> Table {
        let first_table = self.first_k(k);
        println!("{first_table}");
        let follow_sets = self.follow_k(k, &first_table);

        let mut table = Table::new();

        for key in self.keys() {
            let mut row = Row::new();
            let first_sets = &first_table[&key];

            for (id, first_set) in first_sets {
                for first_item in first_set {
                    if first_item.is_empty() {
                        for follow_item in &follow_sets[&key] {
                            row.insert(follow_item.clone(), *id);
                        }
                    } else {
                        row.insert(first_item.clone(), *id);
                    }
                }
            }
            table.insert(key, row);
        }

        table
    }
}

pub type Row = HashMap<Terminals, Id>;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Table(HashMap<Key, Row>);

impl Table {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: Key, row: Row) -> Option<Row> {
        self.0.insert(key, row)
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", "LL-k Table".bold())?;

        for (key, row) in &self.0 {
            writeln!(f, "{}", key.italic())?;

            for (terminals, id) in row {
                write!(f, "\t{id}:")?;
                for t in terminals {
                    write!(f, " {t}")?;
                }
                write!(f, "\n")?;
            }
        }

        Ok(())
    }
}

impl Index<&Key> for Table {
    type Output = Row;

    fn index(&self, index: &Key) -> &Self::Output {
        &self.0[index]
    }
}

// pub type Set = HashSet<Terminals>;

// #[cfg(test)]
// mod test {

//     use crate::grammar::{builder::Syntactical, Grammar, Id, Rule, Symbol, Terminal, TypeName};

//     enum S {
//         A((u8, A, u8)),
//     }

//     enum A {
//         S((bool, Box<S>, bool)),
//         End,
//     }

//     impl Syntactical for S {
//         fn generate(grammar: &mut Grammar, stack: &mut Vec<TypeName>) -> Symbol {
//             let key = TypeName::of::<Self>();

//             if !Self::visited(grammar, stack) {
//                 stack.push(key);

//                 let mut rule = Rule::new();
//                 rule.insert(
//                     Id(0),
//                     vec![
//                         u8::generate(grammar, stack),
//                         A::generate(grammar, stack),
//                         u8::generate(grammar, stack),
//                     ],
//                 );

//                 grammar.insert(key, rule);
//             }

//             Symbol::nonterminal(key)
//         }
//     }

//     impl Syntactical for A {
//         fn generate(grammar: &mut Grammar, stack: &mut Vec<TypeName>) -> Symbol {
//             let key = TypeName::of::<Self>();

//             if !Self::visited(grammar, stack) {
//                 stack.push(key);

//                 let mut rule = Rule::new();
//                 rule.insert(
//                     Id(0),
//                     vec![
//                         bool::generate(grammar, stack),
//                         S::generate(grammar, stack),
//                         bool::generate(grammar, stack),
//                     ],
//                 );
//                 rule.insert(Id(1), vec![Symbol::Epsilon]);

//                 grammar.insert(key, rule);
//             }

//             Symbol::nonterminal(key)
//         }
//     }

//     #[test]
//     fn table_1() {
//         let mut grammar = Grammar::new(TypeName::of::<S>());
//         let mut stack = Vec::new();

//         S::generate(&mut grammar, &mut stack);

//         let table = grammar.table(1);
//         dbg!(&table);
//         panic!();
//     }

//     #[test]
//     fn table_2() {
//         let mut grammar = Grammar::new(TypeName::of::<S>());
//         let mut stack = Vec::new();

//         S::generate(&mut grammar, &mut stack);

//         let table = grammar.table(2);
//         dbg!(&table);
//         panic!();
//     }

//     #[test]
//     fn table_3() {
//         let mut grammar = Grammar::new(TypeName::of::<S>());
//         let mut stack = Vec::new();

//         S::generate(&mut grammar, &mut stack);

//         let table = grammar.table(3);
//         dbg!(&table);
//         panic!();
//     }
// }
