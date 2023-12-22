use crate::{ast::LookAhead, collections::OrderedSet, ir::RuleIndex};

pub struct Machine {
    start: StateIndex,
    states: OrderedSet<State>,
    transitions: OrderedSet<Transition>,
}

pub struct State {
    rule: RuleIndex,
    look_ahead: LookAhead,
    cursor: Cursor,
}

pub enum Cursor {
    Choice(usize),
    Dot(usize),
}

pub struct Transition {
    from: StateIndex,
    to: StateIndex,
}

pub struct StateIndex(usize);

// fn first() {
//     for rule in rules {
//         State {
//             ruleindex,
//             look_ahead: default,
//             cursor: defaul,
//         }
//     }
// }
