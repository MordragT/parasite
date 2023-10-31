use logos::Logos;
use parasite::*;
use sum::*;

struct Start {
    expr: Expr,
}

struct Expr {
    left: Term,
    op: Token,
    right: Term,
}

struct Term {
    left: Atomic,
    op: Token,
    right: Atomic,
}

enum Atomic {
    Number(Token),
    Expr(Box<Expr>),
}

// uses type definitions for rules
// generates Grammar trait
grammar! {
    #[derive(Logos, Debug)]
    #[logos(skip r"[ \t\n\f]+")]
    enum Token {
        #[regex("[0-9]+", |lex| lex.slice().parse().ok())]
        Number(u32),
        #[token("+")]
        Add,
        #[token("-")]
        Sub,
        #[token("*")]
        Mul,
        #[token("/")]
        Div,
        #[token("(")]
        LPar,
        #[token(")")]
        RPar,
        #[token(";")]
        Semicolon,
    }
        // type Token = Token;
    type Start = Start;
    type K = 1;

    Start: Expr { Semicolon Expr };
    Expr: Term (Add | Sub) Term;
    Term: Atomic (Mul | Div) Atomic;
    Atomic: Number | LPar Expr RPar;
}

/// Generated Grammar trait
// /// shown here but typed out but would be generated
// trait Grammar {
//     type Start;
//     type Token;
//     type Error;

//     fn start(expr: Expr) -> Result<Start, Self::Error>;
//     fn expr(term: Term, either: Either<Add, Sub>, term0: Term) -> Result<Expr, Self::Error>;
//     fn term(atomic: Atomic, either: Either<Mul, Div>, atomic0: Atomic) -> Result<Term, Self::Error>;
//     fn atomic(either: Either<Number, (LPar, Expr, RPar)>) -> Result<Atomic, Self::Error>;
// }

pub struct Ast {
    start: Start,
}

// user must implement the Grammar type himself
impl Grammar for Ast {
    type Error = String;

    fn start(&self, input: (Expr, Vec<(Semicolon, Expr)>)) -> Result<Start, Self::Error> {
        todo!()
    }

    fn expr(&self, input: (Term, Sum2<Add, Sub>, Term)) -> Result<Expr, Self::Error> {
        todo!()
    }

    fn term(&self, input: (Atomic, Sum2<Mul, Div>, Atomic)) -> Result<Term, Self::Error> {
        todo!()
    }

    fn atomic(&self, input: Sum2<Number, (LPar, Expr, RPar)>) -> Result<Atomic, Self::Error> {
        todo!()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tokens = Token::lexer("1 + 10 * 8 / ( 4 - 5 )")
        .collect::<Result<Vec<_>, ()>>()
        .unwrap();

    dbg!(tokens);

    Ok(())
}
