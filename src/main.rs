use parasite_core::{
    builder::Syntactical,
    chumsky::{
        primitive::{empty, just},
        text::digits,
        Parseable, Parser,
    },
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
// enum UnitEnum {}

#[derive(Parseable)]
pub enum Test {
    Level(HeadingLevel),
    Content(Content),
}

#[derive(Parseable)]
pub struct UnitStruct;

#[derive(Parseable)]
pub struct TupleStruct(HeadingLevel, Content);

#[derive(Parseable)]
pub struct Heading {
    level: HeadingLevel,
    content: Content,
}

pub struct HeadingLevel {
    level: usize,
}

impl<'a> Parseable<'a, char> for HeadingLevel {
    fn parser() -> impl Parser<char, Self, Error = Self::Error> + Clone {
        just('=')
            .repeated()
            .at_least(1)
            .at_most(8)
            .collect::<String>()
            .map(|level| HeadingLevel { level: level.len() })
    }
}

pub struct Content {
    content: String,
}

impl<'a> Parseable<'a, char> for Content {
    fn parser() -> impl Parser<char, Self, Error = Self::Error> + Clone {
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

fn main() {
    let heading = match Heading::parser().parse("==1234") {
        Ok(heading) => heading,
        Err(errs) => {
            for e in errs {
                println!("{e:?}");
            }
            panic!()
        }
    };
}
