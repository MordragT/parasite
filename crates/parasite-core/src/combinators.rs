use crate::{
    builder::Syntactical,
    grammar::{Grammar, Id, Rule, Symbol, TypeName},
};

pub struct Rec<T>(pub Box<T>);

impl<T: Syntactical + 'static> Syntactical for Rec<T> {
    fn generate(grammar: &mut Grammar, stack: &mut Vec<TypeName>) -> Symbol {
        let key = TypeName::of::<Self>();

        if !Self::visited(grammar, stack) {
            stack.push(key);

            let mut rule = Rule::new();
            rule.insert(Id(0), vec![T::generate(grammar, stack)]);

            grammar.insert(key, rule);
        }
        Symbol::nonterminal(key)
    }
}

pub struct NonEmptyVec<T>(pub Vec<T>);

impl<T: Syntactical + 'static> Syntactical for NonEmptyVec<T> {
    fn generate(grammar: &mut Grammar, stack: &mut Vec<TypeName>) -> Symbol {
        let key = TypeName::of::<Self>();
        let symbol = Symbol::nonterminal(key);

        if !Self::visited(grammar, stack) {
            stack.push(key);

            let mut rule = Rule::new();
            rule.insert(
                Id(0),
                vec![
                    T::generate(grammar, stack),
                    Vec::<T>::generate(grammar, stack),
                ],
            );

            grammar.insert(key, rule);
        }

        symbol
    }
}
