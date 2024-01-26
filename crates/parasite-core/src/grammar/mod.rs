use std::{
    any::type_name,
    collections::HashMap,
    ops::{Index, IndexMut},
};

pub mod builder;
pub mod first;
pub mod follow;
pub mod table;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grammar {
    productions: HashMap<TypeName, Rule>,
    start: TypeName,
}

impl Grammar {
    pub fn new(start: TypeName) -> Self {
        Self {
            productions: HashMap::new(),
            start,
        }
    }

    pub fn count(&self) -> usize {
        self.productions.len()
    }

    pub fn insert(&mut self, key: TypeName, rule: Rule) -> Option<Rule> {
        self.productions.insert(key, rule)
    }

    pub fn contains(&self, key: &TypeName) -> bool {
        self.productions.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = TypeName> + '_ {
        self.productions.keys().cloned()
    }

    pub fn get(&self, key: &TypeName) -> Option<&Rule> {
        self.productions.get(key)
    }

    pub fn get_mut(&mut self, key: &TypeName) -> Option<&mut Rule> {
        self.productions.get_mut(key)
    }

    pub fn get_by_type<T>(&self) -> Option<&Rule> {
        let key = TypeName::of::<T>();
        self.get(&key)
    }

    pub fn get_mut_by_type<T>(&mut self) -> Option<&mut Rule> {
        let key = TypeName::of::<T>();
        self.get_mut(&key)
    }
}

impl Index<&TypeName> for Grammar {
    type Output = Rule;

    fn index(&self, index: &TypeName) -> &Self::Output {
        &self.productions[index]
    }
}

impl IndexMut<&TypeName> for Grammar {
    fn index_mut(&mut self, index: &TypeName) -> &mut Self::Output {
        self.productions.get_mut(index).unwrap()
    }
}

impl<T> IndexMut<&TypeName> for HashMap<TypeName, T> {
    fn index_mut(&mut self, index: &TypeName) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}
pub type Rule = HashMap<Id, Symbols>;

impl<T> IndexMut<&Id> for HashMap<Id, T> {
    fn index_mut(&mut self, index: &Id) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

pub type Symbols = Vec<Symbol>;
pub type Terminals = Vec<Terminal>;
pub type Nonterminals = Vec<Nonterminal>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Symbol {
    Nonterminal(Nonterminal),
    Terminal(Terminal),
    Epsilon,
}

impl Symbol {
    pub fn nonterminal(key: TypeName) -> Self {
        Self::Nonterminal(key.into())
    }

    pub fn terminal(key: TypeName) -> Self {
        Self::Terminal(key.into())
    }

    // pub fn epsilon() -> Self {
    //     Self::Terminal(Terminal::epsilon())
    // }
}

impl From<Nonterminal> for Symbol {
    fn from(value: Nonterminal) -> Self {
        Self::Nonterminal(value)
    }
}

impl From<Terminal> for Symbol {
    fn from(value: Terminal) -> Self {
        Self::Terminal(value)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Nonterminal(pub(crate) TypeName);

impl From<TypeName> for Nonterminal {
    fn from(value: TypeName) -> Self {
        Self(value)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Terminal(pub(crate) TypeName);

// impl Terminal {
//     fn epsilon() -> Self {
//         Self(None)
//     }

//     pub fn is_epsilon(&self) -> bool {
//         self.0.is_none()
//     }
// }

impl From<TypeName> for Terminal {
    fn from(value: TypeName) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeName(pub(crate) &'static str);

impl TypeName {
    pub fn of<T: ?Sized>() -> Self {
        let type_name = type_name::<T>();

        TypeName(type_name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(pub(crate) usize);

impl PartialEq<usize> for Id {
    fn eq(&self, other: &usize) -> bool {
        self.0.eq(other)
    }
}
