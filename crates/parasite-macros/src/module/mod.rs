use parasite_core::grammar::Grammar;
use std::collections::HashMap;
use syn::{Attribute, Item, ItemMod};

use key::TypeKey;
use populate::populate;

pub mod key;
pub mod populate;

pub fn module_check(input: &mut ItemMod) {
    let mut start = None;
    let mut terminals = Vec::new();

    if let Some((_, items)) = &mut input.content {
        let productions = items
            .into_iter()
            .filter_map(|item| match item {
                Item::Enum(item_enum) => {
                    let ident = item_enum.ident.clone();
                    let key = TypeKey::new(ident);

                    if let Some(pos) = attrs_find(&item_enum.attrs, "begin") {
                        assert!(start.replace(key.clone()).is_none());
                        item_enum.attrs.remove(pos);
                    }

                    if let Some(pos) = attrs_find(&item_enum.attrs, "terminal") {
                        terminals.push(key);
                        item_enum.attrs.remove(pos);
                        return None;
                    }

                    Some((key, item.clone()))
                }
                Item::Struct(item_struct) => {
                    let ident = item_struct.ident.clone();
                    let key = TypeKey::new(ident);

                    if let Some(pos) = attrs_find(&item_struct.attrs, "begin") {
                        assert!(start.replace(key.clone()).is_none());
                        item_struct.attrs.remove(pos);
                    }

                    if let Some(pos) = attrs_find(&item_struct.attrs, "terminal") {
                        terminals.push(key);
                        item_struct.attrs.remove(pos);
                        return None;
                    }

                    Some((key, item.clone()))
                }
                _ => None,
            })
            .collect::<HashMap<_, _>>();

        if let Some(start) = start {
            let mut grammar = Grammar::new(start.clone().into());
            let mut stack = Vec::new();
            populate(start, &productions, &mut grammar, &mut stack, &terminals);

            println!("{grammar}");

            let table = grammar.table(2);
            println!("{table}");

            // TODO do checks so that it can be verified that grammar is valid
            // TODO make lookahead as attribute into the proc macro
        }
    }
}

fn attrs_find(attrs: &Vec<Attribute>, ident: &str) -> Option<usize> {
    attrs
        .iter()
        .position(|attr: &Attribute| attr.path().is_ident(ident))
}
