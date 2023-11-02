use syn::ItemFn;

use super::Grammar;

pub struct Table {
    // (rule id, terminal id, rule id)
    table: Vec<(usize, usize)>,
}

impl Grammar {
    fn table(&self) -> Table {
        todo!()
    }

    pub fn parse_decl(&self) -> ItemFn {
        let token_ident = &self.token.ident;
        let start_ident = &self.start;
        let k = self.k;

        syn::parse_quote!(
            fn parse(tokens: impl IntoIterator<Item = #token_ident>, grammar: impl Grammar) -> Result<#start_ident, String> {
                // use std::collections::VecDeque;

                let mut prod_idx = 0;
                let mut idx = 0;
                let tokens = tokens.into_iter().collect::<Vec<_>>();
                let table = todo!();

                loop {
                    let token = tokens[idx];
                    let peek = tokens[(idx + 1)..(#k + 1)];


                }
            }
        )
    }
}

// const K: usize = 3;

// pub struct StateMachine<State, G: Grammar> {
//     index: usize,
//     tokens: Vec<Token>,
//     grammar: G,
//     state: State,
// }

// impl<G: Grammar> StateMachine<Prod0, G> {
//     pub fn parse(mut self) -> Result<Prod1, String> {
//         if self.tokens[self.index..K].starts_with(table.get::<Prod1>()) {
//             self.state = Prod1;
//             Ok(self)
//         }
//         Err...
//     }
// }

// impl<G: Grammar> StateMachine<Prod1, G> {
//     pub fn parse(mut self) -> Result<Prod2, String> {
//         self.state = Prod2;
//         Ok(self)
//     }
// }

// impl<G: Grammar> StateMachine<Prod2, G> {
//     pub fn parse(mut self) -> Result<Prod2, String> {
//         self.state = Prod2;
//         Ok(self)
//     }
// }

// pub enum Item {
//     Terminal(usize),
//     Production(usize),
// }

// pub struct Table {
//     productions: [Vec<Item>; 13],
//     terminals: [Token; Token::count],
// }

/*
// queue containing the current productions
let queue = [0];

// index into tokens
let token_index = 0;

while let Some(prod_id) = queue.pop_front() {
    let (next, next_token_index) = table.next(tokens[token_index..k], prod_id);
    queue.push_back(next);
    token_index = next_token_index;
}
*/
