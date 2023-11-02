use super::Grammar;
use proc_macro2::{Ident, Span, TokenStream};
use std::collections::HashMap;
use syn::{Data, Fields, ItemImpl, ItemStruct};

#[derive(Debug)]
pub struct TokenVariants {
    token_ident: Ident,
    variants: HashMap<Ident, Fields>,
}

impl TokenVariants {
    fn declaration(ident: &Ident, fields: &Fields) -> ItemStruct {
        match fields {
            Fields::Unnamed(fields) => {
                syn::parse_quote!(struct #ident #fields ;)
            }
            Fields::Named(fields) => {
                syn::parse_quote!(struct #ident #fields)
            }
            Fields::Unit => syn::parse_quote!(struct #ident ;),
        }
    }

    pub fn declarations(&self) -> Vec<ItemStruct> {
        self.variants
            .iter()
            .map(|(ident, fields)| Self::declaration(ident, fields))
            .collect()
    }

    fn pattern(ident: &Ident, fields: &Fields) -> TokenStream {
        match fields {
            Fields::Unnamed(fields) => {
                let members = (0..fields.unnamed.len())
                    .into_iter()
                    .map(|n| Ident::new(&format!("n{n}"), Span::call_site()));
                syn::parse_quote!(#ident (#(#members),*))
            }
            fields => syn::parse_quote!(#ident #fields),
        }
    }

    pub fn try_from_impls(&self) -> Vec<ItemImpl> {
        self.variants
            .iter()
            .map(|(ident, fields)| {
                let pat = Self::pattern(ident, fields);
                let token_ident = &self.token_ident;
                syn::parse_quote!(
                    impl TryFrom<#token_ident> for #ident {
                        type Error = String;

                        fn try_from(token: #token_ident) -> Result<#ident, Self::Error> {
                            match token {
                                #token_ident::#pat => Ok(#pat),
                                _ => Err("Token not of variant kind".to_owned())
                            }
                        }
                    }
                )
            })
            .collect()
    }
}

impl Grammar {
    pub fn token_variants(&self) -> TokenVariants {
        let token_ident = self.token.ident.clone();

        let variants = match self.token.data.clone() {
            Data::Enum(data) => data
                .variants
                .into_iter()
                .map(|variant| (variant.ident, variant.fields))
                .collect(),
            _ => unreachable!(),
        };

        TokenVariants {
            token_ident,
            variants,
        }
    }
}
