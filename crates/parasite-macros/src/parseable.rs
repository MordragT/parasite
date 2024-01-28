use quote::{quote, ToTokens};
use syn::{
    Data, DeriveInput, Expr, ExprClosure, Fields, FieldsNamed, FieldsUnnamed, Ident, ItemImpl, Stmt,
};

pub fn parseable_impl(parsed: DeriveInput) -> ItemImpl {
    let ident = parsed.ident;

    let expr = match parsed.data {
        Data::Enum(data) => todo!(),
        Data::Struct(data) => parse_fields_impl(data.fields),
        _ => unimplemented!(),
    };

    let item_impl: ItemImpl = syn::parse_quote!(
        impl<'a> parasite_core::chumsky::Parseable<'a, &'a str> for #ident {
            type Error = parasite_core::chumsky::error::Cheap<&'a str>;

            fn parse() -> impl parasite_core::chumsky::Parser<&'a str, Self, Error = Self::Error> {
                use parasite_core::chumsky::Parser;

                #expr
            }
        }
    );

    println!("{}", item_impl.to_token_stream());

    item_impl
}

// struct Test<'a> {
//     a: &'a str,
// }

// impl<'a> Test<'a> {
//     fn new(a: &'a str) -> Self {
//         Self { a }
//     }
// }

// impl<'a> Parseable<'a, &'a str> for Test<'a> {
//     type Error = chumsky::error::Cheap<&'a str>;

//     fn parse() -> impl Parser<&'a str, Self, Error = Self::Error> {
//         just("10").map(Test::new).or(just("20").map(Test::new))
//     }
// }

// focus on struct for now

fn parse_fields_impl(fields: Fields) -> Expr {
    match fields {
        Fields::Named(fields) => parse_named_fields_impl(fields),
        Fields::Unnamed(fields) => parse_unnamed_fields_impl(fields),
        // unit struct like struct Phantom;
        Fields::Unit => syn::parse_quote!(|_| Self),
    }
}

fn parse_named_fields_impl(fields: FieldsNamed) -> Expr {
    let (idents, types): (Vec<_>, Vec<_>) = fields
        .named
        .into_iter()
        .map(|field| {
            let var = field.ident.unwrap();
            let ty = field.ty;
            (var, ty)
        })
        .unzip();

    let map_fn: ExprClosure = syn::parse_quote!(
        |(#(#idents ,)*)| Self { #(#idents ,)* }
    );

    let mut calls = types.into_iter();

    let first: Option<Expr> = calls
        .next()
        .map(|ty| syn::parse_quote!(<#ty as parasite_core::chumsky::Parseable>::parse()));
    let rest = calls.map(|ty| quote!(.then(<#ty as parasite_core::chumsky::Parseable>::parse())));

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

fn parse_unnamed_fields_impl(fields: FieldsUnnamed) -> Expr {
    todo!()
}
