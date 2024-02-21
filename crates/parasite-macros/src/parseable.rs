use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{
    punctuated::Punctuated, token::Comma, Data, DeriveInput, Expr, ExprClosure, Fields,
    FieldsNamed, FieldsUnnamed, Ident, ItemImpl, Stmt, TypePath, Variant,
};

pub fn parseable_impl(parsed: DeriveInput) -> ItemImpl {
    let ident = parsed.ident;

    let expr = match parsed.data {
        Data::Enum(data) => parse_variants_impl(data.variants, ident.clone()),
        Data::Struct(data) => parse_fields_impl(data.fields, syn::parse_quote!(#ident)),
        _ => unimplemented!(),
    };

    let item_impl: ItemImpl = syn::parse_quote!(
        impl parasite::chumsky::Parseable<'static, char> for #ident {
            type Error = parasite::chumsky::error::Simple<char>;

            fn parser(ctx: &mut parasite::chumsky::Context) -> parasite::chumsky::BoxedParser<'static, char, Self, Self::Error> {
                use parasite::chumsky::Parser;

                if !ctx.contains::<parasite::chumsky::BoxedParser<'static, char, Self, Self::Error>>() {
                    let parser = #expr ;
                    ctx.insert(parser.boxed());
                }

                ctx.get::<parasite::chumsky::BoxedParser<'static, char, Self, Self::Error>>()
                    .unwrap()
                    .clone()

            }
        }
    );

    // println!("{}", item_impl.to_token_stream());

    item_impl
}
// focus on struct for now

type Variants = Punctuated<Variant, Comma>;

fn parse_variants_impl(variants: Variants, ident: Ident) -> Expr {
    let (calls, vars): (Vec<_>, Vec<_>) = variants
        .into_iter()
        .enumerate()
        .map(|(i, variant)| {
            let variant_ident = variant.ident;
            let ty: TypePath = syn::parse_quote!(#ident::#variant_ident);
            let expr = parse_fields_impl(variant.fields, ty);

            let var = Ident::new(&format!("variant_{i}"), Span::call_site());
            let call: Stmt = syn::parse_quote!(let #var = #expr;);

            (call, var)
        })
        .unzip();

    let mut vars_iter = vars.into_iter();

    if let Some(first) = vars_iter.next() {
        syn::parse_quote!({
            #(#calls)*

            #first #( .or ( #vars_iter ) ) *
        })
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
        Fields::Unit => syn::parse_quote!(empty::<Self::Error>().map(|_| #ty)),
    }
}

fn parse_named_fields_impl(fields: FieldsNamed, ty: TypePath) -> Expr {
    let fields = fields.named.into_iter().map(|field| {
        let var = field.ident.unwrap();
        let ty = field.ty;
        (var, ty)
    });

    let (calls, idents): (Vec<_>, Vec<_>) = fields
        .map(|(ident, ty)| {
            let stmt: Stmt = syn::parse_quote!(
                let #ident = <#ty as parasite::chumsky::Parseable<char>>::parser(ctx);
            );
            (stmt, ident)
        })
        .unzip();

    let map_fn = named_map_fn_impl(&idents, &ty);

    let mut idents_iter = idents.into_iter();
    let first: Option<Expr> = idents_iter.next().map(|ident| syn::parse_quote!(#ident));
    let rest = idents_iter.map(|ident| quote!(.then(#ident)));

    if let Some(first) = first {
        syn::parse_quote!({
            #(#calls)*

            #first
                #(#rest)*
                .map( #map_fn )
        })
    } else {
        syn::parse_quote!({
            #(#calls)*

            empty().map(|_| #ty {})
        })
    }
}

fn parse_unnamed_fields_impl(fields: FieldsUnnamed, ty: TypePath) -> Expr {
    let fields = fields.unnamed.into_iter().enumerate().map(|(i, field)| {
        let var = Ident::new(&format!("item_{}", i), Span::call_site());
        let ty = field.ty;
        (var, ty)
    });

    let (calls, idents): (Vec<_>, Vec<_>) = fields
        .map(|(ident, ty)| {
            let stmt: Stmt = syn::parse_quote!(
                let #ident = <#ty as parasite::chumsky::Parseable<char>>::parser(ctx);
            );
            (stmt, ident)
        })
        .unzip();

    let map_fn = unnamed_map_fn_impl(&idents, &ty);

    let mut idents_iter = idents.into_iter();
    let first: Option<Expr> = idents_iter.next().map(|ident| syn::parse_quote!(#ident));
    let rest = idents_iter.map(|ident| quote!(.then(#ident)));

    if let Some(first) = first {
        syn::parse_quote!({
            #(#calls)*

            #first
                #(#rest)*
                .map( #map_fn )
        })
    } else {
        syn::parse_quote!()
    }
}

fn unnamed_map_fn_impl(vars: &Vec<Ident>, ty: &TypePath) -> ExprClosure {
    if vars.len() == 1 {
        let var = &vars[0];

        syn::parse_quote!(
            |#var| #ty ( #var )
        )
    } else {
        let mut vars_iter = vars.iter();
        let first = vars_iter.next().unwrap();
        let tuples = vars_iter.fold(syn::parse_quote!(#first), |accu, var| -> Expr {
            syn::parse_quote!( (#accu, #var) )
        });

        syn::parse_quote!(
            |#tuples| #ty ( #(#vars ,)* )
        )
    }
}

fn named_map_fn_impl(vars: &Vec<Ident>, ty: &TypePath) -> ExprClosure {
    if vars.len() == 1 {
        let var = &vars[0];

        syn::parse_quote!(
            |#var| #ty { #var }
        )
    } else {
        let mut vars_iter = vars.iter();
        let first = vars_iter.next().unwrap();
        let tuples = vars_iter.fold(syn::parse_quote!(#first), |accu, var| -> Expr {
            syn::parse_quote!( (#accu, #var) )
        });

        syn::parse_quote!(
            |#tuples| #ty { #(#vars ,)* }
        )
    }
}
