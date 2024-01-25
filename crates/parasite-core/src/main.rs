use parasite_core::grammar::builder::SyntaxAnalyzer;

type Ast = Option<Vec<u8>>;

struct Parser;

impl SyntaxAnalyzer for Parser {
    type Ast = Ast;
}

fn main() {
    let table = Parser::build(2);
    dbg!(table);
}
