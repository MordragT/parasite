use parasite_core::grammar::{Grammar, Id, Rule, Symbol};
use quote::ToTokens;
use std::collections::HashMap;
use syn::{
    punctuated::Pair, token::Paren, Attribute, Fields, GenericArgument, Ident, Item, ItemImpl,
    ItemMod, Path, PathArguments, PathSegment, Type, TypeArray, TypePath, TypeTuple,
};

pub const PRIMITIVES: &'static [&'static str] = &[
    "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32",
    "f64", "String", "char", "bool",
];

pub const COLLECTIONS: &'static [&'static str] =
    &["Vec", "VecDeque", "HashSet", "BTreeSet", "LinkedList"];

pub const COLLECTION_MAPS: &'static [&'static str] = &["HashMap", "BTreeMap"];

pub fn parser_impl(input: ItemMod) -> Vec<ItemImpl> {
    let mut start = None;
    let mut terminals = Vec::new();

    if let Some((_, items)) = &input.content {
        let productions = items
            .into_iter()
            .filter_map(|item| match item {
                Item::Enum(item_enum) => {
                    let ident = item_enum.ident.clone();
                    let key = TypeKey::new(ident);

                    if attrs_contains(&item_enum.attrs, "start") {
                        assert!(start.replace(key.clone()).is_none());
                    }

                    if attrs_contains(&item_enum.attrs, "terminal") {
                        terminals.push(key);
                        return None;
                    }

                    Some((key, item.clone()))
                }
                Item::Struct(item_struct) => {
                    let ident = item_struct.ident.clone();
                    let key = TypeKey::new(ident);

                    if attrs_contains(&item_struct.attrs, "start") {
                        assert!(start.replace(key.clone()).is_none());
                    }

                    if attrs_contains(&item_struct.attrs, "terminal") {
                        terminals.push(key);
                        return None;
                    }

                    Some((key, item.clone()))
                }
                _ => None,
            })
            .collect::<HashMap<_, _>>();

        if let Some(start) = start {
            let mut grammar = Grammar::new(start.clone());
            let mut stack = Vec::new();
            populate(start, &productions, &mut grammar, &mut stack, &terminals);

            dbg!(grammar);

            // generate parseable implementations from populated grammar
            todo!()
        }
    }
    Vec::new()
}

fn populate(
    key: TypeKey,
    productions: &HashMap<TypeKey, Item>,
    grammar: &mut Grammar<TypeKey>,
    stack: &mut Vec<TypeKey>,
    terminals: &Vec<TypeKey>,
) -> Symbol<TypeKey> {
    if terminals.contains(&key) {
        return Symbol::terminal(key.clone());
    }

    let symbol = Symbol::nonterminal(key.clone());

    if !visited(&key, grammar, stack) {
        stack.push(key.clone());

        let rule = if let Some(item) = productions.get(&key) {
            match item {
                Item::Enum(enum_item) => enum_item
                    .variants
                    .iter()
                    .enumerate()
                    .map(|(id, variant)| {
                        (
                            Id(id),
                            fields_symbols(&variant.fields, productions, grammar, stack, terminals),
                        )
                    })
                    .collect(),
                Item::Struct(struct_item) => {
                    let mut rule = Rule::new();
                    rule.insert(
                        Id(0),
                        fields_symbols(&struct_item.fields, productions, grammar, stack, terminals),
                    );
                    rule
                }
                _ => unreachable!(),
            }
        } else {
            match key.clone() {
                TypeKey::Array(array) => collection_rule(
                    symbol.clone(),
                    *array.elem,
                    productions,
                    grammar,
                    stack,
                    terminals,
                ),
                TypeKey::Tuple(tuple) => tuple_rule(tuple, productions, grammar, stack, terminals),
                TypeKey::Path(path) => path_rule(path, productions, grammar, stack, terminals),
            }
        };

        grammar.insert(key, rule);
    }

    symbol
}

fn fields_symbols(
    fields: &Fields,
    productions: &HashMap<TypeKey, Item>,
    grammar: &mut Grammar<TypeKey>,
    stack: &mut Vec<TypeKey>,
    terminals: &Vec<TypeKey>,
) -> Vec<Symbol<TypeKey>> {
    let mut symbols = Vec::new();

    for field in fields {
        let key = field.ty.clone().try_into().unwrap();
        let symbol = populate(key, productions, grammar, stack, terminals);
        symbols.push(symbol);
    }

    symbols
}

fn collection_rule(
    collection: Symbol<TypeKey>,
    ty: Type,
    productions: &HashMap<TypeKey, Item>,
    grammar: &mut Grammar<TypeKey>,
    stack: &mut Vec<TypeKey>,
    terminals: &Vec<TypeKey>,
) -> Rule<TypeKey> {
    let ty_key = ty.try_into().unwrap();

    let mut rule = Rule::new();
    rule.insert(
        Id(0),
        vec![
            populate(ty_key, productions, grammar, stack, terminals),
            collection,
        ],
    );
    rule.insert(Id(1), vec![Symbol::Epsilon]);

    rule
}

fn tuple_rule(
    tuple: TypeTuple,
    productions: &HashMap<TypeKey, Item>,
    grammar: &mut Grammar<TypeKey>,
    stack: &mut Vec<TypeKey>,
    terminals: &Vec<TypeKey>,
) -> Rule<TypeKey> {
    let mut rule = Rule::new();
    rule.insert(
        Id(0),
        tuple
            .elems
            .into_iter()
            .map(|ty| {
                let ty_key = ty.try_into().unwrap();
                populate(ty_key, productions, grammar, stack, terminals)
            })
            .collect(),
    );

    rule
}

fn path_rule(
    type_path: TypePath,
    productions: &HashMap<TypeKey, Item>,
    grammar: &mut Grammar<TypeKey>,
    stack: &mut Vec<TypeKey>,
    terminals: &Vec<TypeKey>,
) -> Rule<TypeKey> {
    let mut rule = Rule::new();

    if let Some(ident) = type_path.path.get_ident() {
        if PRIMITIVES.contains(&ident.to_string().as_str()) {
            rule.insert(Id(0), vec![Symbol::terminal(TypeKey::new(ident.clone()))]);
        } else {
            // warning treat ident as terminal
            rule.insert(Id(0), vec![Symbol::terminal(TypeKey::new(ident.clone()))]);
        }
    } else {
        // maybe option, vec etc.
        assert!(type_path.qself.is_none());
        let path = &type_path.path;
        assert!(path.leading_colon.is_none());
        let segments = &path.segments;
        assert_eq!(segments.len(), 1);
        let PathSegment { ident, arguments } = &segments[0];

        let arguments = match arguments {
            PathArguments::AngleBracketed(args) => &args.args,
            _ => panic!(),
        };

        let ident_str = ident.to_string();

        if &ident_str == "Box" {
            assert_eq!(arguments.len(), 1);

            let argument = &arguments[0];
            let child_ty = match argument {
                GenericArgument::Type(ty) => ty.clone(),
                _ => panic!(),
            };
            let key_ty = child_ty.try_into().unwrap();
            rule.insert(
                Id(0),
                vec![populate(key_ty, productions, grammar, stack, terminals)],
            );
        } else if COLLECTIONS.contains(&ident_str.as_str()) {
            assert_eq!(arguments.len(), 1);

            let argument = &arguments[0];
            let child_ty = match argument {
                GenericArgument::Type(ty) => ty.clone(),
                _ => panic!(),
            };

            let symbol = Symbol::nonterminal(TypeKey::Path(type_path));

            return collection_rule(symbol, child_ty, productions, grammar, stack, terminals);
        } else if COLLECTION_MAPS.contains(&ident_str.as_str()) {
            assert_eq!(arguments.len(), 2);

            let elems = arguments
                .pairs()
                .map(|pair| match pair {
                    Pair::End(arg) => Pair::End(if let GenericArgument::Type(ty) = arg {
                        ty.to_owned()
                    } else {
                        panic!()
                    }),
                    Pair::Punctuated(arg, p) => Pair::Punctuated(
                        if let GenericArgument::Type(ty) = arg {
                            ty.to_owned()
                        } else {
                            panic!()
                        },
                        p.to_owned(),
                    ),
                })
                .collect();

            let child_ty = Type::Tuple(TypeTuple {
                paren_token: Paren::default(),
                elems,
            });
            let symbol = Symbol::nonterminal(TypeKey::Path(type_path));

            return collection_rule(symbol, child_ty, productions, grammar, stack, terminals);
        } else {
            panic!()
        }
    }

    rule
}

fn visited(key: &TypeKey, grammar: &Grammar<TypeKey>, stack: &Vec<TypeKey>) -> bool {
    grammar.contains(key) || stack.contains(key)
}

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

fn attrs_contains(attrs: &Vec<Attribute>, ident: &str) -> bool {
    attrs
        .iter()
        .find(|attr| attr.path().is_ident(ident))
        .is_some()
}
