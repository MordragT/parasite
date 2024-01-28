use parasite_core::{
    builder::Syntactical,
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

grammar!(
    mod ast {
        #[start]
        enum S {
            A((u8, A, u8)),
        }

        enum A {
            S((bool, Box<S>, bool)),
            End,
        }
    }
);

fn main() {}
