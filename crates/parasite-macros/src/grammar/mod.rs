use parasite_core::grammar::Grammar;
use std::collections::HashMap;
use syn::{Attribute, Item, ItemImpl, ItemMod};

use key::TypeKey;
use populate::populate;

pub mod key;
pub mod populate;

pub fn grammar_impl(input: ItemMod) -> Vec<ItemImpl> {
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

            let table = grammar.table(2);

            // generate parseable implementations from populated grammar
            todo!()
        }
    }
    Vec::new()
}

fn attrs_contains(attrs: &Vec<Attribute>, ident: &str) -> bool {
    attrs
        .iter()
        .find(|attr| attr.path().is_ident(ident))
        .is_some()
}
