use super::{Ast, Nonterminal};

pub use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rec<T>(pub Box<T>);

pub trait Start: Node {
    fn ast(look_ahead: usize) -> Ast {
        let mut ast = Ast::new(look_ahead);
        Self::item(&mut ast, &Nonterminal("".to_owned()));
        ast
    }
}
pub trait Node {
    fn item(ast: &mut Ast, current: &Nonterminal) -> super::Item;
}

impl<T: Node> Node for Rec<T> {
    fn item(ast: &mut Ast, current: &Nonterminal) -> super::Item {
        T::item(ast, current)
    }
}

impl<T: Node> Node for Option<T> {
    fn item(ast: &mut Ast, current: &Nonterminal) -> super::Item {
        super::Item::Optional(super::Optional {
            item: Box::new(T::item(ast, current)),
        })
    }
}

impl<T: Node, const N: usize> Node for [T; N] {
    fn item(ast: &mut Ast, current: &Nonterminal) -> super::Item {
        super::Item::Repeat(super::Repeat {
            item: Box::new(T::item(ast, current)),
            n: Some(N),
        })
    }
}

impl<T: Node> Node for Vec<T> {
    fn item(ast: &mut Ast, current: &Nonterminal) -> super::Item {
        super::Item::Repeat(super::Repeat {
            item: Box::new(T::item(ast, current)),
            n: None,
        })
    }
}

impl<T: Node> Node for VecDeque<T> {
    fn item(ast: &mut Ast, current: &Nonterminal) -> super::Item {
        super::Item::Repeat(super::Repeat {
            item: Box::new(T::item(ast, current)),
            n: None,
        })
    }
}

impl<T: Node> Node for HashSet<T> {
    fn item(ast: &mut Ast, current: &Nonterminal) -> super::Item {
        super::Item::Repeat(super::Repeat {
            item: Box::new(T::item(ast, current)),
            n: None,
        })
    }
}

impl<K: Node, V: Node> Node for HashMap<K, V> {
    fn item(ast: &mut Ast, current: &Nonterminal) -> super::Item {
        super::Item::Repeat(super::Repeat {
            item: Box::new(super::Item::Group(super::Group {
                items: vec![K::item(ast, current), V::item(ast, current)],
            })),
            n: None,
        })
    }
}

impl Node for () {
    fn item(_: &mut Ast, _: &Nonterminal) -> super::Item {
        super::Item::Group(super::Group { items: Vec::new() })
    }
}

// impl<T: Node> Node for (T) {
//     fn node() -> super::Node {
//         super::Node::Group(super::Group {
//             nodes: vec![T::node(productions)],
//         })
//     }
// }

impl<T: Node, U: Node> Node for (T, U) {
    fn item(ast: &mut Ast, current: &Nonterminal) -> super::Item {
        super::Item::Group(super::Group {
            items: vec![T::item(ast, current), U::item(ast, current)],
        })
    }
}

impl<T: Node, U: Node, V: Node> Node for (T, U, V) {
    fn item(ast: &mut Ast, current: &Nonterminal) -> super::Item {
        super::Item::Group(super::Group {
            items: vec![
                T::item(ast, current),
                U::item(ast, current),
                V::item(ast, current),
            ],
        })
    }
}

impl<T: Node, U: Node, V: Node, W: Node> Node for (T, U, V, W) {
    fn item(ast: &mut Ast, current: &Nonterminal) -> super::Item {
        super::Item::Group(super::Group {
            items: vec![
                T::item(ast, current),
                U::item(ast, current),
                V::item(ast, current),
                W::item(ast, current),
            ],
        })
    }
}
