use anymap3::AnyMap;
pub use chumsky::*;

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList, VecDeque},
    fmt::Debug,
    hash::Hash,
};

mod combinators;

pub type Context = AnyMap;

pub trait Parseable<'a, I: Hash + Eq + Clone + 'a>: Sized {
    type Error: chumsky::Error<I> = chumsky::error::Simple<I>;

    fn parser(ctx: &mut Context) -> BoxedParser<'a, I, Self, Self::Error>;
}

impl<I, T> Parseable<'static, I> for Option<T>
where
    I: Clone + Hash + Eq + 'static,
    T: Parseable<'static, I> + 'static,
{
    type Error = T::Error;

    fn parser(ctx: &mut Context) -> BoxedParser<'static, I, Self, Self::Error> {
        if !ctx.contains::<BoxedParser<'static, I, Self, Self::Error>>() {
            let t = T::parser(ctx);

            let parser = t.or_not().boxed();

            ctx.insert(parser);
        }

        ctx.get::<BoxedParser<'static, I, Self, Self::Error>>()
            .unwrap()
            .clone()
    }
}

impl<'a, I, T, const N: usize> Parseable<'static, I> for [T; N]
where
    I: Clone + Hash + Eq + 'static,
    T: Parseable<'static, I> + Debug + 'static,
{
    type Error = T::Error;

    fn parser(ctx: &mut Context) -> BoxedParser<'static, I, Self, Self::Error> {
        if !ctx.contains::<BoxedParser<'static, I, Self, Self::Error>>() {
            let t = T::parser(ctx);

            let parser = t
                .repeated()
                .exactly(N)
                .collect::<Vec<_>>()
                .map(|vec| Self::try_from(vec).unwrap())
                .boxed();

            ctx.insert(parser);
        }

        ctx.get::<BoxedParser<'static, I, Self, Self::Error>>()
            .unwrap()
            .clone()
    }
}

impl<'a, I, T> Parseable<'static, I> for Vec<T>
where
    I: Clone + Hash + Eq + 'static,
    T: Parseable<'static, I> + 'static,
{
    type Error = T::Error;

    fn parser(ctx: &mut Context) -> BoxedParser<'static, I, Self, Self::Error> {
        if !ctx.contains::<BoxedParser<'static, I, Self, Self::Error>>() {
            let t = T::parser(ctx);

            let parser = t.repeated().collect::<Vec<_>>().boxed();

            ctx.insert(parser);
        }

        ctx.get::<BoxedParser<'static, I, Self, Self::Error>>()
            .unwrap()
            .clone()
    }
}

impl<'a, I, T> Parseable<'static, I> for VecDeque<T>
where
    I: Clone + Hash + Eq + 'static,
    T: Parseable<'static, I> + 'static,
{
    type Error = T::Error;
    fn parser(ctx: &mut Context) -> BoxedParser<'static, I, Self, Self::Error> {
        if !ctx.contains::<BoxedParser<'static, I, Self, Self::Error>>() {
            let t = T::parser(ctx);

            let parser = t.repeated().collect::<VecDeque<_>>().boxed();

            ctx.insert(parser);
        }

        ctx.get::<BoxedParser<'static, I, Self, Self::Error>>()
            .unwrap()
            .clone()
    }
}

impl<'a, I, T> Parseable<'static, I> for LinkedList<T>
where
    I: Clone + Hash + Eq + 'static,
    T: Parseable<'static, I> + 'static,
{
    type Error = T::Error;

    fn parser(ctx: &mut Context) -> BoxedParser<'static, I, Self, Self::Error> {
        if !ctx.contains::<BoxedParser<'static, I, Self, Self::Error>>() {
            let t = T::parser(ctx);

            let parser = t.repeated().collect::<LinkedList<_>>().boxed();

            ctx.insert(parser);
        }

        ctx.get::<BoxedParser<'static, I, Self, Self::Error>>()
            .unwrap()
            .clone()
    }
}

impl<'a, I, T> Parseable<'static, I> for HashSet<T>
where
    I: Clone + Hash + Eq + 'static,
    T: Parseable<'static, I> + Eq + Hash + 'static,
{
    type Error = T::Error;

    fn parser(ctx: &mut Context) -> BoxedParser<'static, I, Self, Self::Error> {
        if !ctx.contains::<BoxedParser<'static, I, Self, Self::Error>>() {
            let t = T::parser(ctx);

            let parser = t.repeated().collect::<HashSet<_>>().boxed();

            ctx.insert(parser);
        }

        ctx.get::<BoxedParser<'static, I, Self, Self::Error>>()
            .unwrap()
            .clone()
    }
}

impl<'a, I, T> Parseable<'static, I> for BTreeSet<T>
where
    I: Clone + Hash + Eq + 'static,
    T: Parseable<'static, I> + Eq + Ord + 'static,
{
    type Error = T::Error;

    fn parser(ctx: &mut Context) -> BoxedParser<'static, I, Self, Self::Error> {
        if !ctx.contains::<BoxedParser<'static, I, Self, Self::Error>>() {
            let t = T::parser(ctx);

            let parser = t.repeated().collect::<BTreeSet<_>>().boxed();

            ctx.insert(parser);
        }

        ctx.get::<BoxedParser<'static, I, Self, Self::Error>>()
            .unwrap()
            .clone()
    }
}

impl<I, E, K, V> Parseable<'static, I> for HashMap<K, V>
where
    I: Clone + Hash + Eq + 'static,
    E: chumsky::Error<I> + 'static,
    K: Parseable<'static, I, Error = E> + Eq + Hash + 'static,
    V: Parseable<'static, I, Error = E> + 'static,
{
    type Error = E;

    fn parser(ctx: &mut Context) -> BoxedParser<'static, I, Self, Self::Error> {
        if !ctx.contains::<BoxedParser<'static, I, Self, Self::Error>>() {
            let k = K::parser(ctx);
            let v = V::parser(ctx);

            let parser = k.then(v).repeated().collect::<HashMap<_, _>>().boxed();

            ctx.insert(parser);
        }

        ctx.get::<BoxedParser<'static, I, Self, Self::Error>>()
            .unwrap()
            .clone()
    }
}

impl<'a, I, E, K, V> Parseable<'static, I> for BTreeMap<K, V>
where
    I: Clone + Hash + Eq + 'static,
    E: chumsky::Error<I> + 'static,
    K: Parseable<'static, I, Error = E> + Eq + Ord + 'static,
    V: Parseable<'static, I, Error = E> + 'static,
{
    type Error = E;

    fn parser(ctx: &mut Context) -> BoxedParser<'static, I, Self, Self::Error> {
        if !ctx.contains::<BoxedParser<'static, I, Self, Self::Error>>() {
            let k = K::parser(ctx);
            let v = V::parser(ctx);

            let parser = k.then(v).repeated().collect::<BTreeMap<_, _>>().boxed();

            ctx.insert(parser);
        }

        ctx.get::<BoxedParser<'static, I, Self, Self::Error>>()
            .unwrap()
            .clone()
    }
}

impl<'a, I, E, T, U> Parseable<'static, I> for (T, U)
where
    I: Clone + Hash + Eq + 'static,
    E: chumsky::Error<I> + 'static,
    T: Parseable<'static, I, Error = E> + 'static,
    U: Parseable<'static, I, Error = E> + 'static,
{
    type Error = E;

    fn parser(ctx: &mut Context) -> BoxedParser<'static, I, Self, Self::Error> {
        if !ctx.contains::<BoxedParser<'static, I, Self, Self::Error>>() {
            let t = T::parser(ctx);
            let u = U::parser(ctx);

            let parser = t.then(u).boxed();

            ctx.insert(parser);
        }

        ctx.get::<BoxedParser<'static, I, Self, Self::Error>>()
            .unwrap()
            .clone()
    }
}

impl<'a, I, E, T, U, V> Parseable<'static, I> for (T, U, V)
where
    I: Clone + Hash + Eq + 'static,
    E: chumsky::Error<I> + 'static,
    T: Parseable<'static, I, Error = E> + 'static,
    U: Parseable<'static, I, Error = E> + 'static,
    V: Parseable<'static, I, Error = E> + 'static,
{
    type Error = E;

    fn parser(ctx: &mut Context) -> BoxedParser<'static, I, Self, Self::Error> {
        if !ctx.contains::<BoxedParser<'static, I, Self, Self::Error>>() {
            let t = T::parser(ctx);
            let u = U::parser(ctx);
            let v = V::parser(ctx);

            let parser = t.then(u).then(v).map(|((t, u), v)| (t, u, v)).boxed();

            ctx.insert(parser);
        }

        ctx.get::<BoxedParser<'static, I, Self, Self::Error>>()
            .unwrap()
            .clone()
    }
}

impl<'a, I, E, T, U, V, W> Parseable<'static, I> for (T, U, V, W)
where
    I: Clone + Hash + Eq + 'static,
    E: chumsky::Error<I> + 'static,
    T: Parseable<'static, I, Error = E> + 'static,
    U: Parseable<'static, I, Error = E> + 'static,
    V: Parseable<'static, I, Error = E> + 'static,
    W: Parseable<'static, I, Error = E> + 'static,
{
    type Error = E;

    fn parser(ctx: &mut Context) -> BoxedParser<'static, I, Self, Self::Error> {
        if !ctx.contains::<BoxedParser<'static, I, Self, Self::Error>>() {
            let t = T::parser(ctx);
            let u = U::parser(ctx);
            let v = V::parser(ctx);
            let w = W::parser(ctx);

            let parser = t
                .then(u)
                .then(v)
                .then(w)
                .map(|(((t, u), v), w)| (t, u, v, w))
                .boxed();

            ctx.insert(parser);
        }

        ctx.get::<BoxedParser<'static, I, Self, Self::Error>>()
            .unwrap()
            .clone()
    }
}
