use std::collections::VecDeque;

use super::{
    Alternation, Ast, AstMeta, Group, Item, ItemKind, Nonterminal, NonterminalIndex, Optional,
    Production, Repeat, Rhs, RhsKind, Symbol,
};

pub trait AstInterpreter {
    fn production(&mut self, lhs: NonterminalIndex);
    fn rhs(&mut self, rhs: RhsKind);
    fn alternation(&mut self, ident: Nonterminal);
    fn alternation_item(&mut self, kind: ItemKind, variant: usize);
    fn item(&mut self, kind: ItemKind);
    fn symbol(&mut self, symbol: Symbol);
}

impl Ast {
    pub fn into_machine<G: AstInterpreter>(self, grammar: G) -> AstStateMachine<G, StartState> {
        let Self { productions, meta } = self;

        let queue = VecDeque::from_iter(productions);

        AstStateMachine {
            state: StartState {},
            grammar,
            queue,
            meta,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AstStateMachine<G: AstInterpreter, State> {
    state: State,
    queue: VecDeque<Production>,
    grammar: G,
    meta: AstMeta,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StartState {}

impl<G: AstInterpreter> AstStateMachine<G, StartState> {
    fn start(self) -> AstStateMachine<G, ProductionState> {
        let Self {
            state: _,
            grammar,
            mut queue,
            meta,
        } = self;

        let production = queue.pop_front().expect("Expected atleast one production");
        let state = ProductionState { production };

        AstStateMachine {
            state,
            grammar,
            queue,
            meta,
        }
    }

    pub fn run(self) -> (AstMeta, G) {
        let mut machine = self.start();

        loop {
            match machine
                .items_or_alternations()
                .wait_or_alternation_items()
                .finish()
                .next()
            {
                EndOrProductionMachine::Production(m) => {
                    machine = m;
                }
                EndOrProductionMachine::End(m) => return (m.meta, m.grammar),
            };
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProductionState {
    pub production: Production,
}

impl<G: AstInterpreter> AstStateMachine<G, ProductionState> {
    // pub fn next(self) -> Option<AstStateMachine<G, ProductionState>> {
    //     self.items_or_alternations()
    //         .wait_or_alternation_items()
    //         .finish()
    //         .next()
    // }

    fn items_or_alternations(self) -> AstStateMachine<G, ItemsOrAlternationsState> {
        let Self {
            state,
            mut grammar,
            queue,
            meta,
        } = self;

        let ProductionState { production } = state;
        let Production { lhs, rhs } = production;

        grammar.production(lhs);
        grammar.rhs(rhs.kind());

        let state = match rhs {
            Rhs::Alternations(alternations) => {
                let alternations = alternations
                    .into_iter()
                    .map(|Alternation { ident, items }| AlternationState { ident, items })
                    .collect();
                ItemsOrAlternationsState::Alternations(alternations)
            }
            Rhs::Items(items) => ItemsOrAlternationsState::Items(items),
        };

        AstStateMachine {
            state,
            grammar,
            queue,
            meta,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct AlternationState {
    ident: Nonterminal,
    items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ItemsOrAlternationsState {
    Items(Vec<Item>),
    Alternations(Vec<AlternationState>),
}

impl<G: AstInterpreter> AstStateMachine<G, ItemsOrAlternationsState> {
    fn wait_or_alternation_items(self) -> AstStateMachine<G, WaitOrAlternationItemsState> {
        let Self {
            state,
            mut grammar,
            queue,
            meta,
        } = self;

        let state = match state {
            ItemsOrAlternationsState::Alternations(alternations) => {
                WaitOrAlternationItemsState::AlternationsItems(
                    alternations
                        .into_iter()
                        .map(|AlternationState { ident, items }| {
                            grammar.alternation(ident);
                            items
                        })
                        .collect(),
                )
            }
            ItemsOrAlternationsState::Items(items) => {
                let mut queue = VecDeque::from_iter(items);
                while let Some(item) = queue.pop_front() {
                    let kind = item.kind();
                    match item {
                        Item::Group(Group { items }) => {
                            queue.extend(items);
                            grammar.item(kind);
                        }
                        Item::Optional(Optional { item }) => {
                            queue.push_back(*item);
                            grammar.item(kind);
                        }
                        Item::Repeat(Repeat { item, n }) => {
                            queue.push_back(*item);
                            grammar.item(kind);
                        }
                        Item::Symbol(symbol) => {
                            grammar.symbol(symbol);
                        }
                    }
                }
                WaitOrAlternationItemsState::Wait
            }
        };

        AstStateMachine {
            state,
            grammar,
            queue,
            meta,
        }
    }
}
enum WaitOrAlternationItemsState {
    Wait,
    AlternationsItems(Vec<Vec<Item>>),
}

impl<G: AstInterpreter> AstStateMachine<G, WaitOrAlternationItemsState> {
    fn finish(self) -> AstStateMachine<G, EndProductionState> {
        let Self {
            state,
            mut grammar,
            queue,
            meta,
        } = self;

        match state {
            WaitOrAlternationItemsState::AlternationsItems(alternation_items) => {
                for (variant, items) in alternation_items.into_iter().enumerate() {
                    let mut queue = VecDeque::from_iter(items);
                    while let Some(item) = queue.pop_front() {
                        let kind = item.kind();
                        match item {
                            Item::Group(Group { items }) => {
                                queue.extend(items);
                                grammar.alternation_item(kind, variant);
                            }
                            Item::Optional(Optional { item }) => {
                                queue.push_back(*item);
                                grammar.alternation_item(kind, variant);
                            }
                            Item::Repeat(Repeat { item, n }) => {
                                queue.push_back(*item);
                                grammar.alternation_item(kind, variant);
                            }
                            Item::Symbol(symbol) => {
                                grammar.symbol(symbol);
                            }
                        }
                    }
                }
            }
            WaitOrAlternationItemsState::Wait => (),
        }

        AstStateMachine {
            state: EndProductionState {},
            grammar,
            queue,
            meta,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct EndProductionState {}

impl<G: AstInterpreter> AstStateMachine<G, EndProductionState> {
    fn next(self) -> EndOrProductionMachine<G> {
        let Self {
            state: _,
            grammar,
            mut queue,
            meta,
        } = self;

        if let Some(production) = queue.pop_front() {
            EndOrProductionMachine::Production(AstStateMachine {
                state: ProductionState { production },
                grammar,
                queue,
                meta,
            })
        } else {
            EndOrProductionMachine::End(AstStateMachine {
                state: EndState {},
                grammar,
                queue,
                meta,
            })
        }
    }
}

enum EndOrProductionMachine<G: AstInterpreter> {
    Production(AstStateMachine<G, ProductionState>),
    End(AstStateMachine<G, EndState>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct EndState {}
