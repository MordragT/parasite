use proc_macro2::Ident;

use crate::{Alternation, Alternations, Factor, GrammarDefinition, LookAheadSet, Production};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
};

#[derive(Debug, Clone)]
pub struct FirstSets<'a> {
    k: usize,
    sets: HashMap<&'a Ident, FirstSet<'a>>,
}

impl<'a> FirstSets<'a> {
    pub fn new(k: usize) -> Self {
        Self {
            k,
            sets: HashMap::new(),
        }
    }

    pub fn insert(&mut self, ident: &'a Ident, set: FirstSet<'a>) {
        self.sets.insert(ident, set);
    }

    pub fn get(&self, ident: &Ident, k: usize) -> Option<FirstSet<'a>> {
        if let Some(set) = self.sets.get(ident) {
            if k != self.k {
                let mut new_set = FirstSet::new();

                for item in set {
                    let mut item = item.clone();
                    item.truncate(k);
                    new_set.insert(item);
                }

                Some(new_set)
            } else {
                Some(set.clone())
            }
        } else {
            None
        }
    }
}

pub type FirstSet<'a> = HashSet<Vec<&'a Ident>>;

// create prodmap from grammar for better performance
#[derive(Clone, Debug)]
pub struct FirstBuilder<'a> {
    grammar: &'a GrammarDefinition,
}

impl<'a> FirstBuilder<'a> {
    pub(crate) fn new(grammar: &'a GrammarDefinition) -> Self {
        Self { grammar }
    }

    pub(crate) fn build(&'a self) -> FirstSets<'a> {
        let mut cache = FirstSets::new(self.k());
        let start = self.grammar.start().unwrap();
        self.production(start, self.k(), &mut cache);
        cache
    }

    fn k(&self) -> usize {
        self.grammar.k as usize
    }

    fn production(
        &'a self,
        production: &'a Production,
        k: usize,
        cache: &mut FirstSets<'a>,
    ) -> FirstSet<'a> {
        let ident = &production.lhs;

        if let Some(set) = cache.get(ident, k) {
            set
        } else {
            let set = self.alternations(&production.alternations, self.k(), cache);
            cache.insert(ident, set);
            cache.get(ident, k).unwrap()
        }
    }

    fn alternations(
        &'a self,
        alternations: &'a Alternations,
        k: usize,
        cache: &mut FirstSets<'a>,
    ) -> FirstSet<'a> {
        let mut set = FirstSet::new();

        if k > 0 {
            for alternation in &alternations.alternations {
                let mut alternation_set = FirstSet::new();

                for (i, factor) in alternation.factors.iter().take(k).enumerate() {
                    alternation_set = match factor {
                        Factor::Group(alternations) => {
                            self.group(alternations, k - i, alternation_set, cache)
                        }
                        Factor::Repeat(alternations) => {
                            self.repeat(alternations, k - i, alternation_set, cache)
                        }
                        Factor::Optional(alternations) => {
                            self.optional(alternations, k - i, alternation_set, cache)
                        }
                        Factor::Symbol(ident) => self.symbol(ident, k - i, alternation_set, cache),
                    };
                }
                set = set.union(&alternation_set).cloned().collect();
            }
        }

        set
    }

    fn group(
        &'a self,
        alternations: &'a Alternations,
        k: usize,
        set: FirstSet<'a>,
        cache: &mut FirstSets<'a>,
    ) -> FirstSet<'a> {
        let group_set = self.alternations(alternations, k, cache);
        let mut new_set = FirstSet::new();

        if set.is_empty() {
            new_set = group_set;
        } else {
            for group_item in group_set {
                for mut item in set.clone() {
                    item.append(&mut group_item.clone());
                    new_set.insert(item);
                }
            }
        }

        new_set
    }

    fn repeat(
        &'a self,
        alternations: &'a Alternations,
        k: usize,
        set: FirstSet<'a>,
        cache: &mut FirstSets<'a>,
    ) -> FirstSet<'a> {
        let repeat_set = self.alternations(alternations, k, cache);
        let mut new_set = FirstSet::new();

        if set.is_empty() {
            for repeat_item in repeat_set {
                let mut repeat_item = repeat_item.repeat(k);
                repeat_item.truncate(k);
                new_set.insert(repeat_item);
            }
        } else {
            for repeat_item in repeat_set {
                for mut item in set.clone() {
                    let mut repeat_item = repeat_item.repeat(k);
                    repeat_item.truncate(k);
                    item.append(&mut repeat_item);
                    new_set.insert(item);
                }
            }
        }

        new_set
    }

    fn optional(
        &'a self,
        alternations: &'a Alternations,
        k: usize,
        set: FirstSet<'a>,
        cache: &mut FirstSets<'a>,
    ) -> FirstSet<'a> {
        let optional_set = self.alternations(alternations, k, cache);
        let mut new_set = FirstSet::new();

        if set.is_empty() {
            new_set = optional_set;
        } else {
            for optional_item in optional_set {
                for mut item in set.clone() {
                    item.append(&mut optional_item.clone());
                    new_set.insert(item);
                }
            }
        }

        new_set
    }

    fn symbol(
        &'a self,
        ident: &'a Ident,
        k: usize,
        mut set: FirstSet<'a>,
        cache: &mut FirstSets<'a>,
    ) -> FirstSet<'a> {
        let mut new_set = FirstSet::new();

        if self.grammar.is_terminal(ident) {
            if set.is_empty() {
                set.insert(vec![ident]);
                set
            } else {
                for mut item in set {
                    item.push(ident);
                    new_set.insert(item);
                }

                new_set
            }
        } else {
            let production = self.grammar.find_production(ident).unwrap();
            let prod_set = self.production(production, k, cache);

            if set.is_empty() {
                prod_set
            } else {
                for mut item in set {
                    for mut prod_item in prod_set.clone() {
                        item.append(&mut prod_item);
                    }
                }

                new_set
            }
        }
    }
}
