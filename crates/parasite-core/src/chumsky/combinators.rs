use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList, VecDeque},
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
};

use chumsky::Parser;

impl<I: Clone, T: Parseable<I>> Parseable<I> for Rec<T> {
    type Error<'a> = T::Error<'a>;

    fn parse<'a>() -> impl Parser<I, Self, Error = Self::Error<'a>> {
        T::parse().map(|t| Rec(Box::new(t)))
    }
}

impl<I: Clone, T: Parseable<I>> Parseable<I> for NonEmptyVec<T> {
    type Error<'a> = T::Error<'a>;

    fn parse<'a>() -> impl Parser<I, Self, Error = Self::Error<'a>> {
        T::parse().repeated().at_least(1).collect().map(NonEmptyVec)
    }
}
