use core::fmt;
use std::{
    any::type_name,
    collections::HashMap,
    hash::Hash,
    ops::{Index, IndexMut},
};

use ecow::EcoString;
use owo_colors::{OwoColorize, Style};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grammar {
    pub productions: HashMap<Key, Rule>,
    pub start: Key,
}

impl Grammar {
    pub fn new(start: Key) -> Self {
        Self {
            productions: HashMap::new(),
            start,
        }
    }

    pub fn count(&self) -> usize {
        self.productions.len()
    }

    pub fn insert(&mut self, key: Key, mut rule: Rule) -> Option<Rule> {
        if &key == &self.start {
            for (_, sym) in &mut rule {
                sym.push(Symbol::terminal(Key::new("$")))
            }
        }
        self.productions.insert(key, rule)
    }

    pub fn contains(&self, key: &Key) -> bool {
        self.productions.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = Key> + '_ {
        self.productions.keys().cloned()
    }

    pub fn get(&self, key: &Key) -> Option<&Rule> {
        self.productions.get(key)
    }

    pub fn get_mut(&mut self, key: &Key) -> Option<&mut Rule> {
        self.productions.get_mut(key)
    }
}

impl Grammar {
    pub fn get_by_type<T>(&self) -> Option<&Rule> {
        let key = Key::of::<T>();
        self.get(&key)
    }

    pub fn get_mut_by_type<T>(&mut self) -> Option<&mut Rule> {
        let key = Key::of::<T>();
        self.get_mut(&key)
    }
}

impl fmt::Display for Grammar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", "Grammar".bold())?;

        for (key, rule) in &self.productions {
            let key_style = if key == &self.start {
                Style::new().italic().red()
            } else {
                Style::new().italic()
            };

            let mut rule_iter = rule.iter();

            if let Some((_, rhs)) = rule_iter.next() {
                write!(f, "{} {}", key.style(key_style), ":=".bold())?;
                for sym in rhs {
                    write!(f, " {sym}")?;
                }
            }

            for (_, rhs) in rule_iter {
                write!(f, "\n\t{}", "|".bold())?;
                for sym in rhs {
                    write!(f, " {sym}")?;
                }
            }
            writeln!(f, "\n\t{}", ";".bold())?;
        }

        Ok(())
    }
}

impl Index<&Key> for Grammar {
    type Output = Rule;

    fn index(&self, index: &Key) -> &Self::Output {
        &self.productions[index]
    }
}

impl IndexMut<&Key> for Grammar {
    fn index_mut(&mut self, index: &Key) -> &mut Self::Output {
        self.productions.get_mut(index).unwrap()
    }
}

impl<T> IndexMut<&Key> for HashMap<Key, T> {
    fn index_mut(&mut self, index: &Key) -> &mut Self::Output {
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Symbol {
    Nonterminal(Nonterminal),
    Terminal(Terminal),
    Epsilon,
}

impl Symbol {
    pub fn nonterminal(key: Key) -> Self {
        Self::Nonterminal(key.into())
    }

    pub fn terminal(key: Key) -> Self {
        Self::Terminal(key.into())
    }

    // pub fn epsilon() -> Self {
    //     Self::Terminal(Terminal::epsilon())
    // }

    pub fn into_terminal(self) -> Option<Terminal> {
        match self {
            Self::Terminal(terminal) => Some(terminal),
            _ => None,
        }
    }

    pub fn into_nonterminal(self) -> Option<Nonterminal> {
        match self {
            Self::Nonterminal(nonterminal) => Some(nonterminal),
            _ => None,
        }
    }

    pub fn as_terminal(&self) -> Option<&Terminal> {
        match self {
            Self::Terminal(terminal) => Some(terminal),
            _ => None,
        }
    }

    pub fn as_nonterminal(&self) -> Option<&Nonterminal> {
        match self {
            Self::Nonterminal(nonterminal) => Some(nonterminal),
            _ => None,
        }
    }

    pub fn is_terminal(&self) -> bool {
        self.as_terminal().is_some()
    }

    pub fn is_nonterminal(&self) -> bool {
        self.as_nonterminal().is_some()
    }

    pub fn is_epsilon(&self) -> bool {
        match self {
            Self::Epsilon => true,
            _ => false,
        }
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Epsilon => write!(f, "ε"),
            Self::Nonterminal(nt) => nt.fmt(f),
            Self::Terminal(t) => t.fmt(f),
        }
    }
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Nonterminal(pub Key);

impl From<Key> for Nonterminal {
    fn from(value: Key) -> Self {
        Self(value)
    }
}

impl fmt::Display for Nonterminal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.italic().blue().fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Terminal(pub Key);

impl From<Key> for Terminal {
    fn from(value: Key) -> Self {
        Self(value)
    }
}

impl fmt::Display for Terminal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key(EcoString);

impl Key {
    pub fn new(key: impl Into<EcoString>) -> Self {
        Self(key.into())
    }

    pub fn of<T: ?Sized>() -> Self {
        let type_name = type_name::<T>();

        Key(EcoString::from(type_name))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(pub usize);

impl PartialEq<usize> for Id {
    fn eq(&self, other: &usize) -> bool {
        self.0.eq(other)
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
