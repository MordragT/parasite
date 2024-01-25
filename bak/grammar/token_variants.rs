use super::Grammar;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::collections::HashMap;
use syn::{Data, Fields, ItemEnum, ItemImpl, ItemStruct};

#[derive(Debug)]
pub struct TokenVariants {
    pub(crate) token_ident: Ident,
    pub(crate) variants: HashMap<Ident, Fields>,
}

impl TokenVariants {
    fn declaration(ident: &Ident, fields: &Fields) -> ItemStruct {
        match fields {
            Fields::Unnamed(fields) => {
                syn::parse_quote!(
                    #[derive(Debug)]
                    struct #ident #fields ;
                )
            }
            Fields::Named(fields) => {
                syn::parse_quote!(
                    #[derive(Debug)]
                    struct #ident #fields
                )
            }
            Fields::Unit => syn::parse_quote!(
                #[derive(Debug)]
                struct #ident ;
            ),
        }
    }

    pub fn declarations(&self) -> Vec<ItemStruct> {
        self.variants
            .iter()
            .map(|(ident, fields)| Self::declaration(ident, fields))
            .collect()
    }

    pub fn into_kind_impl(&self) -> ItemImpl {
        let token_ident = &self.token_ident;
        let token_kind_ident = Ident::new(&format!("{token_ident}Kind"), Span::call_site());

        let matchings = self.variants.iter().map(|(ident, fields)| {
            let pat = Self::pattern(ident, fields);
            quote!(#token_ident::#pat => #token_kind_ident::#ident)
        });

        syn::parse_quote!(
            impl #token_ident {
                pub fn kind(&self) -> #token_kind_ident {
                    match self {
                        #(#matchings),*
                    }
                }
            }
        )
    }

    pub fn kind_decl(&self) -> ItemEnum {
        let token_ident = &self.token_ident;
        let token_kind_ident = Ident::new(&format!("{token_ident}Kind"), Span::call_site());
        let variants = self.variants.iter().map(|(ident, _)| ident);

        syn::parse_quote!(
            pub enum #token_kind_ident {
                #(#variants),*
            }
        )
    }

    pub fn pattern(ident: &Ident, fields: &Fields) -> TokenStream {
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
        let token_ident = &self.token_ident;

        self.variants
            .iter()
            .map(|(ident, fields)| {
                let pat = Self::pattern(ident, fields);
                let error = format!("Token not of variant kind: {ident}");
                syn::parse_quote!(
                    impl TryFrom<#token_ident> for #ident {
                        type Error = String;

                        fn try_from(token: #token_ident) -> Result<#ident, Self::Error> {
                            match token {
                                #token_ident::#pat => Ok(#pat),
                                _ => Err(#error.to_owned())
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
