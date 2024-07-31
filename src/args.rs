use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Lex {
        path: PathBuf,
    },
    Tree {
        path: PathBuf,
    },
    Parse {
        path: PathBuf,
    },
    First {
        path: PathBuf,
        non_terminal: Option<String>,
    },
    Follow {
        path: PathBuf,
        non_terminal: Option<String>,
    },
}
