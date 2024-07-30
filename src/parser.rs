//! Recursive descent parser

use crate::{lexer::Lexer, token};

#[derive(Debug)]
pub struct Tree {
    pub kind: Kind,
    pub children: Vec<Child>,
}

#[derive(Debug, PartialEq, Eq)]
enum Event {
    Open { kind: Kind },
    Close,
    Skip,
    Advance { token: token::Token },
}

#[derive(Debug)]
pub enum Child {
    Tree(Tree),
    Token(token::Token),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Kind {
    Grammar,
    Rule,
    Sequence,
    ZeroOrMore,
    Optional,
    Branch,
    Error,
}

struct MarkOpen {
    index: usize,
}

struct MarkClose {
    index: usize,
}

pub struct Parser<'src> {
    lexer: Lexer<'src, 2>,
    events: Vec<Event>,
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            lexer: Lexer::new(source),
            events: Vec::new(),
        }
    }

    fn eof(&mut self) -> bool {
        self.lexer.peek_kind() == token::Kind::Eof
    }

    fn advance(&mut self) {
        let token = self.lexer.next_token();
        self.events.push(Event::Advance { token });
    }

    fn skip(&mut self) {
        self.lexer.advance();
        self.events.push(Event::Skip);
    }

    fn skip_if(&mut self, kind: token::Kind) -> bool {
        if self.lexer.peek_kind() == kind {
            self.skip();
            true
        } else {
            false
        }
    }

    fn skip_expect(&mut self, kind: token::Kind) {
        if !self.skip_if(kind) {
            let token = self.lexer.peek_token();
            panic!(
                "Skip expected {:?}, got {:?} at {:?}",
                kind,
                token,
                token.span.location(self.lexer.source())
            );
        }
    }

    fn open(&mut self) -> MarkOpen {
        self.events.push(Event::Open { kind: Kind::Error });
        MarkOpen {
            index: self.events.len() - 1,
        }
    }

    fn close(&mut self, opened: MarkOpen, kind: Kind) -> MarkClose {
        self.events[opened.index] = Event::Open { kind };
        self.events.push(Event::Close);
        MarkClose {
            index: opened.index,
        }
    }

    fn open_before(&mut self, opened: MarkClose) -> MarkOpen {
        self.events
            .insert(opened.index, Event::Open { kind: Kind::Error });
        MarkOpen {
            index: opened.index,
        }
    }

    fn expect(&mut self, kind: token::Kind) {
        if self.advance_if(kind) {
            return;
        } else {
            let token = self.lexer.peek_token();
            panic!(
                "Expected {:?}, got {:?} at {:?}",
                kind,
                token,
                token.span.location(self.lexer.source())
            );
        }
    }

    fn advance_if(&mut self, kind: crate::token::Kind) -> bool {
        if self.lexer.peek_kind() == kind {
            self.advance();
            true
        } else {
            false
        }
    }

    pub fn peek_array(&mut self) -> [token::Kind; 2] {
        self.lexer.peek_array()
    }

    pub fn peek(&mut self) -> token::Kind {
        self.lexer.peek_kind()
    }

    pub fn parse(&mut self) {
        grammar::file(self);
    }

    pub fn tree(mut self) -> Tree {
        let mut stack = Vec::new();

        assert_eq!(self.events.pop(), Some(Event::Close));

        for event in self.events {
            match event {
                Event::Open { kind } => {
                    stack.push(Tree {
                        kind,
                        children: Vec::new(),
                    });
                }
                Event::Close => {
                    let tree = stack.pop().unwrap();
                    stack.last_mut().unwrap().children.push(Child::Tree(tree));
                }
                Event::Skip => {}
                Event::Advance { token } => {
                    stack.last_mut().unwrap().children.push(Child::Token(token));
                }
            }
        }

        stack.pop().unwrap()
    }
}

mod grammar {
    use super::MarkClose;
    use super::Parser;
    use crate::token::Kind::*;
    use crate::token::Paren::*;

    pub fn file(p: &mut Parser) {
        let opened = p.open();
        while !p.eof() {
            rule(p);
        }

        p.close(opened, super::Kind::Grammar);
    }

    fn term(p: &mut Parser) {
        match p.peek() {
            Ident | Literal => {
                let star_or_question = if matches!(p.peek_array(), [_, Star]) {
                    Some(Star)
                } else if matches!(p.peek_array(), [_, Question]) {
                    Some(Question)
                } else {
                    None
                };

                if let Some(star_or_question) = star_or_question {
                    let mark = p.open();
                    p.advance();
                    p.skip();
                    p.close(
                        mark,
                        match star_or_question {
                            Star => super::Kind::ZeroOrMore,
                            Question => super::Kind::Optional,
                            _ => unreachable!(),
                        },
                    );
                } else {
                    p.advance();
                }
            }
            Paren(Open) => {
                p.skip();
                let close = expr(p);
                p.skip_expect(Paren(Close));

                if p.peek() == Star {
                    let mark = p.open_before(close);
                    p.skip();
                    p.close(mark, super::Kind::ZeroOrMore);
                } else if p.peek() == Question {
                    let mark = p.open_before(close);
                    p.skip();
                    p.close(mark, super::Kind::Optional);
                }
            }
            _ => panic!("Unexpected token"),
        }
    }

    fn expr(p: &mut Parser) -> MarkClose {
        if p.peek_array() == [Ident, Equal] {
            panic!("Unexpected rule definition");
        }

        let mut opened = p.open();
        let mut is_branch = false;
        let mut last_was_pipe = false;
        let mut parsed_in_sequence = 1;

        term(p);
        loop {
            match p.peek() {
                Pipe => {
                    // Allows `Rule = A B C | d` as `Rule = (A B C) | d `
                    if parsed_in_sequence > 1 {
                        let closed = p.close(opened, super::Kind::Sequence);
                        opened = p.open_before(closed);
                    }

                    p.skip(); // Skip the pipe
                    is_branch = true;
                    last_was_pipe = true;
                }
                Ident | Literal | Paren(Open) => {
                    if !is_branch {
                        parsed_in_sequence += 1;
                    }

                    if p.peek_array() == [Ident, Equal] {
                        return if is_branch {
                            p.close(opened, super::Kind::Branch)
                        } else {
                            p.close(opened, super::Kind::Sequence)
                        };
                    }

                    if is_branch && !last_was_pipe {
                        panic!(
                            "Cant use sequence in branch at {:?}",
                            p.lexer.peek_token().span.location(p.lexer.source())
                        );
                    }

                    last_was_pipe = false;
                    term(p)
                }
                _ => break,
            }
        }

        if is_branch {
            p.close(opened, super::Kind::Branch)
        } else {
            p.close(opened, super::Kind::Sequence)
        }
    }

    fn rule(p: &mut Parser) {
        let opened = p.open();
        p.expect(Ident);
        p.skip_expect(Equal);

        expr(p);

        p.close(opened, super::Kind::Rule);
    }
}
