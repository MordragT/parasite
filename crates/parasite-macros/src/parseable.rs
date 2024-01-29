use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{
    punctuated::Punctuated, token::Comma, Data, DeriveInput, Expr, ExprClosure, Fields,
    FieldsNamed, FieldsUnnamed, Ident, Item, ItemImpl, ItemUse, TypePath, Variant,
};

pub fn parseable_impl(parsed: DeriveInput) -> ItemImpl {
    let ident = parsed.ident;

    let expr = match parsed.data {
        Data::Enum(data) => parse_variants_impl(data.variants, ident.clone()),
        Data::Struct(data) => parse_fields_impl(data.fields, syn::parse_quote!(#ident)),
        _ => unimplemented!(),
    };

    let item_impl: ItemImpl = syn::parse_quote!(
        impl<'a> parasite_core::chumsky::Parseable<'a, char> for #ident {
            type Error = parasite_core::chumsky::error::Cheap<char>;

            fn parser() -> impl parasite_core::chumsky::Parser<char, Self, Error = Self::Error> {
                use parasite_core::chumsky::Parser;

                #expr
            }
        }
    );

    println!("{}", item_impl.to_token_stream());

    item_impl
}
// focus on struct for now

type Variants = Punctuated<Variant, Comma>;

fn parse_variants_impl(variants: Variants, ident: Ident) -> Expr {
    let mut variants = variants.into_iter().map(|variant| {
        // let ident = Ident::new(&format!("{ident}::{}", variant.ident), Span::call_site());
        let variant_ident = variant.ident;
        let ty: TypePath = syn::parse_quote!(#ident::#variant_ident);
        parse_fields_impl(variant.fields, ty)
    });

    if let Some(first) = variants.next() {
        // let ors = variants.map(|expr| -> Expr { syn::parse_quote!(or(#expr)) });

        syn::parse_quote!(
            #first #( .or ( #variants ) ) *
        )
    } else {
        // syn::parse_quote!(empty().map(|_| #ident {}))
        panic!()
    }
}

fn parse_fields_impl(fields: Fields, ty: TypePath) -> Expr {
    match fields {
        Fields::Named(fields) => parse_named_fields_impl(fields, ty),
        Fields::Unnamed(fields) => parse_unnamed_fields_impl(fields, ty),
        // unit struct like struct Phantom;
        Fields::Unit => syn::parse_quote!(empty().map(|_| #ty)),
    }
}

fn parse_named_fields_impl(fields: FieldsNamed, ty: TypePath) -> Expr {
    let (vars, types): (Vec<_>, Vec<_>) = fields
        .named
        .into_iter()
        .map(|field| {
            let var = field.ident.unwrap();
            let ty = field.ty;
            (var, ty)
        })
        .unzip();

    let map_fn: ExprClosure = syn::parse_quote!(
        |(#(#vars ,)*)| #ty { #(#vars ,)* }
    );

    let mut calls = types.into_iter();

    let first: Option<Expr> = calls
        .next()
        .map(|ty| syn::parse_quote!(<#ty as parasite_core::chumsky::Parseable<char>>::parser()));
    let rest =
        calls.map(|ty| quote!(.then(<#ty as parasite_core::chumsky::Parseable<char>>::parser())));

    if let Some(first) = first {
        syn::parse_quote!(
            #first
                #(#rest)*
                .map( #map_fn )
        )
    } else {
        syn::parse_quote!(empty().map(|_| #ty {}))
    }
}

fn parse_unnamed_fields_impl(fields: FieldsUnnamed, ty: TypePath) -> Expr {
    let (vars, types): (Vec<_>, Vec<_>) = fields
        .unnamed
        .into_iter()
        .enumerate()
        .map(|(i, field)| {
            let var = Ident::new(&format!("item_{}", i), Span::call_site());
            let ty = field.ty;
            (var, ty)
        })
        .unzip();

    let map_fn: ExprClosure = if vars.len() == 1 {
        let var = &vars[0];

        syn::parse_quote!(
            |#var| #ty ( #var )
        )
    } else {
        syn::parse_quote!(
            |(#(#vars ,)*)| #ty ( #(#vars ,)* )
        )
    };

    let mut calls = types.into_iter();

    let first: Option<Expr> = calls
        .next()
        .map(|ty| syn::parse_quote!(<#ty as parasite_core::chumsky::Parseable<char>>::parser()));
    let rest =
        calls.map(|ty| quote!(.then(<#ty as parasite_core::chumsky::Parseable<char>>::parser())));

    if let Some(first) = first {
        syn::parse_quote!(
            #first
                #(#rest)*
                .map( #map_fn )
        )
    } else {
        syn::parse_quote!()
    }
}
