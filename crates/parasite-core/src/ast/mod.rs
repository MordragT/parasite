//! The abstract syntax tree of the grammar.
//! Is independent of the grammar notation,
//! so that it can be used in other contexts.

use crate::collections::OrderedSet;

pub mod iter;
pub mod machine;
pub mod nodes;
pub mod validate;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AstMeta {
    pub nonterminals: OrderedSet<Nonterminal>,
    pub terminals: OrderedSet<Terminal>,
    pub look_ahead: LookAhead,
    pub start: ProductionIndex,
}

impl AstMeta {
    pub fn new(look_ahead: usize) -> Self {
        Self {
            start: ProductionIndex(0),
            terminals: OrderedSet::new(),
            nonterminals: OrderedSet::new(),
            look_ahead: LookAhead(look_ahead),
        }
    }
    pub fn get_terminal(&self, idx: TerminalIndex) -> &Terminal {
        self.terminals.get(idx.0).unwrap()
    }

    pub fn insert_terminal(&mut self, terminal: Terminal) -> TerminalIndex {
        let idx = self.terminals.insert(terminal);
        TerminalIndex(idx)
    }

    pub fn find_terminal_idx(&self, needle: &Terminal) -> Option<TerminalIndex> {
        match self.terminals.binary_search(needle) {
            Ok(idx) => Some(TerminalIndex(idx)),
            Err(_) => None,
        }
    }

    pub fn get_nonterminal(&self, idx: NonterminalIndex) -> &Nonterminal {
        self.nonterminals.get(idx.0).unwrap()
    }

    pub fn insert_nonterminal(&mut self, nonterminal: Nonterminal) -> NonterminalIndex {
        let idx = self.nonterminals.insert(nonterminal);
        NonterminalIndex(idx)
    }

    pub fn find_nonterminal_idx(&self, needle: &Nonterminal) -> Option<NonterminalIndex> {
        match self.nonterminals.binary_search(needle) {
            Ok(idx) => Some(NonterminalIndex(idx)),
            Err(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ast {
    pub productions: OrderedSet<Production>,
    pub meta: AstMeta,
}

impl Ast {
    pub fn new(look_ahead: usize) -> Self {
        Self {
            productions: OrderedSet::new(),
            meta: AstMeta::new(look_ahead),
        }
    }

    pub fn insert_terminal(&mut self, terminal: Terminal) -> TerminalIndex {
        self.meta.insert_terminal(terminal)
    }

    pub fn find_terminal_idx(&self, needle: &Terminal) -> Option<TerminalIndex> {
        self.meta.find_terminal_idx(needle)
    }

    pub fn get_terminal(&self, idx: TerminalIndex) -> &Terminal {
        self.meta.get_terminal(idx)
    }

    pub fn insert_production(&mut self, lhs: Nonterminal, rhs: Rhs) -> ProductionIndex {
        let lhs = self.meta.insert_nonterminal(lhs);
        let idx = self.productions.insert(Production { lhs, rhs });

        ProductionIndex(idx)
    }

    pub fn find_production_idx(&self, needle: &Nonterminal) -> Option<ProductionIndex> {
        if let Some(nonterminal_idx) = self.meta.find_nonterminal_idx(needle) {
            match self
                .productions
                .binary_search_by(|production| production.lhs.cmp(&nonterminal_idx))
            {
                Ok(idx) => Some(ProductionIndex(idx)),
                Err(_) => None,
            }
        } else {
            None
        }
    }

    pub fn get_production(&self, idx: ProductionIndex) -> &Production {
        self.productions.get(idx.0).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Production {
    pub lhs: NonterminalIndex,
    pub rhs: Rhs,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rhs {
    Alternations(Vec<Alternation>),
    Items(Vec<Item>),
}

impl Rhs {
    pub fn kind(&self) -> RhsKind {
        match self {
            Self::Alternations(_) => RhsKind::Alternations,
            Self::Items(_) => RhsKind::Items,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RhsKind {
    Alternations,
    Items,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Alternation {
    pub ident: Nonterminal,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Item {
    Optional(Optional),
    Repeat(Repeat),
    Group(Group),
    Symbol(Symbol),
}

impl Item {
    pub fn kind(&self) -> ItemKind {
        match self {
            Self::Optional(_) => ItemKind::Optional,
            Self::Repeat(_) => ItemKind::Repeat,
            Self::Group(_) => ItemKind::Group,
            Self::Symbol(_) => ItemKind::Symbol,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ItemKind {
    Optional,
    Repeat,
    Group,
    Symbol,
}

impl From<Optional> for Item {
    fn from(value: Optional) -> Self {
        Self::Optional(value)
    }
}

impl From<Repeat> for Item {
    fn from(value: Repeat) -> Self {
        Self::Repeat(value)
    }
}

impl From<Group> for Item {
    fn from(value: Group) -> Self {
        Self::Group(value)
    }
}

impl From<Symbol> for Item {
    fn from(value: Symbol) -> Self {
        Self::Symbol(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Symbol {
    Terminal(TerminalIndex),
    Nonterminal(ProductionIndex),
    Recursive(Nonterminal),
    Eof,
}

impl From<TerminalIndex> for Symbol {
    fn from(value: TerminalIndex) -> Self {
        Self::Terminal(value)
    }
}

impl From<ProductionIndex> for Symbol {
    fn from(value: ProductionIndex) -> Self {
        Self::Nonterminal(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Optional {
    pub item: Box<Item>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Repeat {
    pub item: Box<Item>,
    pub n: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Group {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Nonterminal(String);

impl Nonterminal {
    pub fn new(ident: &str) -> Self {
        Self(ident.to_owned())
    }
}

impl From<&str> for Nonterminal {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<String> for Nonterminal {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Terminal(String);

impl Terminal {
    pub fn new(ident: &str) -> Self {
        Self(ident.to_owned())
    }
}

impl From<&str> for Terminal {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<String> for Terminal {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LookAhead(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TerminalIndex(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonterminalIndex(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProductionIndex(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AlternationIndex {
    production: usize,
    variant: usize,
}
