use parasite_macros::*;

module!(
    mod ast {

        #[begin]
        enum S {
            A((u8, A, u8)),
        }

        enum A {
            S((bool, Box<S>, bool)),
            End,
        }
    }
);

fn main() {}
