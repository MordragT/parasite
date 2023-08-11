use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    braced, bracketed, parenthesized,
    parse::{discouraged::AnyDelimiter, Parse},
    parse_macro_input,
    token::{Brace, Bracket, Paren},
    Ident, Token,
};

// Macro to define grammar rules and generate the Grammar trait
#[proc_macro]
pub fn grammar(input: TokenStream) -> TokenStream {
    let grammar_definition = parse_macro_input!(input as GrammarDefinition);
    let grammar_trait = grammar_definition.generate_trait();

    grammar_trait.into()
}

// Structure to represent grammar rules
#[derive(Debug)]
struct GrammarDefinition {
    productions: Vec<Production>,
}

impl GrammarDefinition {
    fn generate_trait(&self) -> proc_macro2::TokenStream {
        let functions = self
            .productions
            .iter()
            .map(|production| production.generate_function());

        quote! {
            trait Grammar {
                type Error;

                #( #functions )*
            }
        }
    }
}

impl Parse for GrammarDefinition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut productions = Vec::new();
        while !input.is_empty() {
            let production = input.parse::<Production>()?;
            productions.push(production);
        }
        Ok(Self { productions })
    }
}

#[derive(Debug)]
struct Production {
    lhs: Ident,
    /// split by |
    alternations: Alternations,
}

impl Production {
    fn generate_function(&self) -> proc_macro2::TokenStream {
        let name = Ident::new(&self.lhs.to_string().to_lowercase(), Span::call_site());
        let ret = &self.lhs;
        let parameter = self.alternations.generate_parameter();

        quote! { fn #name(&self, input: #parameter) -> Result<#ret, Self::Error>; }
    }
}

impl Parse for Production {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lhs = input.parse()?;
        let _colon = input.parse::<Token![:]>()?;
        let alternations = input.parse()?;
        let _semi = input.parse::<Token![;]>()?;

        Ok(Self { lhs, alternations })
    }
}

#[derive(Debug)]
struct Alternations {
    alternations: Vec<Alternation>,
}

impl Alternations {
    fn generate_parameter(&self) -> proc_macro2::TokenStream {
        if self.alternations.len() == 1 {
            self.alternations[0].generate_parameter()
        } else {
            let inner = self
                .alternations
                .iter()
                .map(|alternation| alternation.generate_parameter());
            match self.alternations.len() {
                2 => quote!(Either<#(#inner),*>),
                _ => panic!("More than 16 alternations are not supported"),
            }
        }
    }
}

impl Parse for Alternations {
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
struct Alternation {
    /// split by ' '
    factors: Vec<Factor>,
}

impl Alternation {
    fn generate_parameter(&self) -> proc_macro2::TokenStream {
        if self.factors.len() == 1 {
            self.factors[0].generate_parameter()
        } else {
            let inner = self
                .factors
                .iter()
                .map(|factor| factor.generate_parameter());
            quote!(
                (#(#inner),*)
            )
        }
    }
}

impl Parse for Alternation {
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

#[derive(Debug)]
enum Factor {
    // '(' Alternations ')'
    Group(Alternations),
    // '{' Alternations '}'
    Repeat(Alternations),
    // '[' Alternations ']'
    Optional(Alternations),
    Symbol(Ident),
}

impl Factor {
    fn generate_parameter(&self) -> proc_macro2::TokenStream {
        match self {
            Self::Group(alternations) => {
                let inner = alternations.generate_parameter();
                quote!(#inner)
            }
            Self::Repeat(alternations) => {
                let inner = alternations.generate_parameter();

                quote!(Vec<#inner>)
            }
            Self::Optional(alternations) => {
                let inner = alternations.generate_parameter();

                quote!(Option<#inner>)
            }
            Self::Symbol(ident) => {
                quote!(#ident)
            }
        }
    }
}

impl Parse for Factor {
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
