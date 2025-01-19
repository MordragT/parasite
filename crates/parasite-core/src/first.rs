use owo_colors::OwoColorize;

use crate::grammar::{Grammar, Id, Key, Symbol, Terminals};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt,
    ops::Index,
};

pub type FirstSets = HashMap<Id, FirstSet>;
pub type FirstSet = HashSet<Terminals>;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FirstTable(HashMap<Key, FirstSets>);

impl FirstTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &Key) -> Option<&FirstSets> {
        self.0.get(key)
    }

    pub fn get_mut(&mut self, key: &Key) -> Option<&mut FirstSets> {
        self.0.get_mut(key)
    }

    pub fn insert(&mut self, key: Key, sets: FirstSets) -> Option<FirstSets> {
        self.0.insert(key, sets)
    }
}

impl fmt::Display for FirstTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "First Sets\n".bold().fmt(f)?;

        for (key, sets) in &self.0 {
            writeln!(f, "{}", key.italic())?;

            for (id, set) in sets {
                for terminals in set {
                    write!(f, "\t{id}:")?;
                    for t in terminals {
                        write!(f, " {t}")?;
                    }
                    write!(f, "\n")?;
                }
            }
        }

        Ok(())
    }
}

impl Index<&Key> for FirstTable {
    type Output = FirstSets;

    fn index(&self, index: &Key) -> &Self::Output {
        &self.0[index]
    }
}

type FirstItem = (Id, (Terminals, VecDeque<Symbol>));

impl Grammar {
    fn first_k_of(
        &self,
        k: usize,
        of: Key,
        queue: &mut VecDeque<Key>,
        cache: &mut HashMap<Key, Vec<FirstItem>>,
    ) {
        if cache[&of]
            .iter()
            .all(|(_, (terminals, symbols))| terminals.len() >= k || symbols.is_empty())
        {
            return;
        }

        let mut items = Vec::new();

        for (id, (mut terminals, mut symbols)) in cache[&of].clone() {
            while let Some(symbol) = symbols.pop_front() {
                match symbol {
                    Symbol::Epsilon => (),
                    Symbol::Terminal(terminal) => terminals.push(terminal),
                    Symbol::Nonterminal(nonterminal) => {
                        let key = nonterminal.0;

                        let mut to_push = Vec::new();
                        for (_, (terms, syms)) in &cache[&key] {
                            let mut terminals = terminals.clone();
                            terminals.append(&mut terms.clone());

                            let mut syms = syms.clone();
                            syms.append(&mut symbols.clone());

                            to_push.push((id, (terminals, syms)));
                        }
                        if let Some((_, (terms, syms))) = to_push.pop() {
                            terminals = terms;
                            symbols = syms;
                        }

                        for item in to_push {
                            items.push(item);
                        }

                        queue.push_back(of.clone());
                        break;
                    }
                }

                if terminals.len() >= k {
                    terminals.truncate(k);
                    break;
                }
            }
            items.push((id, (terminals, symbols)));
        }

        cache.insert(of, items);
    }

    pub fn first_k(&self, k: usize) -> FirstTable {
        let mut queue = VecDeque::new();
        let mut cache = HashMap::new();
        let mut table = FirstTable::new();

        for (key, rule) in &self.productions {
            let mut sets = FirstSets::new();
            let mut items = Vec::new();

            for (id, symbols) in rule {
                sets.insert(*id, FirstSet::new());
                items.push((*id, (Vec::new(), VecDeque::from_iter(symbols.clone()))));
            }

            table.insert(key.clone(), sets);
            cache.insert(key.clone(), items);
            queue.push_back(key.clone());
        }

        while let Some(of) = queue.pop_front() {
            self.first_k_of(k, of, &mut queue, &mut cache);
        }

        for (key, items) in cache {
            for (id, (terminals, _)) in items {
                table.get_mut(&key).unwrap()[&id].insert(terminals);
            }
        }

        table
    }
}

#[cfg(test)]
mod test {

    use crate::{
        builder::Syntactical,
        grammar::{Grammar, Id, Key, Rule, Symbol, Terminal},
    };

    #[allow(dead_code)]
    enum A {
        Recurse((u8, Box<Self>)),
        End,
    }

    impl Syntactical for A {
        fn generate(grammar: &mut Grammar, stack: &mut Vec<Key>) -> Symbol {
            let key = Key::of::<Self>();
            let symbol = Symbol::nonterminal(key.clone());

            if !Self::visited(grammar, stack) {
                stack.push(key.clone());

                let mut rule = Rule::new();
                rule.insert(Id(0), vec![u8::generate(grammar, stack), symbol.clone()]);
                rule.insert(Id(1), vec![Symbol::Epsilon]);

                grammar.insert(key, rule);
            }

            symbol
        }
    }

    #[test]
    fn first_1() {
        let mut grammar = Grammar::new(Key::of::<A>());
        let mut stack = Vec::new();

        A::generate(&mut grammar, &mut stack);

        let first_table = grammar.first_k(1);
        let first_sets = &first_table[&Key::of::<A>()];

        let recurse = &first_sets[&Id(0)];
        assert_eq!(recurse.len(), 1);
        assert!(recurse.contains(&vec![Terminal::from(Key::of::<u8>())]));

        let end = &first_sets[&Id(1)];
        assert_eq!(end.len(), 1);
        assert!(end.contains(&Vec::new()));
    }

    #[test]
    fn first_2() {
        let mut grammar = Grammar::new(Key::of::<A>());
        let mut stack = Vec::new();

        A::generate(&mut grammar, &mut stack);

        let first_table = grammar.first_k(2);
        let first_sets = &first_table[&Key::of::<A>()];

        let recurse = &first_sets[&Id(0)];
        assert_eq!(recurse.len(), 2);
        assert!(recurse.contains(&vec![
            Terminal::from(Key::of::<u8>()),
            Terminal::from(Key::of::<u8>()),
        ]));
        assert!(recurse.contains(&vec![Terminal::from(Key::of::<u8>()),]));

        let end = &first_sets[&Id(1)];
        assert_eq!(end.len(), 1);
        assert!(end.contains(&Vec::new()));
    }

    #[test]
    fn first_3() {
        let mut grammar = Grammar::new(Key::of::<A>());
        let mut stack = Vec::new();

        A::generate(&mut grammar, &mut stack);

        let first_table = grammar.first_k(3);
        let first_sets = &first_table[&Key::of::<A>()];

        let recurse = &first_sets[&Id(0)];
        assert_eq!(recurse.len(), 3);
        assert!(recurse.contains(&vec![
            Terminal::from(Key::of::<u8>()),
            Terminal::from(Key::of::<u8>()),
            Terminal::from(Key::of::<u8>()),
        ]));
        assert!(recurse.contains(&vec![
            Terminal::from(Key::of::<u8>()),
            Terminal::from(Key::of::<u8>()),
        ]));
        assert!(recurse.contains(&vec![Terminal::from(Key::of::<u8>())]));

        let end = &first_sets[&Id(1)];
        assert_eq!(end.len(), 1);
        assert!(end.contains(&Vec::new()));
    }
}
