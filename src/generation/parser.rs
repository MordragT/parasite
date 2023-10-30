use proc_macro2::Ident;

use crate::{
    analysis::{first::FirstSets, follow::FollowSets},
    grammar::{Expander, Grammar},
    GrammarAst,
};

pub struct ParserGenerator<'a> {
    terminals: &'a Vec<Ident>,
}

impl<'a> ParserGenerator<'a> {
    pub fn new(grammar: &'a GrammarAst) -> Self {
        Self {
            terminals: &grammar.terminals,
        }
    }
}

/*
Generate function

parse(TokenInputStream) -> Start
*/
