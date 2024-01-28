use parasite_core::{
    builder::Syntactical,
    chumsky::{error::Cheap, primitive::just, text::digits, Parseable, Parser},
    combinators::Rec,
    grammar::{Grammar, TypeName},
};
use parasite_macros::*;

#[derive(Terminal)]
pub struct Leaf {}

#[derive(Syntactical)]
pub enum Branch {
    Branch(Rec<(Leaf, Branch)>),
    Leaf(Leaf),
}

fn syntactical() {
    let mut grammar = Grammar::new(TypeName::of::<Branch>());
    let mut stack = Vec::new();

    Branch::generate(&mut grammar, &mut stack);

    dbg!(&grammar);
}

// #[derive(Parseable)]
pub struct Heading<'a> {
    level: HeadingLevel<'a>,
    content: Content,
}

impl<'a> parasite_core::chumsky::Parseable<'a, &'a str> for Heading<'a> {
    type Error = parasite_core::chumsky::error::Cheap<&'a str>;
    fn parse() -> impl parasite_core::chumsky::Parser<&'a str, Self, Error = Self::Error> {
        use parasite_core::chumsky::Parser;
        <HeadingLevel<'a> as parasite_core::chumsky::Parseable<&'a str>>::parse()
            .then(<Content as parasite_core::chumsky::Parseable<'a, char>>::parse())
            .map(|(level, content)| Self { level, content })
    }
}

pub struct HeadingLevel<'a> {
    level: Vec<&'a str>,
}

impl<'a> Parseable<'a, &'a str> for HeadingLevel<'a> {
    fn parse() -> impl Parser<&'a str, Self, Error = Self::Error> {
        just("=")
            .repeated()
            .at_least(1)
            .at_most(8)
            .collect()
            .map(|level| HeadingLevel { level })
    }
}

pub struct Content {
    content: String,
}

impl<'a> Parseable<'a, char> for Content {
    fn parse() -> impl Parser<char, Self, Error = Self::Error> {
        digits(10).map(|content| Content { content })
    }
}

// grammar!(
//     mod ast {

//         #[begin]
//         enum S {
//             A((u8, A, u8)),
//         }

//         enum A {
//             S((bool, Box<S>, bool)),
//             End,
//         }
//     }
// );

fn main() {}
