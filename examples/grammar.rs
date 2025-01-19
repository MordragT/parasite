#![feature(let_chains)]

// use logos::Logos;
use parasite::*;

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

#[derive(Debug)]
enum Token {
    Number(u32),
    Add,
    Sub,
    Mul,
    Div,
    LPar,
    RPar,
    Semicolon,
}

// uses type definitions for rules
// generates Grammar trait
grammar! {
    enum Token {
        number,
        add,
        sub,
        mul,
        div,
        lpar,
        rpar,
        semicolon,
    }
    // type Token = Token;
    type Start = Start;
    type K = 1;

    Start: Expr { semicolon Expr };
    Expr: Term [ (add | sub) Term ];
    Term: Atomic [ (mul | div) Atomic ];
    Atomic: number | lpar Expr rpar;
}

fn main() {}

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
