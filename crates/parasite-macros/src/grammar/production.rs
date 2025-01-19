use super::{alternation::AlternationsNode, Node, NodeIndex};
use syn::{parse::Parse, Ident, Token};

#[derive(Debug)]
pub struct ProductionNode {
    pub lhs: Ident,
    /// split by |
    pub alternations: AlternationsNode,
}

impl ProductionNode {
    pub fn get(&self, index: NodeIndex) -> Node {
        if !index.is_empty() {
            self.alternations.get(index)
        } else {
            Node::Production(self, Vec::new())
        }
    }
}

impl Parse for ProductionNode {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lhs = input.parse()?;
        let _colon = input.parse::<Token![:]>()?;
        let alternations = input.parse()?;
        let _semi = input.parse::<Token![;]>()?;

        Ok(Self { lhs, alternations })
    }
}
