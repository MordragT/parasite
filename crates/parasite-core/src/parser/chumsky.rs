use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList, VecDeque},
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
};

use chumsky::Parser;

use crate::{
    builder::{NonEmptyVec, Rec, Syntactical},
    grammar::{Grammar, TypeName},
    table::Table,
};

// pub struct ParserGenerator<T: Syntactical> {
//     grammar: Grammar,
//     table: Table,
//     ast_ty: PhantomData<T>,
// }

// impl<T: Syntactical> ParserGenerator<T> {
//     pub fn new(k: usize) -> Self {
//         let mut grammar = Grammar::new(TypeName::of::<T>());
//         let mut stack = Vec::new();

//         T::generate(&mut grammar, &mut stack);

//         let table = grammar.table(k);

//         Self {
//             grammar,
//             table,
//             ast_ty: PhantomData,
//         }
//     }

//     pub fn generate(&self) {
//         todo!()
//     }
// }

// TODO reuse Grammar inside proc macro to derive Parseable trait

pub trait Parseable<I: Clone>: Sized {
    type Error<'a>: chumsky::Error<I>;

    fn parse<'a>() -> impl Parser<I, Self, Error = Self::Error<'a>>;
}

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

impl<I: Clone, T: Parseable<I>> Parseable<I> for Option<T> {
    type Error<'a> = T::Error<'a>;

    fn parse<'a>() -> impl Parser<I, Self, Error = Self::Error<'a>> {
        T::parse().or_not()
    }
}

impl<I: Clone, T: Parseable<I> + Debug, const N: usize> Parseable<I> for [T; N] {
    type Error<'a> = T::Error<'a>;

    fn parse<'a>() -> impl Parser<I, Self, Error = Self::Error<'a>> {
        T::parse()
            .repeated()
            .exactly(N)
            .collect::<Vec<_>>()
            .map(|vec| Self::try_from(vec).unwrap())
    }
}

impl<I: Clone, T: Parseable<I>> Parseable<I> for Vec<T> {
    type Error<'a> = T::Error<'a>;

    fn parse<'a>() -> impl Parser<I, Self, Error = Self::Error<'a>> {
        T::parse().repeated().collect()
    }
}

impl<I: Clone, T: Parseable<I>> Parseable<I> for VecDeque<T> {
    type Error<'a> = T::Error<'a>;

    fn parse<'a>() -> impl Parser<I, Self, Error = Self::Error<'a>> {
        T::parse().repeated().collect()
    }
}

impl<I: Clone, T: Parseable<I>> Parseable<I> for LinkedList<T> {
    type Error<'a> = T::Error<'a>;

    fn parse<'a>() -> impl Parser<I, Self, Error = Self::Error<'a>> {
        T::parse().repeated().collect()
    }
}

impl<I: Clone, T: Parseable<I> + Eq + Hash> Parseable<I> for HashSet<T> {
    type Error<'a> = T::Error<'a>;

    fn parse<'a>() -> impl Parser<I, Self, Error = Self::Error<'a>> {
        T::parse().repeated().collect()
    }
}

impl<I: Clone, T: Parseable<I> + Ord + Eq> Parseable<I> for BTreeSet<T> {
    type Error<'a> = T::Error<'a>;

    fn parse<'a>() -> impl Parser<I, Self, Error = Self::Error<'a>> {
        T::parse().repeated().collect()
    }
}

impl<I, E, K, V> Parseable<I> for HashMap<K, V>
where
    I: Clone,
    E: chumsky::Error<I>,
    K: for<'a> Parseable<I, Error<'a> = E> + Eq + Hash,
    V: for<'a> Parseable<I, Error<'a> = E>,
{
    type Error<'a> = E;

    fn parse<'a>() -> impl Parser<I, Self, Error = Self::Error<'a>> {
        K::parse().then(V::parse()).repeated().collect()
    }
}

impl<I, E, K, V> Parseable<I> for BTreeMap<K, V>
where
    I: Clone,
    E: chumsky::Error<I>,
    K: for<'a> Parseable<I, Error<'a> = E> + Ord + Eq,
    V: for<'a> Parseable<I, Error<'a> = E>,
{
    type Error<'a> = E;

    fn parse<'a>() -> impl Parser<I, Self, Error = Self::Error<'a>> {
        K::parse().then(V::parse()).repeated().collect()
    }
}
