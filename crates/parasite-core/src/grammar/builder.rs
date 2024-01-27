use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList, VecDeque};

use super::{table::Table, Grammar, Id, Rule, Symbol, TypeName};

pub trait SyntaxAnalyzer {
    type Ast: Syntactical;

    fn build(k: usize) -> Table {
        let mut grammar = Grammar::new(TypeName::of::<Self::Ast>());
        let mut stack = Vec::new();

        Self::Ast::generate(&mut grammar, &mut stack);

        grammar.table(k)
    }
}

pub trait Syntactical {
    fn generate(_grammar: &mut Grammar, _stack: &mut Vec<TypeName>) -> Symbol {
        Symbol::terminal(TypeName::of::<Self>())
    }

    fn visited(grammar: &Grammar, stack: &Vec<TypeName>) -> bool {
        let key = TypeName::of::<Self>();
        grammar.contains(&key) || stack.contains(&key)
    }
}

impl<T: Syntactical + 'static> Syntactical for Option<T> {
    fn generate(grammar: &mut Grammar, stack: &mut Vec<TypeName>) -> Symbol {
        let key = TypeName::of::<Self>();

        if !Self::visited(grammar, stack) {
            stack.push(key);

            let mut rule = Rule::new();
            rule.insert(Id(0), vec![T::generate(grammar, stack)]);
            rule.insert(Id(1), vec![Symbol::Epsilon]);

            grammar.insert(key, rule);
        }
        Symbol::nonterminal(key)
    }
}

impl<T: Syntactical + 'static, const N: usize> Syntactical for [T; N] {
    fn generate(grammar: &mut Grammar, stack: &mut Vec<TypeName>) -> Symbol {
        let key = TypeName::of::<Self>();

        if !Self::visited(grammar, stack) {
            stack.push(key);

            let child = T::generate(grammar, stack);
            let mut rule = Rule::new();
            rule.insert(Id(0), vec![child; N]);

            grammar.insert(key, rule);
        }

        Symbol::nonterminal(key)
    }
}

impl<T: Syntactical + 'static> Syntactical for Vec<T> {
    fn generate(grammar: &mut Grammar, stack: &mut Vec<TypeName>) -> Symbol {
        let key = TypeName::of::<Self>();
        let symbol = Symbol::nonterminal(key);

        if !Self::visited(grammar, stack) {
            stack.push(key);

            let mut rule = Rule::new();
            rule.insert(Id(0), vec![T::generate(grammar, stack), symbol]);
            rule.insert(Id(1), vec![Symbol::Epsilon]);

            grammar.insert(key, rule);
        }

        symbol
    }
}

impl<T: Syntactical + 'static> Syntactical for VecDeque<T> {
    fn generate(grammar: &mut Grammar, stack: &mut Vec<TypeName>) -> Symbol {
        let key = TypeName::of::<Self>();
        let symbol = Symbol::nonterminal(key);

        if !Self::visited(grammar, stack) {
            stack.push(key);

            let mut rule = Rule::new();
            rule.insert(Id(0), vec![T::generate(grammar, stack), symbol]);
            rule.insert(Id(1), vec![Symbol::Epsilon]);

            grammar.insert(key, rule);
        }

        symbol
    }
}

impl<T: Syntactical + 'static> Syntactical for LinkedList<T> {
    fn generate(grammar: &mut Grammar, stack: &mut Vec<TypeName>) -> Symbol {
        let key = TypeName::of::<Self>();
        let symbol = Symbol::nonterminal(key);

        if !Self::visited(grammar, stack) {
            stack.push(key);

            let mut rule = Rule::new();
            rule.insert(Id(0), vec![T::generate(grammar, stack), symbol]);
            rule.insert(Id(1), vec![Symbol::Epsilon]);

            grammar.insert(key, rule);
        }

        symbol
    }
}

impl<T: Syntactical + 'static> Syntactical for HashSet<T> {
    fn generate(grammar: &mut Grammar, stack: &mut Vec<TypeName>) -> Symbol {
        let key = TypeName::of::<Self>();
        let symbol = Symbol::nonterminal(key);

        if !Self::visited(grammar, stack) {
            stack.push(key);

            let mut rule = Rule::new();
            rule.insert(Id(0), vec![T::generate(grammar, stack), symbol]);
            rule.insert(Id(1), vec![Symbol::Epsilon]);

            grammar.insert(key, rule);
        }

        symbol
    }
}

impl<T: Syntactical + 'static> Syntactical for BTreeSet<T> {
    fn generate(grammar: &mut Grammar, stack: &mut Vec<TypeName>) -> Symbol {
        let key = TypeName::of::<Self>();
        let symbol = Symbol::nonterminal(key);

        if !Self::visited(grammar, stack) {
            stack.push(key);

            let mut rule = Rule::new();
            rule.insert(Id(0), vec![T::generate(grammar, stack), symbol]);
            rule.insert(Id(1), vec![Symbol::Epsilon]);

            grammar.insert(key, rule);
        }

        symbol
    }
}

impl<K: Syntactical + 'static, V: Syntactical + 'static> Syntactical for HashMap<K, V> {
    fn generate(grammar: &mut Grammar, stack: &mut Vec<TypeName>) -> Symbol {
        let key = TypeName::of::<Self>();
        let symbol = Symbol::nonterminal(key);

        if !Self::visited(grammar, stack) {
            stack.push(key);

            let mut rule = Rule::new();
            rule.insert(
                Id(0),
                vec![
                    K::generate(grammar, stack),
                    V::generate(grammar, stack),
                    symbol,
                ],
            );
            rule.insert(Id(1), vec![Symbol::Epsilon]);

            grammar.insert(key, rule);
        }

        symbol
    }
}

impl<K: Syntactical + 'static, V: Syntactical + 'static> Syntactical for BTreeMap<K, V> {
    fn generate(grammar: &mut Grammar, stack: &mut Vec<TypeName>) -> Symbol {
        let key = TypeName::of::<Self>();
        let symbol = Symbol::nonterminal(key);

        if !Self::visited(grammar, stack) {
            stack.push(key);

            let mut rule = Rule::new();
            rule.insert(
                Id(0),
                vec![
                    K::generate(grammar, stack),
                    V::generate(grammar, stack),
                    symbol,
                ],
            );
            rule.insert(Id(1), vec![Symbol::Epsilon]);

            grammar.insert(key, rule);
        }

        symbol
    }
}

impl Syntactical for () {}

impl<T, U> Syntactical for (T, U)
where
    T: Syntactical + 'static,
    U: Syntactical + 'static,
{
    fn generate(grammar: &mut Grammar, stack: &mut Vec<TypeName>) -> Symbol {
        let key = TypeName::of::<Self>();

        if !Self::visited(grammar, stack) {
            stack.push(key);

            let mut rule = Rule::new();
            rule.insert(
                Id(0),
                vec![T::generate(grammar, stack), U::generate(grammar, stack)],
            );

            grammar.insert(key, rule);
        }

        Symbol::nonterminal(key)
    }
}

impl<T, U, V> Syntactical for (T, U, V)
where
    T: Syntactical + 'static,
    U: Syntactical + 'static,
    V: Syntactical + 'static,
{
    fn generate(grammar: &mut Grammar, stack: &mut Vec<TypeName>) -> Symbol {
        let key = TypeName::of::<Self>();

        if !Self::visited(grammar, stack) {
            stack.push(key);

            let mut rule = Rule::new();
            rule.insert(
                Id(0),
                vec![
                    T::generate(grammar, stack),
                    U::generate(grammar, stack),
                    V::generate(grammar, stack),
                ],
            );

            grammar.insert(key, rule);
        }

        Symbol::nonterminal(key)
    }
}

impl<T, U, V, W> Syntactical for (T, U, V, W)
where
    T: Syntactical + 'static,
    U: Syntactical + 'static,
    V: Syntactical + 'static,
    W: Syntactical + 'static,
{
    fn generate(grammar: &mut Grammar, stack: &mut Vec<TypeName>) -> Symbol {
        let key = TypeName::of::<Self>();

        if !Self::visited(grammar, stack) {
            stack.push(key);

            let mut rule = Rule::new();
            rule.insert(
                Id(0),
                vec![
                    T::generate(grammar, stack),
                    U::generate(grammar, stack),
                    V::generate(grammar, stack),
                    W::generate(grammar, stack),
                ],
            );

            grammar.insert(key, rule);
        }

        Symbol::nonterminal(key)
    }
}

impl Syntactical for String {}

impl Syntactical for char {}

impl Syntactical for bool {}

impl Syntactical for u8 {}

impl Syntactical for u16 {}

impl Syntactical for u32 {}

impl Syntactical for u64 {}

impl Syntactical for u128 {}

impl Syntactical for usize {}

impl Syntactical for i8 {}

impl Syntactical for i16 {}

impl Syntactical for i32 {}

impl Syntactical for i64 {}

impl Syntactical for i128 {}

impl Syntactical for isize {}

impl Syntactical for f32 {}

impl Syntactical for f64 {}
