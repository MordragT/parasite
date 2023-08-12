use std::collections::HashSet;

use first::FirstBuilder;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    braced, bracketed, parenthesized,
    parse::{discouraged::AnyDelimiter, Parse},
    parse_macro_input,
    token::{Brace, Bracket, Paren},
    Ident, LitInt, Token,
};

mod first;

type LookAheadSet<'a> = HashSet<Vec<&'a Ident>>;

// Macro to define grammar rules and generate the Grammar trait
#[proc_macro]
pub fn grammar(input: TokenStream) -> TokenStream {
    let grammar_definition = parse_macro_input!(input as GrammarDefinition);

    let first = FirstBuilder::new(&grammar_definition);
    let first_sets = first.build();

    dbg!(&first_sets);

    let grammar_trait = grammar_definition.generate_trait();

    grammar_trait.into()
}

// Structure to represent grammar rules
#[derive(Debug)]
struct GrammarDefinition {
    productions: Vec<Production>,
    terminals: Vec<Ident>,
    start: Ident,
    k: u16,
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

    fn is_terminal(&self, ident: &Ident) -> bool {
        self.terminals.contains(ident)
    }

    fn start(&self) -> Option<&Production> {
        self.productions.iter().find(|prod| prod.lhs == self.start)
    }

    fn find_production(&self, ident: &Ident) -> Option<&Production> {
        self.productions.iter().find(|prod| prod.lhs == *ident)
    }
}

impl Parse for GrammarDefinition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut start = None;
        let mut k = 3;
        let mut terminals = Vec::new();

        while input.peek(Token![type]) {
            input.parse::<Token![type]>()?;
            let ident = input.parse::<Ident>()?.to_string();
            input.parse::<Token![=]>()?;

            if ident == "Terminals" {
                terminals.push(input.parse()?);
                while input.peek(Token!(|)) {
                    input.parse::<Token!(|)>()?;
                    let terminal = input.parse()?;
                    terminals.push(terminal);
                }
            } else if ident == "Start" {
                start = Some(input.parse()?);
            } else if ident == "K" {
                let lit = input.parse::<LitInt>()?;
                k = lit.base10_parse::<u16>()?;
            }

            input.parse::<Token![;]>()?;
        }

        let start = match start {
            Some(start) => start,
            None => panic!("A start symbol must be defined"),
        };

        let mut productions = Vec::new();
        while !input.is_empty() {
            let production = input.parse::<Production>()?;
            productions.push(production);
        }

        Ok(Self {
            productions,
            start,
            k,
            terminals,
        })
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
        let len = self.alternations.len();

        if len == 1 {
            self.alternations[0].generate_parameter()
        } else {
            let inner = self
                .alternations
                .iter()
                .map(|alternation| alternation.generate_parameter());

            let variant_name = Ident::new(&format!("Sum{len}"), Span::call_site());

            quote!(#variant_name<#(#inner),*>)
            // match self.alternations.len() {
            //     2 => ,
            //     _ => panic!("More than 16 alternations are not supported"),
            // }
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
