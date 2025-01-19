use super::{factor::FactorNode, Node, NodeIndex};
use syn::{parse::Parse, Token};

#[derive(Debug)]
pub struct AlternationsNode {
    pub alternations: Vec<AlternationNode>,
}

impl AlternationsNode {
    pub fn get(&self, mut index: NodeIndex) -> Node {
        if !index.is_empty() {
            let id = index.remove(0);
            self.alternations[id].get(index)
        } else {
            Node::Alternations(self, Vec::new())
        }
    }
}

impl Parse for AlternationsNode {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut alternations = vec![input.parse()?];
        while input.peek(Token!(|)) {
            let _pipe = input.parse::<Token!(|)>()?;
            let alternation = input.parse()?;
            alternations.push(alternation);
        }

        Ok(Self { alternations })
    }
}

#[derive(Debug)]
pub struct AlternationNode {
    /// split by ' '
    pub factors: Vec<FactorNode>,
}

impl AlternationNode {
    pub fn get(&self, mut index: NodeIndex) -> Node {
        if !index.is_empty() {
            let id = index.remove(0);
            self.factors[id].get(index)
        } else {
            Node::Alternation(self, Vec::new())
        }
    }
}

impl Parse for AlternationNode {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut factors = vec![input.parse()?];
        while !input.peek(Token![;])
        // && !input.peek(Paren)
        // && !input.peek(Brace)
        // && !input.peek(Bracket)
        && !input.peek(Token!(|))
        && !input.is_empty()
        {
            let factor = input.parse()?;
            factors.push(factor);
        }
        Ok(Self { factors })
    }
}
