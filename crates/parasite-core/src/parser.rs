use crate::grammar::{Grammar, Id, Key, Symbol, Terminal};
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
    pub fn parse(&self, k: usize, terminals: &[Terminal]) -> Result<Vec<(Key, Id)>, ParseError> {
        let table = self.table(k);

        let mut applied = Vec::new();

        let start = Symbol::nonterminal(self.start.clone());
        let mut stack = vec![start];
        let mut cursor = 0;

        loop {
            let current = stack.pop().unwrap().into_nonterminal().unwrap().0;
            let look_ahead = &table[&current];

            let &id = (1..=k)
                .into_iter()
                .find_map(|i| {
                    let peek = &terminals[cursor..(i + cursor)];
                    look_ahead.get(peek)
                })
                .ok_or(ParseError::Unexpected {
                    terminal: terminals[cursor].clone(),
                    expected: look_ahead.keys().cloned().collect(),
                })?;

            for symbol in &self.productions[&current][&id] {
                stack.push(symbol.clone());
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
                        applied.push((current.clone(), id));
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

    use crate::{
        builder::Syntactical,
        grammar::{Grammar, Id, Key, Rule, Symbol, Terminal},
    };

    #[allow(dead_code)]
    enum S {
        A((u8, A, u8)),
    }

    #[allow(dead_code)]
    enum A {
        S((bool, Box<S>, bool)),
        End,
    }

    impl Syntactical for S {
        fn generate(grammar: &mut Grammar, stack: &mut Vec<Key>) -> Symbol {
            let key = Key::of::<Self>();

            if !Self::visited(grammar, stack) {
                stack.push(key.clone());

                let mut rule = Rule::new();
                rule.insert(
                    Id(0),
                    vec![
                        u8::generate(grammar, stack),
                        A::generate(grammar, stack),
                        u8::generate(grammar, stack),
                    ],
                );

                grammar.insert(key.clone(), rule);
            }

            Symbol::nonterminal(key)
        }
    }

    impl Syntactical for A {
        fn generate(grammar: &mut Grammar, stack: &mut Vec<Key>) -> Symbol {
            let key = Key::of::<Self>();

            if !Self::visited(grammar, stack) {
                stack.push(key.clone());

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

                grammar.insert(key.clone(), rule);
            }

            Symbol::nonterminal(key)
        }
    }

    #[test]
    fn parse_1() {
        let mut grammar = Grammar::new(Key::of::<S>());
        let mut stack = Vec::new();

        S::generate(&mut grammar, &mut stack);

        let uint = Terminal(Key::of::<u8>());
        let boolean = Terminal(Key::of::<bool>());

        let rules = grammar
            .parse(
                1,
                &[
                    uint.clone(),
                    boolean.clone(),
                    uint.clone(),
                    uint.clone(),
                    boolean,
                    uint,
                ],
            )
            .unwrap();

        let a = Key::of::<A>();
        let s = Key::of::<S>();

        assert_eq!(
            rules,
            vec![
                (s.clone(), Id(0)),
                (a.clone(), Id(0)),
                (s, Id(0)),
                (a, Id(1))
            ]
        );
    }

    #[test]
    fn parse_2() {
        let mut grammar = Grammar::new(Key::of::<S>());
        let mut stack = Vec::new();

        S::generate(&mut grammar, &mut stack);

        let uint = Terminal(Key::of::<u8>());
        let boolean = Terminal(Key::of::<bool>());

        let rules = grammar
            .parse(
                2,
                &[
                    uint.clone(),
                    boolean.clone(),
                    uint.clone(),
                    uint.clone(),
                    boolean,
                    uint,
                ],
            )
            .unwrap();

        let a = Key::of::<A>();
        let s = Key::of::<S>();

        assert_eq!(
            rules,
            vec![
                (s.clone(), Id(0)),
                (a.clone(), Id(0)),
                (s, Id(0)),
                (a, Id(1))
            ]
        );
    }

    #[test]
    fn parse_3() {
        let mut grammar = Grammar::new(Key::of::<S>());
        let mut stack = Vec::new();

        S::generate(&mut grammar, &mut stack);

        let uint = Terminal(Key::of::<u8>());
        let boolean = Terminal(Key::of::<bool>());

        let rules = grammar
            .parse(
                3,
                &[
                    uint.clone(),
                    boolean.clone(),
                    uint.clone(),
                    uint.clone(),
                    boolean,
                    uint,
                ],
            )
            .unwrap();

        let a = Key::of::<A>();
        let s = Key::of::<S>();

        assert_eq!(
            rules,
            vec![
                (s.clone(), Id(0)),
                (a.clone(), Id(0)),
                (s, Id(0)),
                (a, Id(1))
            ]
        );
    }
}
