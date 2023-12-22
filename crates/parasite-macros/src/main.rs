use parasite_core::{
    ast::nodes::{Rec, Start},
    ir::builder::IrBuilder,
};
use parasite_macros::*;

#[derive(Node)]
#[Terminal]
// #[terminal = Token::Leaf]
pub struct Leaf {}

#[derive(Node)]
#[Start]
pub enum Branch {
    Branch(Rec<(Leaf, Branch)>),
    Leaf(Leaf),
}

// pub enum MoreComplex {
//     #[collection(repeat = 4)]
//     List4(Vec<(Leaf, Branch)>),
//     #[leaf]
//     Leaf,
// }

fn main() {
    let ast = Branch::ast(1);
    dbg!(&ast);

    // TODO Order isnt right
    // let prefix = ast.into_iter().collect::<Vec<_>>();
    // dbg!(&prefix);

    IrBuilder::build(ast);
}
