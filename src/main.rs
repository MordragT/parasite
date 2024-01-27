use parasite_core::{
    builder::{Rec, Syntactical},
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

fn main() {
    let mut grammar = Grammar::new(TypeName::of::<Branch>());
    let mut stack = Vec::new();

    Branch::generate(&mut grammar, &mut stack);

    dbg!(&grammar);
}
