use std::marker::PhantomData;

use crate::{
    builder::Syntactical,
    grammar::{Grammar, Id, Rule, Symbol, TypeName},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rec<T>(pub Box<T>);

impl<T: Syntactical + 'static> Syntactical for Rec<T> {
    fn generate(grammar: &mut Grammar, stack: &mut Vec<TypeName>) -> Symbol {
        let key = TypeName::of::<Self>();

        if !Self::visited(grammar, stack) {
            stack.push(key);

            let mut rule = Rule::new();
            rule.insert(Id(0), vec![T::generate(grammar, stack)]);

            grammar.insert(key, rule);
        }
        Symbol::nonterminal(key)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NonEmptyVec<T>(pub Vec<T>);

impl<T: Syntactical + 'static> Syntactical for NonEmptyVec<T> {
    fn generate(grammar: &mut Grammar, stack: &mut Vec<TypeName>) -> Symbol {
        let key = TypeName::of::<Self>();
        let symbol = Symbol::nonterminal(key);

        if !Self::visited(grammar, stack) {
            stack.push(key);

            let mut rule = Rule::new();
            rule.insert(
                Id(0),
                vec![
                    T::generate(grammar, stack),
                    Vec::<T>::generate(grammar, stack),
                ],
            );

            grammar.insert(key, rule);
        }

        symbol
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Just<const CHAR: char>();

impl<const CHAR: char> Syntactical for Just<CHAR> {
    fn generate(_grammar: &mut Grammar, _stack: &mut Vec<TypeName>) -> Symbol {
        Symbol::terminal(TypeName::of::<Self>())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SeparatedBy<S, T>(pub Vec<T>, PhantomData<S>);

impl<S, T> SeparatedBy<S, T> {
    pub fn new(values: Vec<T>) -> Self {
        Self(values, PhantomData)
    }
}

impl<T: Syntactical + 'static, S: Syntactical + 'static> Syntactical for SeparatedBy<S, T> {
    fn generate(grammar: &mut Grammar, stack: &mut Vec<TypeName>) -> Symbol {
        let key = TypeName::of::<Self>();
        let symbol = Symbol::nonterminal(key);

        if !Self::visited(grammar, stack) {
            stack.push(key);

            let mut rule = Rule::new();
            rule.insert(
                Id(0),
                vec![
                    T::generate(grammar, stack),
                    S::generate(grammar, stack),
                    symbol,
                ],
            );
            rule.insert(Id(1), vec![T::generate(grammar, stack)]);

            grammar.insert(key, rule);
        }

        symbol
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PaddedBy<P, T>(pub T, PhantomData<P>);

impl<P, T> PaddedBy<P, T> {
    pub fn new(value: T) -> Self {
        Self(value, PhantomData)
    }
}

impl<T: Syntactical + 'static, P: Syntactical + 'static> Syntactical for PaddedBy<P, T> {
    fn generate(grammar: &mut Grammar, stack: &mut Vec<TypeName>) -> Symbol {
        let key = TypeName::of::<Self>();

        if !Self::visited(grammar, stack) {
            stack.push(key);

            let pat = P::generate(grammar, stack);

            let mut rule = Rule::new();
            rule.insert(Id(0), vec![pat, T::generate(grammar, stack), pat]);

            grammar.insert(key, rule);
        }

        Symbol::nonterminal(key)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DelimitedBy<L, R, T>(pub T, PhantomData<L>, PhantomData<R>);

impl<L, R, T> DelimitedBy<L, R, T> {
    pub fn new(value: T) -> Self {
        Self(value, PhantomData, PhantomData)
    }
}

impl<T: Syntactical + 'static, L: Syntactical + 'static, R: Syntactical + 'static> Syntactical
    for DelimitedBy<L, R, T>
{
    fn generate(grammar: &mut Grammar, stack: &mut Vec<TypeName>) -> Symbol {
        let key = TypeName::of::<Self>();

        if !Self::visited(grammar, stack) {
            stack.push(key);

            let mut rule = Rule::new();
            rule.insert(
                Id(0),
                vec![
                    L::generate(grammar, stack),
                    T::generate(grammar, stack),
                    R::generate(grammar, stack),
                ],
            );

            grammar.insert(key, rule);
        }

        Symbol::nonterminal(key)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct End;

impl Syntactical for End {
    fn generate(_grammar: &mut Grammar, _stack: &mut Vec<TypeName>) -> Symbol {
        Symbol::Epsilon
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Any(pub char);

impl Syntactical for Any {
    fn generate(_grammar: &mut Grammar, _stack: &mut Vec<TypeName>) -> Symbol {
        Symbol::terminal(TypeName::of::<Self>())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NewLine;

impl Syntactical for NewLine {
    fn generate(_grammar: &mut Grammar, _stack: &mut Vec<TypeName>) -> Symbol {
        Symbol::terminal(TypeName::of::<Self>())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WhiteSpace;

impl Syntactical for WhiteSpace {
    fn generate(_grammar: &mut Grammar, _stack: &mut Vec<TypeName>) -> Symbol {
        Symbol::terminal(TypeName::of::<Self>())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Identifier(pub String);

impl Syntactical for Identifier {
    fn generate(_grammar: &mut Grammar, _stack: &mut Vec<TypeName>) -> Symbol {
        Symbol::terminal(TypeName::of::<Self>())
    }
}
