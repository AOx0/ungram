//! Recursive descent parser with an embedded pratt parser to
//! parse grammar rules.

use crate::{
    lexer::Lexer,
    token::{Kind as TKind, Token},
};

#[derive(Debug)]
pub struct Tree {
    pub kind: Kind,
    pub children: Vec<Child>,
}

impl Tree {
    pub fn rule(token: Token) -> Self {
        Self {
            kind: Kind::Rule,
            children: vec![Child::Token(token)],
        }
    }
}

#[derive(Debug)]
pub enum Child {
    Tree(Tree),
    Token(Token),
}

#[derive(Debug)]
enum Kind {
    Grammar,
    Rule,
    ZeroOrMore,
    Optional,
    Sequence,
    Branch,
}

pub struct Parser<'src> {
    lexer: Lexer<'src, 2>,
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            lexer: Lexer::new(source),
        }
    }

    fn eof(&mut self) -> bool {
        self.lexer.peek_kind() == TKind::Eof
    }

    fn advance(&mut self) -> Token {
        self.lexer.next_token()
    }

    fn advance_if(&mut self, kind: crate::token::Kind) -> Option<Token> {
        (self.lexer.peek_kind() == kind).then(|| self.lexer.next_token())
    }

    pub fn parse(mut self) -> Tree {
        let mut tree = Tree {
            kind: Kind::Grammar,
            children: Vec::new(),
        };

        while !self.eof() {
            let rule = self.parse_rule();
            tree.children.push(Child::Tree(rule));
        }

        tree
    }

    pub fn parse_rule(&mut self) -> Tree {
        let mut tree = Tree {
            kind: Kind::Rule,
            children: Vec::new(),
        };

        let Some(name) = self.advance_if(TKind::Ident) else {
            panic!(
                "Expected identifier found {:?} at {:?}",
                self.lexer.peek_token(),
                self.lexer.peek_token().span.location(self.lexer.source())
            );
        };

        let _ = self.advance_if(TKind::Equal).unwrap();

        let expr = self.parse_expr();
        tree.children.push(Child::Tree(expr));

        tree
    }

    pub fn peek(&mut self) -> TKind {
        self.lexer.peek_kind()
    }

    // Uses pratt parsing technique with absolute order of precedence
    // based on
    pub fn parse_expr(&mut self) -> Tree {
        use crate::token::Paren;
        use TKind::*;

        let mut seq = Tree {
            kind: Kind::Sequence,
            children: vec![match self.peek() {
                Ident => Child::Token(self.advance()),
                Literal => Child::Token(self.advance()),
                Paren(Paren::Open) => {
                    let _ = self.advance_if(Paren(Paren::Open)).unwrap();
                    let expr = self.parse_expr();
                    let _ = self.advance_if(Paren(Paren::Close)).unwrap();
                    Child::Tree(expr)
                }
                _ => panic!("Unexpected token"),
            }],
        };

        while !matches!(self.lexer.peek_array(), [Ident, Equal]) {
            match self.peek() {
                Ident if matches!(self.lexer.peek_array(), [Ident, Equal]) => break,
                Paren(Paren::Close) => break,
                Eof => break,
                Pipe => {
                    let _ = self.advance_if(Pipe).unwrap();
                    let rhs = self.parse_expr();
                    seq.kind = Kind::Branch;
                    seq.children.push(Child::Tree(rhs));
                }
                Star => {
                    let _ = self.advance_if(Star).unwrap();
                    seq = Tree {
                        kind: Kind::ZeroOrMore,
                        children: vec![Child::Tree(seq)],
                    };
                }
                Question => {
                    let _ = self.advance_if(Question).unwrap();
                    seq = Tree {
                        kind: Kind::Optional,
                        children: vec![Child::Tree(seq)],
                    };
                }
                Ident | Literal => {
                    seq.children.push(Child::Token(self.advance()));
                }
                Paren(Paren::Open) => {
                    let _ = self.advance_if(Paren(Paren::Open)).unwrap();
                    let expr = self.parse_expr();
                    let _ = self.advance_if(Paren(Paren::Close)).unwrap();
                    seq.children.push(Child::Tree(expr));
                }
                _ => panic!(
                    "Unexpected token {:?} at {:?} status:\n{:#?}",
                    self.lexer.peek_token(),
                    self.lexer.peek_token().span.location(self.lexer.source()),
                    seq
                ),
            }
        }

        seq
    }
}
