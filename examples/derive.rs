use parasite_core::{
    builder::Syntactical,
    combinators::Rec,
    grammar::{Grammar, Key},
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
    let mut grammar = Grammar::new(Key::of::<Branch>());
    let mut stack = Vec::new();

    Branch::generate(&mut grammar, &mut stack);

    println!("{grammar}");

    let k = 2;
    let table = grammar.table(k);

    println!("{table}")
}
