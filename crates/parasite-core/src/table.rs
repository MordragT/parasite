use crate::grammar::{Grammar, Id, Terminals, TypeName};
use std::{collections::HashMap, hash::Hash};

impl<Key: Clone + Eq + Hash> Grammar<Key> {
    pub fn table(&self, k: usize) -> Table<Key> {
        let first_table = self.first_k(k);
        let follow_sets = self.follow_k(k, &first_table);

        // dbg!(&first_table);
        // dbg!(&follow_sets);

        let mut table = Table::new();

        for key in self.keys() {
            let mut row = Row::new();
            let first_sets = &first_table[&key];

            for (id, first_set) in first_sets {
                // let mut set = Set::new();

                for first_item in first_set {
                    if first_item.is_empty() {
                        for follow_item in &follow_sets[&key] {
                            row.insert(follow_item.clone(), *id);
                        }
                    } else {
                        row.insert(first_item.clone(), *id);
                    }
                }

                // row.insert(*id, set);
            }
            table.insert(key, row);
        }

        table
    }
}

pub type Table<Key = TypeName> = HashMap<Key, Row<Key>>;
pub type Row<Key = TypeName> = HashMap<Terminals<Key>, Id>;
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
