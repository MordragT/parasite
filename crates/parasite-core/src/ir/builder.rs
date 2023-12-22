use std::collections::HashMap;

use super::{Ir, Lhs, Rule};
use crate::{
    ast::{
        machine::AstInterpreter, Ast, AstMeta, ItemKind, Nonterminal, NonterminalIndex, RhsKind,
        Symbol, TerminalIndex,
    },
    collections::{OrderedMap, OrderedSet},
};

enum RuleBuilder {
    Alternations(AlternationsRuleBuilder),
    Symbols(SymbolsRuleBuilder),
}

struct AlternationsRuleBuilder {
    lhs: Lhs,
    rhs: Vec<NonterminalOrRaw>,
    epsilon: bool,
}

enum NonterminalOrRaw {
    Nonterminal(NonterminalIndex),
    Raw(usize),
}

enum SymbolOrRaw {
    Nonterminal(NonterminalIndex),
    Terminal(TerminalIndex),
    Raw(usize),
}

struct SymbolsRuleBuilder {
    lhs: Lhs,
    rhs: Vec<SymbolOrRaw>,
    epsilon: bool,
}

struct Converter {
    stack: Vec<NonterminalIndex>,
    builders: Vec<RuleBuilder>,
}

impl Converter {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            builders: Vec::new(),
        }
    }

    pub fn build(self, meta: AstMeta) -> Ir {
        todo!()
    }
}

impl AstInterpreter for Converter {
    fn production(&mut self, lhs: NonterminalIndex) {
        self.stack.push(lhs);
    }

    fn rhs(&mut self, rhs: RhsKind) {
        let idx = self.stack.pop().unwrap();
        match rhs {
            RhsKind::Alternations => {
                self.builders
                    .push(RuleBuilder::Alternations(AlternationsRuleBuilder {
                        lhs: Lhs::Nonterminal(idx),
                        rhs: Vec::new(),
                        epsilon: false,
                    }))
            }
            RhsKind::Items => self.builders.push(RuleBuilder::Symbols(SymbolsRuleBuilder {
                lhs: Lhs::Nonterminal(idx),
                rhs: Vec::new(),
                epsilon: false,
            })),
        }
    }

    fn alternation(&mut self, ident: Nonterminal) {
        todo!()
    }

    fn alternation_item(&mut self, kind: ItemKind, variant: usize) {
        todo!()
    }

    fn item(&mut self, kind: ItemKind) {
        todo!()
    }

    fn symbol(&mut self, symbol: Symbol) {
        todo!()
    }
}

pub struct IrBuilder {}

impl IrBuilder {
    pub fn build(ast: Ast) -> Ir {
        let (meta, converter) = ast.into_machine(Converter::new()).run();
        converter.build(meta)
    }
}
