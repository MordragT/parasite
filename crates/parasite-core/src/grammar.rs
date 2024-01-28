use std::{
    any::type_name,
    collections::HashMap,
    fmt,
    hash::Hash,
    ops::{Index, IndexMut},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grammar<Key = TypeName>
where
    Key: Clone + Eq + Hash,
{
    pub productions: HashMap<Key, Rule<Key>>,
    pub start: Key,
}

impl<Key: Clone + Eq + Hash> Grammar<Key> {
    pub fn new(start: Key) -> Self {
        Self {
            productions: HashMap::new(),
            start,
        }
    }

    pub fn count(&self) -> usize {
        self.productions.len()
    }

    pub fn insert(&mut self, key: Key, rule: Rule<Key>) -> Option<Rule<Key>> {
        self.productions.insert(key, rule)
    }

    pub fn contains(&self, key: &Key) -> bool {
        self.productions.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = Key> + '_ {
        self.productions.keys().cloned()
    }

    pub fn get(&self, key: &Key) -> Option<&Rule<Key>> {
        self.productions.get(key)
    }

    pub fn get_mut(&mut self, key: &Key) -> Option<&mut Rule<Key>> {
        self.productions.get_mut(key)
    }
}

impl Grammar {
    pub fn get_by_type<T>(&self) -> Option<&Rule> {
        let key = TypeName::of::<T>();
        self.get(&key)
    }

    pub fn get_mut_by_type<T>(&mut self) -> Option<&mut Rule> {
        let key = TypeName::of::<T>();
        self.get_mut(&key)
    }
}

impl<Key: Clone + Eq + Hash> Index<&Key> for Grammar<Key> {
    type Output = Rule<Key>;

    fn index(&self, index: &Key) -> &Self::Output {
        &self.productions[index]
    }
}

impl<Key: Clone + Eq + Hash> IndexMut<&Key> for Grammar<Key> {
    fn index_mut(&mut self, index: &Key) -> &mut Self::Output {
        self.productions.get_mut(index).unwrap()
    }
}

impl<T> IndexMut<&TypeName> for HashMap<TypeName, T> {
    fn index_mut(&mut self, index: &TypeName) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}
pub type Rule<Key = TypeName> = HashMap<Id, Symbols<Key>>;

impl<T> IndexMut<&Id> for HashMap<Id, T> {
    fn index_mut(&mut self, index: &Id) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

pub type Symbols<Key = TypeName> = Vec<Symbol<Key>>;
pub type Terminals<Key = TypeName> = Vec<Terminal<Key>>;
pub type Nonterminals<Key = TypeName> = Vec<Nonterminal<Key>>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Symbol<Key = TypeName> {
    Nonterminal(Nonterminal<Key>),
    Terminal(Terminal<Key>),
    Epsilon,
}

impl<Key> Symbol<Key> {
    pub fn nonterminal(key: Key) -> Self {
        Self::Nonterminal(key.into())
    }

    pub fn terminal(key: Key) -> Self {
        Self::Terminal(key.into())
    }

    // pub fn epsilon() -> Self {
    //     Self::Terminal(Terminal::epsilon())
    // }

    pub fn into_terminal(self) -> Option<Terminal<Key>> {
        match self {
            Self::Terminal(terminal) => Some(terminal),
            _ => None,
        }
    }

    pub fn into_nonterminal(self) -> Option<Nonterminal<Key>> {
        match self {
            Self::Nonterminal(nonterminal) => Some(nonterminal),
            _ => None,
        }
    }

    pub fn is_terminal(&self) -> bool {
        match self {
            Self::Terminal(_) => true,
            _ => false,
        }
    }

    pub fn is_nonterminal(&self) -> bool {
        match self {
            Self::Nonterminal(_) => true,
            _ => false,
        }
    }

    pub fn is_epsilon(&self) -> bool {
        match self {
            Self::Epsilon => true,
            _ => false,
        }
    }
}

impl<Key> From<Nonterminal<Key>> for Symbol<Key> {
    fn from(value: Nonterminal<Key>) -> Self {
        Self::Nonterminal(value)
    }
}

impl<Key> From<Terminal<Key>> for Symbol<Key> {
    fn from(value: Terminal<Key>) -> Self {
        Self::Terminal(value)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Nonterminal<Key = TypeName>(pub Key);

impl<Key> From<Key> for Nonterminal<Key> {
    fn from(value: Key) -> Self {
        Self(value)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Terminal<Key = TypeName>(pub Key);

impl<Key> From<Key> for Terminal<Key> {
    fn from(value: Key) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeName(pub &'static str);

impl TypeName {
    pub fn of<T: ?Sized>() -> Self {
        let type_name = type_name::<T>();

        TypeName(type_name)
    }

    pub fn as_str(&self) -> &'static str {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(pub usize);

impl PartialEq<usize> for Id {
    fn eq(&self, other: &usize) -> bool {
        self.0.eq(other)
    }
}
