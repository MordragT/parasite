use super::Parseable;
use crate::combinators::{NonEmptyVec, Rec};
use chumsky::Parser;

impl<'a, I, T> Parseable<'a, I> for Rec<T>
where
    I: Clone + 'a,
    T: Parseable<'a, I>,
{
    type Error = T::Error;

    fn parse() -> impl Parser<I, Self, Error = Self::Error> {
        T::parse().map(|t| Rec(Box::new(t)))
    }
}

impl<'a, I, T> Parseable<'a, I> for NonEmptyVec<T>
where
    I: Clone + 'a,
    T: Parseable<'a, I>,
{
    type Error = T::Error;

    fn parse() -> impl Parser<I, Self, Error = Self::Error> {
        T::parse().repeated().at_least(1).collect().map(NonEmptyVec)
    }
}
