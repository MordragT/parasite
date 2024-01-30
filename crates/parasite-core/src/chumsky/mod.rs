pub use chumsky::*;

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList, VecDeque},
    fmt::Debug,
    hash::Hash,
};

mod combinators;

pub trait Parseable<'a, I: Hash + Eq + Clone + 'a>: Sized {
    type Error: chumsky::Error<I> = chumsky::error::Simple<I>;

    fn parser() -> impl Parser<I, Self, Error = Self::Error>;
}

impl<'a, I, T> Parseable<'a, I> for Option<T>
where
    I: Clone + Hash + Eq + 'a,
    T: Parseable<'a, I>,
{
    type Error = T::Error;

    fn parser() -> impl Parser<I, Self, Error = Self::Error> {
        T::parser().or_not()
    }
}

impl<'a, I, T, const N: usize> Parseable<'a, I> for [T; N]
where
    I: Clone + Hash + Eq + 'a,
    T: Parseable<'a, I> + Debug,
{
    type Error = T::Error;

    fn parser() -> impl Parser<I, Self, Error = Self::Error> {
        T::parser()
            .repeated()
            .exactly(N)
            .collect::<Vec<_>>()
            .map(|vec| Self::try_from(vec).unwrap())
    }
}

impl<'a, I, T> Parseable<'a, I> for Vec<T>
where
    I: Clone + Hash + Eq + 'a,
    T: Parseable<'a, I>,
{
    type Error = T::Error;

    fn parser() -> impl Parser<I, Self, Error = Self::Error> {
        T::parser().repeated().collect()
    }
}

impl<'a, I, T> Parseable<'a, I> for VecDeque<T>
where
    I: Clone + Hash + Eq + 'a,
    T: Parseable<'a, I>,
{
    type Error = T::Error;
    fn parser() -> impl Parser<I, Self, Error = Self::Error> {
        T::parser().repeated().collect()
    }
}

impl<'a, I, T> Parseable<'a, I> for LinkedList<T>
where
    I: Clone + Hash + Eq + 'a,
    T: Parseable<'a, I>,
{
    type Error = T::Error;

    fn parser() -> impl Parser<I, Self, Error = Self::Error> {
        T::parser().repeated().collect()
    }
}

impl<'a, I, T> Parseable<'a, I> for HashSet<T>
where
    I: Clone + Hash + Eq + 'a,
    T: Parseable<'a, I> + Eq + Hash,
{
    type Error = T::Error;

    fn parser() -> impl Parser<I, Self, Error = Self::Error> {
        T::parser().repeated().collect()
    }
}

impl<'a, I, T> Parseable<'a, I> for BTreeSet<T>
where
    I: Clone + Hash + Eq + 'a,
    T: Parseable<'a, I> + Eq + Ord,
{
    type Error = T::Error;

    fn parser() -> impl Parser<I, Self, Error = Self::Error> {
        T::parser().repeated().collect()
    }
}

impl<'a, I, E, K, V> Parseable<'a, I> for HashMap<K, V>
where
    I: Clone + Hash + Eq + 'a,
    E: chumsky::Error<I>,
    K: Parseable<'a, I, Error = E> + Eq + Hash,
    V: Parseable<'a, I, Error = E>,
{
    type Error = E;

    fn parser() -> impl Parser<I, Self, Error = Self::Error> {
        K::parser().then(V::parser()).repeated().collect()
    }
}

impl<'a, I, E, K, V> Parseable<'a, I> for BTreeMap<K, V>
where
    I: Clone + Hash + Eq + 'a,
    E: chumsky::Error<I>,
    K: Parseable<'a, I, Error = E> + Eq + Ord,
    V: Parseable<'a, I, Error = E>,
{
    type Error = E;

    fn parser() -> impl Parser<I, Self, Error = Self::Error> {
        K::parser().then(V::parser()).repeated().collect()
    }
}

impl<'a, I, E, T, U> Parseable<'a, I> for (T, U)
where
    I: Clone + Hash + Eq + 'a,
    E: chumsky::Error<I>,
    T: Parseable<'a, I, Error = E>,
    U: Parseable<'a, I, Error = E>,
{
    type Error = E;

    fn parser() -> impl Parser<I, Self, Error = Self::Error> {
        T::parser().then(U::parser())
    }
}

impl<'a, I, E, T, U, V> Parseable<'a, I> for (T, U, V)
where
    I: Clone + Hash + Eq + 'a,
    E: chumsky::Error<I>,
    T: Parseable<'a, I, Error = E>,
    U: Parseable<'a, I, Error = E>,
    V: Parseable<'a, I, Error = E>,
{
    type Error = E;

    fn parser() -> impl Parser<I, Self, Error = Self::Error> {
        T::parser()
            .then(U::parser())
            .then(V::parser())
            .map(|((t, u), v)| (t, u, v))
    }
}

impl<'a, I, E, T, U, V, W> Parseable<'a, I> for (T, U, V, W)
where
    I: Clone + Hash + Eq + 'a,
    E: chumsky::Error<I>,
    T: Parseable<'a, I, Error = E>,
    U: Parseable<'a, I, Error = E>,
    V: Parseable<'a, I, Error = E>,
    W: Parseable<'a, I, Error = E>,
{
    type Error = E;

    fn parser() -> impl Parser<I, Self, Error = Self::Error> {
        T::parser()
            .then(U::parser())
            .then(V::parser())
            .then(W::parser())
            .map(|(((t, u), v), w)| (t, u, v, w))
    }
}
