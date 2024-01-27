use parasite_core::builder::Rec;
use parasite_macros::*;

#[derive(Syntactical)]
#[Terminal]
pub struct Leaf {}

#[derive(Syntactical)]
pub enum Branch {
    Branch(Rec<(Leaf, Branch)>),
    Leaf(Leaf),
}

fn main() {}
