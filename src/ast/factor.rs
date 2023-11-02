use super::{alternation::AlternationsNode, Node, NodeIndex};
use syn::{
    braced, bracketed, parenthesized,
    parse::Parse,
    token::{Brace, Bracket, Paren},
    Ident,
};

#[derive(Debug)]
pub enum FactorNode {
    // '(' Alternations ')'
    Group(AlternationsNode),
    // '{' Alternations '}'
    Repeat(AlternationsNode),
    // '[' Alternations ']'
    Optional(AlternationsNode),
    Symbol(Ident),
}

impl FactorNode {
    pub fn get(&self, mut index: NodeIndex) -> Node {
        if !index.is_empty() {
            index.remove(0);
            match self {
                Self::Group(alternations)
                | Self::Optional(alternations)
                | Self::Repeat(alternations) => alternations.get(index),
                Self::Symbol(_) => Node::Factor(self, Vec::new()),
            }
        } else {
            Node::Factor(self, Vec::new())
        }
    }
}

impl Parse for FactorNode {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;

        let factor = if input.peek(Paren) {
            parenthesized!(content in input);
            let alternations = content.parse()?;
            Self::Group(alternations)
        } else if input.peek(Brace) {
            braced!(content in input);
            let alternations = content.parse()?;
            Self::Repeat(alternations)
        } else if input.peek(Bracket) {
            bracketed!(content in input);
            let alternations = content.parse()?;
            Self::Optional(alternations)
        } else {
            Self::Symbol(input.parse()?)
        };

        Ok(factor)
    }
}
