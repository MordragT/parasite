use cst::{Data, Enum, Field, Struct, Terminal, Variant};
use syn::{DeriveInput, ItemImpl};

pub mod cst;

pub fn syntactical_impl(parsed: DeriveInput) -> ItemImpl {
    let ident = parsed.ident;

    if parsed
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("Terminal"))
        .is_some()
    {
        let terminal = Terminal { ident };
        let syntactical_impl = terminal.syntactical_impl();

        return syntactical_impl;
    }

    let data = match parsed.data {
        syn::Data::Enum(data) => {
            let variants = data
                .variants
                .into_iter()
                .map(|variant| {
                    let fields = variant
                        .fields
                        .into_iter()
                        .map(|field| Field { ty: field.ty })
                        .collect();
                    Variant {
                        fields,
                        ident: variant.ident,
                    }
                })
                .collect();
            Data::Enum(Enum { ident, variants })
        }
        syn::Data::Struct(data) => {
            let fields = data
                .fields
                .into_iter()
                .map(|field| Field { ty: field.ty })
                .collect();
            Data::Struct(Struct { ident, fields })
        }
        _ => unimplemented!(),
    };

    data.syntactical_impl()
}
