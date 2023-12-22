use crate::{
    ast::{self, Group, LookAhead, Nonterminal, NonterminalIndex, Terminal, TerminalIndex},
    collections::OrderedSet,
};

pub mod builder;
pub mod machine;
pub mod validate;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ir {
    rules: OrderedSet<Rule>,
    terminals: OrderedSet<Terminal>,
    nonterminals: OrderedSet<Nonterminal>,
    look_ahead: LookAhead,
    start: RuleIndex,
}
// Instruction ?
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rule {
    lhs: Lhs, // return ??
    rhs: Rhs, // arguments ??
    epsilon: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Lhs {
    Computed,
    Nonterminal(NonterminalIndex),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rhs {
    Alternations(Vec<RuleIndex>),
    Symbols(Vec<SymbolIndex>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SymbolIndex {
    Terminal(TerminalIndex),
    Rule(RuleIndex),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuleIndex(usize);
