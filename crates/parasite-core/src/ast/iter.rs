use super::{
    Alternation, Ast, AstMeta, Group, Item, ItemKind, Nonterminal, NonterminalIndex, Optional,
    Production, Repeat, Rhs, RhsKind, Symbol,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IterItem {
    Production { lhs: NonterminalIndex },
    Rhs { kind: RhsKind },
    Alternation { ident: Nonterminal },
    AlternationItem { kind: ItemKind },
    AlternationSymbol { symbol: Symbol },
    Item { kind: ItemKind },
    Symbol { symbol: Symbol },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum StackItem {
    Production(Production),
    Rhs(Rhs),
    Alternation(Alternation),
    AlternationItem(Item),
    AlternationSymbol(Symbol),
    Item(Item),
    Symbol(Symbol),
}

impl IntoIterator for Ast {
    type IntoIter = AstIterator;
    type Item = IterItem;

    fn into_iter(self) -> Self::IntoIter {
        let Self { productions, meta } = self;

        let mut stack = Vec::new();

        for production in productions {
            stack.push(StackItem::Production(production));
        }

        AstIterator { stack, meta }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AstIterator {
    stack: Vec<StackItem>,
    pub meta: AstMeta,
}

impl Iterator for AstIterator {
    type Item = IterItem;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.stack.pop() {
            match item {
                StackItem::Production(Production { lhs, rhs }) => {
                    self.stack.push(StackItem::Rhs(rhs));
                    Some(IterItem::Production { lhs })
                }
                StackItem::Rhs(rhs) => match rhs {
                    Rhs::Items(items) => {
                        for item in items {
                            self.stack.push(StackItem::Item(item));
                        }
                        Some(IterItem::Rhs {
                            kind: RhsKind::Items,
                        })
                    }
                    Rhs::Alternations(alternations) => {
                        for alternation in alternations {
                            self.stack.push(StackItem::Alternation(alternation));
                        }
                        Some(IterItem::Rhs {
                            kind: RhsKind::Alternations,
                        })
                    }
                },
                StackItem::Alternation(Alternation { ident, items }) => {
                    for item in items {
                        self.stack.push(StackItem::AlternationItem(item));
                    }
                    Some(IterItem::Alternation { ident })
                }
                StackItem::AlternationItem(item) => {
                    let kind = item.kind();
                    match item {
                        Item::Group(Group { items }) => {
                            for item in items {
                                self.stack.push(StackItem::AlternationItem(item));
                            }
                        }
                        Item::Repeat(Repeat { item, n }) => {
                            self.stack.push(StackItem::AlternationItem(*item))
                        }
                        Item::Optional(Optional { item }) => {
                            self.stack.push(StackItem::AlternationItem(*item))
                        }
                        Item::Symbol(sym) => self.stack.push(StackItem::AlternationSymbol(sym)),
                    }

                    Some(IterItem::AlternationItem { kind })
                }
                StackItem::Item(item) => {
                    let kind = item.kind();
                    match item {
                        Item::Group(Group { items }) => {
                            for item in items {
                                self.stack.push(StackItem::Item(item));
                            }
                        }
                        Item::Repeat(Repeat { item, n }) => self.stack.push(StackItem::Item(*item)),
                        Item::Optional(Optional { item }) => {
                            self.stack.push(StackItem::Item(*item))
                        }
                        Item::Symbol(sym) => self.stack.push(StackItem::Symbol(sym)),
                    }

                    Some(IterItem::Item { kind })
                }
                StackItem::AlternationSymbol(symbol) => {
                    Some(IterItem::AlternationSymbol { symbol })
                }
                StackItem::Symbol(symbol) => Some(IterItem::Symbol { symbol }),
            }
        } else {
            None
        }
    }
}
