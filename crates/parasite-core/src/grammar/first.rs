use super::{Grammar, Id, Symbol, Terminals, TypeName};
use std::collections::{HashMap, HashSet, VecDeque};

pub type FirstTable = HashMap<TypeName, FirstSets>;
pub type FirstSets = HashMap<Id, FirstSet>;
pub type FirstSet = HashSet<Terminals>;

type FirstItem = (Id, (Terminals, VecDeque<Symbol>));

impl Grammar {
    fn first_k_of(
        &self,
        k: usize,
        of: TypeName,
        queue: &mut VecDeque<TypeName>,
        cache: &mut HashMap<TypeName, Vec<FirstItem>>,
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
                            terminals.extend(terms);

                            let mut symbols = symbols.clone();
                            symbols.extend(syms);

                            to_push.push((id, (terminals, symbols)));
                        }
                        if let Some((_, (terms, syms))) = to_push.pop() {
                            terminals = terms;
                            symbols = syms;
                        }

                        for item in to_push {
                            items.push(item);
                        }

                        queue.push_back(of);
                        break;
                    }
                }

                if terminals.len() == k {
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

            table.insert(*key, sets);
            cache.insert(*key, items);
            queue.push_back(*key);
        }

        while let Some(of) = queue.pop_front() {
            self.first_k_of(k, of, &mut queue, &mut cache);
        }

        for (key, items) in cache {
            for (id, (terminals, _)) in items {
                table[&key][&id].insert(terminals);
            }
        }

        table
    }
}

#[cfg(test)]
mod test {

    use crate::grammar::{builder::Syntactical, Grammar, Id, Rule, Symbol, Terminal, TypeName};

    enum A {
        Recurse((u8, Box<Self>)),
        End,
    }

    impl Syntactical for A {
        fn generate(grammar: &mut crate::grammar::Grammar) -> crate::grammar::Symbol {
            let key = TypeName::of::<Self>();
            let symbol = Symbol::nonterminal(key);

            if !grammar.contains(&key) {
                let mut rule = Rule::new();
                rule.insert(Id(0), vec![u8::generate(grammar), symbol]);
                rule.insert(Id(1), vec![Symbol::terminal(TypeName("End"))]);

                grammar.insert(key, rule);
            }

            symbol
        }
    }

    #[test]
    fn first_1_of_a() {
        let mut grammar = Grammar::new(TypeName::of::<A>());

        A::generate(&mut grammar);

        let first_table = grammar.first_k(1);
        let first_sets = &first_table[&TypeName::of::<A>()];

        let recurse = &first_sets[&Id(0)];
        assert_eq!(recurse.len(), 1);
        assert!(recurse.contains(&vec![Terminal::from(TypeName::of::<u8>())]));

        let end = &first_sets[&Id(1)];
        assert_eq!(end.len(), 1);
        assert!(end.contains(&vec![Terminal::from(TypeName("End"))]));
    }

    #[test]
    fn first_2_of_a() {
        let mut grammar = Grammar::new(TypeName::of::<A>());

        A::generate(&mut grammar);

        let first_table = grammar.first_k(2);
        let first_sets = &first_table[&TypeName::of::<A>()];

        let recurse = &first_sets[&Id(0)];
        assert_eq!(recurse.len(), 2);
        assert!(recurse.contains(&vec![
            Terminal::from(TypeName::of::<u8>()),
            Terminal::from(TypeName::of::<u8>()),
        ]));
        assert!(recurse.contains(&vec![
            Terminal::from(TypeName::of::<u8>()),
            Terminal::from(TypeName("End")),
        ]));

        let end = &first_sets[&Id(1)];
        assert_eq!(end.len(), 1);
        assert!(end.contains(&vec![Terminal::from(TypeName("End"))]));
    }

    #[test]
    fn first_3_of_a() {
        let mut grammar = Grammar::new(TypeName::of::<A>());

        A::generate(&mut grammar);

        let first_table = grammar.first_k(3);
        let first_sets = &first_table[&TypeName::of::<A>()];

        let recurse = &first_sets[&Id(0)];
        assert_eq!(recurse.len(), 3);
        assert!(recurse.contains(&vec![
            Terminal::from(TypeName::of::<u8>()),
            Terminal::from(TypeName::of::<u8>()),
            Terminal::from(TypeName::of::<u8>()),
        ]));
        assert!(recurse.contains(&vec![
            Terminal::from(TypeName::of::<u8>()),
            Terminal::from(TypeName::of::<u8>()),
            Terminal::from(TypeName("End")),
        ]));
        assert!(recurse.contains(&vec![
            Terminal::from(TypeName::of::<u8>()),
            Terminal::from(TypeName("End")),
        ]));

        let end = &first_sets[&Id(1)];
        assert_eq!(end.len(), 1);
        assert!(end.contains(&vec![Terminal::from(TypeName("End"))]));
    }
}
