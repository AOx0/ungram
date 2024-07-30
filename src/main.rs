use clap::Parser;
use grammar::GrammarBuilder;

mod args;
mod grammar;
mod lexer;
mod parser;
mod ring;
mod span;
mod token;

fn main() {
    let args = args::Args::parse();

    match args.command {
        args::Command::Lex { path } => {
            let source = std::fs::read_to_string(&path).unwrap();
            let lexer = lexer::Lexer::new(&source);
            let tokens = lexer.collect::<Vec<_>>();
            println!("{tokens:?}");
        }
        args::Command::Parse { path } => {
            let source = std::fs::read_to_string(&path).unwrap();
            let mut parser = parser::Parser::new(&source);
            parser.parse();
            let tree = parser.tree();

            let grammar = GrammarBuilder::new(&source, tree).build();

            println!("{grammar:#?}");
        }
    }
}
