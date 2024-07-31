#![feature(iter_map_windows)]
#![feature(let_chains)]

use std::collections::HashSet;

use clap::Parser;

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
        args::Command::Tree { path } => {
            let source = std::fs::read_to_string(&path).unwrap();
            let mut parser = parser::Parser::new(&source);
            parser.parse();
            let tree = parser.tree();

            println!("{tree:#?}");
        }
        args::Command::Parse { path } => {
            let source = std::fs::read_to_string(&path).unwrap();
            let mut parser = parser::Parser::new(&source);
            parser.parse();
            let grammar = grammar::GrammarBuilder::new(&source, parser.tree()).build();

            println!("{grammar:#?}");
        }
        args::Command::First { path, non_terminal } => {
            let source = std::fs::read_to_string(&path).unwrap();
            let mut parser = parser::Parser::new(&source);
            parser.parse();
            let tree = parser.tree();

            let grammar = grammar::GrammarBuilder::new(&source, tree).build();

            if let Some(nt) = non_terminal {
                let first = grammar.first_set(&nt);
                println!("{first:?}");
            } else {
                for nt in grammar.non_terminals() {
                    let first = grammar.first_set(&nt);
                    println!("{nt}: {first:?}");
                }
            }
        }
        args::Command::Follow { path, non_terminal } => {
            let source = std::fs::read_to_string(&path).unwrap();
            let mut parser = parser::Parser::new(&source);
            parser.parse();
            let tree = parser.tree();

            let grammar = grammar::GrammarBuilder::new(&source, tree).build();

            for nt in grammar.non_terminals() {
                let mut follow = HashSet::new();
                for (name, rule) in grammar.rules.iter() {
                    let f = grammar.follow_set_impl(&nt, name, rule, &mut HashSet::from([*name]));
                    follow.extend(f);
                }
                println!("{nt}: {follow:?}");
            }
        }
    }
}
