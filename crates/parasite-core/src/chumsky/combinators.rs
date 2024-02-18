use std::hash::Hash;

use super::Parseable;
use crate::combinators::{
    Any, DelimitedBy, End, Identifier, Just, NewLine, NonEmptyVec, PaddedBy, Rec, SeparatedBy,
    WhiteSpace,
};
use chumsky::{
    primitive::{any, just},
    text::{self, newline, whitespace, Character},
    Parser,
};

impl<'a, I, T> Parseable<'a, I> for Rec<T>
where
    I: Clone + Hash + Eq + 'a,
    T: Parseable<'a, I>,
{
    type Error = T::Error;

    fn parser() -> impl Parser<I, Self, Error = Self::Error> + Clone {
        T::parser().map(|t| Rec(Box::new(t)))
    }
}

impl<'a, I, T> Parseable<'a, I> for NonEmptyVec<T>
where
    I: Clone + Hash + Eq + 'a,
    T: Parseable<'a, I>,
{
    type Error = T::Error;

    fn parser() -> impl Parser<I, Self, Error = Self::Error> + Clone {
        T::parser()
            .repeated()
            .at_least(1)
            .collect()
            .map(NonEmptyVec)
    }
}

impl<'a, I, S, T> Parseable<'a, I> for SeparatedBy<S, T>
where
    I: Clone + Hash + Eq + 'a,
    T: Parseable<'a, I>,
    S: Parseable<'a, I, Error = T::Error>,
{
    type Error = T::Error;

    fn parser() -> impl Parser<I, Self, Error = Self::Error> + Clone {
        T::parser()
            .separated_by(S::parser())
            .at_least(1)
            .collect()
            .map(SeparatedBy::new)
    }
}

impl<'a, I, P, T> Parseable<'a, I> for PaddedBy<P, T>
where
    I: Clone + Hash + Eq + 'a,
    T: Parseable<'a, I>,
    P: Parseable<'a, I, Error = T::Error>,
{
    type Error = T::Error;

    fn parser() -> impl Parser<I, Self, Error = Self::Error> + Clone {
        T::parser().padded_by(P::parser()).map(PaddedBy::new)
    }
}

impl<'a, I, L, R, T> Parseable<'a, I> for DelimitedBy<L, R, T>
where
    I: Clone + Hash + Eq + 'a,
    T: Parseable<'a, I>,
    L: Parseable<'a, I, Error = T::Error> + Clone,
    R: Parseable<'a, I, Error = T::Error> + Clone,
{
    type Error = T::Error;

    fn parser() -> impl Parser<I, Self, Error = Self::Error> + Clone {
        T::parser()
            .delimited_by(L::parser(), R::parser())
            .map(DelimitedBy::new)
    }
}

impl<const CHAR: char> Parseable<'_, char> for Just<CHAR> {
    fn parser() -> impl Parser<char, Self, Error = Self::Error> + Clone {
        just(CHAR).to(Self())
    }
}

impl Parseable<'_, char> for End {
    fn parser() -> impl Parser<char, Self, Error = Self::Error> + Clone {
        chumsky::primitive::end().to(Self)
    }
}

impl Parseable<'_, char> for Any {
    fn parser() -> impl Parser<char, Self, Error = Self::Error> + Clone {
        any().map(Any)
    }
}

impl<'a, I: Character + Eq + Hash + 'a> Parseable<'a, I> for NewLine {
    fn parser() -> impl Parser<I, Self, Error = Self::Error> + Clone {
        newline().to(Self)
    }
}

impl Parseable<'_, char> for WhiteSpace {
    fn parser() -> impl Parser<char, Self, Error = Self::Error> + Clone {
        whitespace().to(WhiteSpace)
    }
}

impl Parseable<'_, char> for Identifier {
    fn parser() -> impl Parser<char, Self, Error = Self::Error> + Clone {
        text::ident().map(Identifier)
    }
}
