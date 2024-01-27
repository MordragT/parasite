use super::{Grammar, Id, Symbol, Terminal, TypeName};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Unexpected {terminal:?} while parsing, expected one of {expected:?}")]
    Unexpected {
        terminal: Terminal,
        expected: Vec<Vec<Terminal>>,
    },
}

impl Grammar {
    pub fn parse(
        &self,
        k: usize,
        terminals: &[Terminal],
    ) -> Result<Vec<(TypeName, Id)>, ParseError> {
        let table = self.table(k);

        let mut applied = Vec::new();

        let mut stack = vec![Symbol::nonterminal(self.start)];
        let mut cursor = 0;

        'outer: loop {
            let current = stack.pop().unwrap().into_nonterminal().unwrap().0;
            let look_ahead = &table[&current];

            let (peek, &id) = (1..=k)
                .into_iter()
                .find_map(|i| {
                    let peek = &terminals[cursor..(i + cursor)];
                    look_ahead.get_key_value(peek)
                })
                .ok_or(ParseError::Unexpected {
                    terminal: terminals[cursor],
                    expected: look_ahead.keys().cloned().collect(),
                })?;

            for &symbol in &self.productions[&current][&id] {
                stack.push(symbol);
            }

            while let Some(symbol) = stack.pop() {
                match symbol {
                    Symbol::Epsilon => (),
                    Symbol::Terminal(terminal) => {
                        assert_eq!(&terminal, &terminals[cursor]);
                        cursor += 1;
                    }
                    Symbol::Nonterminal(_) => {
                        stack.push(symbol);
                        applied.push((current, id));
                        break;
                    }
                }
            }

            if stack.is_empty() {
                applied.push((current, id));
                break;
            }
        }

        assert_eq!(cursor, terminals.len());

        Ok(applied)
    }
}

#[cfg(test)]
mod test {

    use crate::grammar::{builder::Syntactical, Grammar, Id, Rule, Symbol, Terminal, TypeName};

    enum S {
        A((u8, A, u8)),
    }

    enum A {
        S((bool, Box<S>, bool)),
        End,
    }

    impl Syntactical for S {
        fn generate(grammar: &mut Grammar, stack: &mut Vec<TypeName>) -> Symbol {
            let key = TypeName::of::<Self>();

            if !Self::visited(grammar, stack) {
                stack.push(key);

                let mut rule = Rule::new();
                rule.insert(
                    Id(0),
                    vec![
                        u8::generate(grammar, stack),
                        A::generate(grammar, stack),
                        u8::generate(grammar, stack),
                    ],
                );

                grammar.insert(key, rule);
            }

            Symbol::nonterminal(key)
        }
    }

    impl Syntactical for A {
        fn generate(grammar: &mut Grammar, stack: &mut Vec<TypeName>) -> Symbol {
            let key = TypeName::of::<Self>();

            if !Self::visited(grammar, stack) {
                stack.push(key);

                let mut rule = Rule::new();
                rule.insert(
                    Id(0),
                    vec![
                        bool::generate(grammar, stack),
                        S::generate(grammar, stack),
                        bool::generate(grammar, stack),
                    ],
                );
                rule.insert(Id(1), vec![Symbol::Epsilon]);

                grammar.insert(key, rule);
            }

            Symbol::nonterminal(key)
        }
    }

    #[test]
    fn parse_1() {
        let mut grammar = Grammar::new(TypeName::of::<S>());
        let mut stack = Vec::new();

        S::generate(&mut grammar, &mut stack);

        let uint = Terminal(TypeName::of::<u8>());
        let boolean = Terminal(TypeName::of::<bool>());

        let rules = grammar
            .parse(1, &[uint, boolean, uint, uint, boolean, uint])
            .unwrap();

        let a = TypeName::of::<A>();
        let s = TypeName::of::<S>();

        assert_eq!(rules, vec![(s, Id(0)), (a, Id(0)), (s, Id(0)), (a, Id(1))]);
    }

    #[test]
    fn parse_2() {
        let mut grammar = Grammar::new(TypeName::of::<S>());
        let mut stack = Vec::new();

        S::generate(&mut grammar, &mut stack);

        let uint = Terminal(TypeName::of::<u8>());
        let boolean = Terminal(TypeName::of::<bool>());

        let rules = grammar
            .parse(2, &[uint, boolean, uint, uint, boolean, uint])
            .unwrap();

        let a = TypeName::of::<A>();
        let s = TypeName::of::<S>();

        assert_eq!(rules, vec![(s, Id(0)), (a, Id(0)), (s, Id(0)), (a, Id(1))]);
    }

    #[test]
    fn parse_3() {
        let mut grammar = Grammar::new(TypeName::of::<S>());
        let mut stack = Vec::new();

        S::generate(&mut grammar, &mut stack);

        let uint = Terminal(TypeName::of::<u8>());
        let boolean = Terminal(TypeName::of::<bool>());

        let rules = grammar
            .parse(3, &[uint, boolean, uint, uint, boolean, uint])
            .unwrap();

        let a = TypeName::of::<A>();
        let s = TypeName::of::<S>();

        assert_eq!(rules, vec![(s, Id(0)), (a, Id(0)), (s, Id(0)), (a, Id(1))]);
    }
}
