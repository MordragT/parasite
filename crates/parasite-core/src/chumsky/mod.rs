pub use chumsky::*;

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList, VecDeque},
    fmt::Debug,
    hash::Hash,
};

mod combinators;

pub trait Parseable<'a, I: Clone + 'a>: Sized {
    type Error: chumsky::Error<I> = chumsky::error::Cheap<I>;

    fn parse() -> impl Parser<I, Self, Error = Self::Error>;
}

impl<'a, I, T> Parseable<'a, I> for Option<T>
where
    I: Clone + 'a,
    T: Parseable<'a, I>,
{
    type Error = T::Error;

    fn parse() -> impl Parser<I, Self, Error = Self::Error> {
        T::parse().or_not()
    }
}

impl<'a, I, T, const N: usize> Parseable<'a, I> for [T; N]
where
    I: Clone + 'a,
    T: Parseable<'a, I> + Debug,
{
    type Error = T::Error;

    fn parse() -> impl Parser<I, Self, Error = Self::Error> {
        T::parse()
            .repeated()
            .exactly(N)
            .collect::<Vec<_>>()
            .map(|vec| Self::try_from(vec).unwrap())
    }
}

impl<'a, I, T> Parseable<'a, I> for Vec<T>
where
    I: Clone + 'a,
    T: Parseable<'a, I>,
{
    type Error = T::Error;

    fn parse() -> impl Parser<I, Self, Error = Self::Error> {
        T::parse().repeated().collect()
    }
}

impl<'a, I, T> Parseable<'a, I> for VecDeque<T>
where
    I: Clone + 'a,
    T: Parseable<'a, I>,
{
    type Error = T::Error;
    fn parse() -> impl Parser<I, Self, Error = Self::Error> {
        T::parse().repeated().collect()
    }
}

impl<'a, I, T> Parseable<'a, I> for LinkedList<T>
where
    I: Clone + 'a,
    T: Parseable<'a, I>,
{
    type Error = T::Error;

    fn parse() -> impl Parser<I, Self, Error = Self::Error> {
        T::parse().repeated().collect()
    }
}

impl<'a, I, T> Parseable<'a, I> for HashSet<T>
where
    I: Clone + 'a,
    T: Parseable<'a, I> + Eq + Hash,
{
    type Error = T::Error;

    fn parse() -> impl Parser<I, Self, Error = Self::Error> {
        T::parse().repeated().collect()
    }
}

impl<'a, I, T> Parseable<'a, I> for BTreeSet<T>
where
    I: Clone + 'a,
    T: Parseable<'a, I> + Eq + Ord,
{
    type Error = T::Error;

    fn parse() -> impl Parser<I, Self, Error = Self::Error> {
        T::parse().repeated().collect()
    }
}

impl<'a, I, E, K, V> Parseable<'a, I> for HashMap<K, V>
where
    I: Clone + 'a,
    E: chumsky::Error<I>,
    K: Parseable<'a, I, Error = E> + Eq + Hash,
    V: Parseable<'a, I, Error = E>,
{
    type Error = E;

    fn parse() -> impl Parser<I, Self, Error = Self::Error> {
        K::parse().then(V::parse()).repeated().collect()
    }
}

impl<'a, I, E, K, V> Parseable<'a, I> for BTreeMap<K, V>
where
    I: Clone + 'a,
    E: chumsky::Error<I>,
    K: Parseable<'a, I, Error = E> + Eq + Ord,
    V: Parseable<'a, I, Error = E>,
{
    type Error = E;

    fn parse() -> impl Parser<I, Self, Error = Self::Error> {
        K::parse().then(V::parse()).repeated().collect()
    }
}
