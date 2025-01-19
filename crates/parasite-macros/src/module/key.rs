use parasite_core::grammar::Key;
use quote::ToTokens;
use syn::{Ident, Path, PathSegment, Type, TypeArray, TypePath, TypeTuple};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeKey {
    Array(TypeArray),
    Tuple(TypeTuple),
    Path(TypePath),
}

impl TypeKey {
    pub fn new(ident: Ident) -> Self {
        let segment = PathSegment {
            ident,
            arguments: syn::PathArguments::None,
        };

        let path = Path::from(segment);

        Self::Path(TypePath { qself: None, path })
    }
}

impl ToTokens for TypeKey {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Array(array) => array.to_tokens(tokens),
            Self::Tuple(tuple) => tuple.to_tokens(tokens),
            Self::Path(path) => path.to_tokens(tokens),
        }
    }
}

impl Into<Key> for TypeKey {
    fn into(self) -> Key {
        let s = self.to_token_stream().to_string();
        Key::new(s)
    }
}

impl From<Ident> for TypeKey {
    fn from(value: Ident) -> Self {
        Self::new(value)
    }
}

impl From<TypeArray> for TypeKey {
    fn from(value: TypeArray) -> Self {
        Self::Array(value)
    }
}

impl From<TypeTuple> for TypeKey {
    fn from(value: TypeTuple) -> Self {
        Self::Tuple(value)
    }
}

impl From<TypePath> for TypeKey {
    fn from(value: TypePath) -> Self {
        Self::Path(value)
    }
}

impl TryFrom<Type> for TypeKey {
    type Error = Type;

    fn try_from(value: Type) -> Result<Self, Self::Error> {
        match value {
            Type::Array(array) => Ok(array.into()),
            Type::Tuple(tuple) => Ok(tuple.into()),
            Type::Path(path) => Ok(path.into()),
            _ => Err(value),
        }
    }
}
