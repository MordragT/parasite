#![feature(let_chains)]

use logos::Logos;
use parasite::*;
use sum::*;

fn productions<Root: Node>() {
    let mut productions = vec![];



    if Root::Production::is_leaf() {
        std::any::type_name::<Root::Production>();
    }
}


struct Start {
    expr: Expr,
}
// is derived
impl Node for Start {
    type Child: impl Node = Expr;

    fn populate(productions: &mut Vec<&' static str>) {
        std::any::type_name::<Root::Production>();

    }

    fn id_path(&mut Vec<>) {

    }
}

struct Expr {
    left: Term,
    op: Token,
    right: Term,
}

impl Node for Expr {
    type Production: impl Node = (Term, Sum2<Add, Sub>, Term);
}

struct Term {
    left: Atomic,
    op: Token,
    right: Atomic,
}

impl Node for Term {
    type Production: impl Node = (Atomic, Sum2<Mul, Div>, Atomic);
}

enum Atomic {
    Number(Token),
    Expr(Box<Expr>),
}

impl Node for Atomic {
    type Production: impl Node = Sum2<Number, Box<Expr>>;
}

impl Node for Number {
    fn is_leaf() -> bool {
        true
    }
}

impl Node for () {
    fn is_leaf() -> bool {
        true
    }
}


// uses type definitions for rules
// generates Grammar trait
// grammar! {
//     %start = Start;
//     %lookahead = 1;

//     #[derive(Logos, Debug)]
//     #[logos(skip r"[ \t\n\f]+")]
//     Token ::=
//         #[regex("[0-9]+", |lex| lex.slice().parse().ok())]
//         Number(u32) |
//         #[token("+")]
//         Add |
//         #[token("-")]
//         Sub |
//         #[token("*")]
//         Mul |
//         #[token("/")]
//         Div |
//         #[token("(")]
//         LPar |
//         #[token(")")]
//         RPar |
//         #[token(";")]
//         Semicolon;
    

//     Start ::= Expr { Semicolon Expr };
//     Expr ::= Term [ (Add | Sub) Term ];
//     Term ::= Atomic [ (Mul | Div) Atomic ];
//     Atomic ::= Number | LPar Expr RPar;
// }


grammar! {
    type Token = Token;
    type Start = Start;
    type K = 1;

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

    struct Start {
        expr: Expr,
        tail: Vec<(Semicolon, Expr)>
    }

    struct Expr {
        term: Term
        tail: Option<(Sum2<Add, Sub>, Term)>
    }

    struct Term {
         atomic: Atomic,
         tail: Option<(Sum2<Mul, Div>, Atomic)>,
    }

    enum Atomic {
        Number(Number),
        Tail {
            lpar: LPar,
            expr: Box<Expr>,
            rpar: RPar
        }
    }
}

grammar! {
    %start = Start
    %lookahead = 1

    type Token = Token;
    type Start = Start;
    type K = 1;

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

    Start ::= expr: Expr
        , { semi: Semicolon, expr: Expr }
        ;

    Expr ::= term: Term
        , [ ( add: Add | sub: Sub ), term: Term ]
        ;

    Term ::= atomic: Atomic
         ,  [ ( mul: Mul | div: Div ), atomic: Atomic ]
         ;

    Atomic ::= number: Number
        | (lpar: Lpar, expr: Expr, rpar: Rpar)
        ;
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

// trait Factor
// impl Factor for Vec, Option, Box, Tuple, Array

pub struct Ast {
    start: Start,
}

// user must implement the Grammar type himself
impl Grammar for Ast {
    type Error = String;

    fn start(&self, input: (Expr, Vec<(Semicolon, Expr)>)) -> Result<Start, Self::Error> {
        todo!()
    }

    fn expr(&self, input: (Term, Option<(Sum2<Add, Sub>, Term)>)) -> Result<Expr, Self::Error> {
        todo!()
    }

    fn term(&self, _: (Atomic, Option<(Sum2<Mul, Div>, Atomic)>)) -> Result<Term, Self::Error> {
        todo!()
    }

    fn atomic(&self, input: Sum2<Number, (LPar, Expr, RPar)>) -> Result<Atomic, Self::Error> {
        todo!()
    }

    /*
    fn parse_start(tokens: &mut Vec<Token>) -> Result<Start> {
        match tokens.peek().kind {
            first(Start) =>
            first
        }
        let expr = Self::parse_expr(tokens)
    }
     */
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tokens = Token::lexer("(1 + (10 * (8 / ( 4 - 5 ))))")
        .collect::<Result<Vec<_>, ()>>()
        .unwrap();

    let mut parser = Parser::new(tokens);
    parser.parse();

    Ok(())
}

// Productions
// ===============
// 0(Start)	: Expr 13
// 1(Expr)	: Term 9 Term
// 2(Term)	: Atomic 6 Atomic
// 3(Atomic)	: 4
// 	| 5
// 4(group)	: "Number"
// 5(group)	: "LPar" Expr "RPar"
// 6(group)	: 7
// 	| 8
// 7(group)	: "Mul"
// 8(group)	: "Div"
// 9(group)	: 10
// 	| 11
// 10(group)	: "Add"
// 11(group)	: "Sub"
// 12(group)	: "Semicolon" Expr
// 13(repeat)	: 12 13
// 	|
