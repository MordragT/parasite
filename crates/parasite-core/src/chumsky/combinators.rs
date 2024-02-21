use std::hash::Hash;

use super::{Context, Parseable};
use crate::combinators::{
    Any, DelimitedBy, End, Identifier, Just, NewLine, NonEmptyVec, PaddedBy, Rec, SeparatedBy,
    WhiteSpace,
};
use chumsky::{
    primitive::{any, just},
    recursive::Recursive,
    text::{self, newline, whitespace, Character},
    BoxedParser, Parser,
};

impl<I, T> Parseable<'static, I> for Rec<T>
where
    I: Clone + Hash + Eq + 'static,
    T: Parseable<'static, I> + 'static,
{
    type Error = T::Error;

    fn parser(ctx: &mut Context) -> BoxedParser<'static, I, Self, Self::Error> {
        ctx.get::<Recursive<'static, I, T, Self::Error>>()
            .unwrap()
            .clone()
            .map(|t| Rec(Box::new(t)))
            .boxed()
    }
}

impl<I, T> Parseable<'static, I> for NonEmptyVec<T>
where
    I: Clone + Hash + Eq + 'static,
    T: Parseable<'static, I> + 'static,
{
    type Error = T::Error;

    fn parser(ctx: &mut Context) -> BoxedParser<'static, I, Self, Self::Error> {
        if !ctx.contains::<BoxedParser<'static, I, Self, Self::Error>>() {
            let t = T::parser(ctx);

            let parser = t.repeated().at_least(1).collect().map(NonEmptyVec).boxed();

            ctx.insert(parser);
        }

        ctx.get::<BoxedParser<'static, I, Self, Self::Error>>()
            .unwrap()
            .clone()
    }
}

impl<I, S, T> Parseable<'static, I> for SeparatedBy<S, T>
where
    I: Clone + Hash + Eq + 'static,
    T: Parseable<'static, I> + 'static,
    S: Parseable<'static, I, Error = T::Error> + 'static,
{
    type Error = T::Error;

    fn parser(ctx: &mut Context) -> BoxedParser<'static, I, Self, Self::Error> {
        if !ctx.contains::<BoxedParser<'static, I, Self, Self::Error>>() {
            let t = T::parser(ctx);
            let s = S::parser(ctx);

            let parser = t
                .separated_by(s)
                .at_least(1)
                .collect()
                .map(SeparatedBy::<S, T>::new)
                .boxed();

            ctx.insert(parser);
        }

        ctx.get::<BoxedParser<'static, I, Self, Self::Error>>()
            .unwrap()
            .clone()
    }
}

impl<I, P, T> Parseable<'static, I> for PaddedBy<P, T>
where
    I: Clone + Hash + Eq + 'static,
    T: Parseable<'static, I> + 'static,
    P: Parseable<'static, I, Error = T::Error> + 'static,
{
    type Error = T::Error;

    fn parser(ctx: &mut Context) -> BoxedParser<'static, I, Self, Self::Error> {
        if !ctx.contains::<BoxedParser<'static, I, Self, Self::Error>>() {
            let t = T::parser(ctx);
            let p = P::parser(ctx);

            let parser = t.padded_by(p).map(PaddedBy::<P, T>::new).boxed();

            ctx.insert(parser);
        }

        ctx.get::<BoxedParser<'static, I, Self, Self::Error>>()
            .unwrap()
            .clone()
    }
}

impl<I, L, R, T> Parseable<'static, I> for DelimitedBy<L, R, T>
where
    I: Clone + Hash + Eq + 'static,
    T: Parseable<'static, I> + 'static,
    L: Parseable<'static, I, Error = T::Error> + 'static,
    R: Parseable<'static, I, Error = T::Error> + 'static,
{
    type Error = T::Error;

    fn parser(ctx: &mut Context) -> BoxedParser<'static, I, Self, Self::Error> {
        if !ctx.contains::<BoxedParser<'static, I, Self, Self::Error>>() {
            let t = T::parser(ctx);
            let l = L::parser(ctx);
            let r = R::parser(ctx);

            let parser = t
                .delimited_by(l, r)
                .map(DelimitedBy::<L, R, T>::new)
                .boxed();

            ctx.insert(parser);
        }

        ctx.get::<BoxedParser<'static, I, Self, Self::Error>>()
            .unwrap()
            .clone()
    }
}

impl<'a, const CHAR: char> Parseable<'a, char> for Just<CHAR> {
    fn parser(_: &mut Context) -> BoxedParser<'a, char, Self, Self::Error> {
        just(CHAR).to(Self()).boxed()
    }
}

impl<'a> Parseable<'a, char> for End {
    fn parser(_: &mut Context) -> BoxedParser<'a, char, Self, Self::Error> {
        chumsky::primitive::end().to(Self).boxed()
    }
}

impl<'a> Parseable<'a, char> for Any {
    fn parser(_: &mut Context) -> BoxedParser<'a, char, Self, Self::Error> {
        any().map(Any).boxed()
    }
}

impl<'a, I: Character + Eq + Hash + 'static> Parseable<'a, I> for NewLine {
    fn parser(_: &mut Context) -> BoxedParser<'static, I, Self, Self::Error> {
        newline().to(Self).boxed()
    }
}

impl<'a> Parseable<'a, char> for WhiteSpace {
    fn parser(_: &mut Context) -> BoxedParser<'a, char, Self, Self::Error> {
        whitespace().to(WhiteSpace).boxed()
    }
}

impl<'a> Parseable<'a, char> for Identifier {
    fn parser(_: &mut Context) -> BoxedParser<'a, char, Self, Self::Error> {
        text::ident().map(Identifier).boxed()
    }
}
