use either::Either;
use parasite::grammar;

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

struct Number(f32);

struct Add;

struct Sub;

struct Mul;

struct Div;

struct LPar;

struct RPar;

enum Token {
    Number(Number),
    Add(Add),
    Sub(Sub),
    Mul(Mul),
    Div(Div),
    LPar(LPar),
    RPar(RPar),
}

// uses type definitions for rules
// generates Grammar trait
grammar! {
    Start: Expr;
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

    fn start(&self, input: Expr) -> Result<Start, Self::Error> {}

    fn expr(&self, input: (Term, Either<Add, Sub>, Term)) -> Result<Expr, Self::Error> {}

    fn term(&self, input: (Atomic, Either<Mul, Div>, Atomic)) -> Result<Term, Self::Error> {}

    fn atomic(&self, input: Either<Number, (LPar, Expr, RPar)>) -> Result<Atomic, Self::Error> {}
}

fn main() {
    println!("Hello, world!");
}
