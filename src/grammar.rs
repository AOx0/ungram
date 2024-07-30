use std::collections::HashMap;

use crate::{
    parser::{Child, Kind, Tree},
    token,
};

#[derive(Debug)]
pub struct Grammar<'src> {
    rules: HashMap<&'src str, Expr<'src>>,
}

pub struct GrammarBuilder<'src> {
    source: &'src str,
    tree: Tree,
}

impl<'src> GrammarBuilder<'src> {
    pub fn new(source: &'src str, tree: Tree) -> Self {
        Self { source, tree }
    }

    pub fn build(mut self) -> Grammar<'src> {
        let mut rules = HashMap::new();
        for child in &self.tree.children {
            let Child::Tree(Tree {
                kind: Kind::Rule,
                children,
            }) = child
            else {
                panic!("expected rule");
            };

            let name = match &children[0] {
                Child::Token(token) => match token.kind {
                    token::Kind::Ident => &self.source[token.span.range()],
                    _ => panic!("expected ident"),
                },
                _ => panic!("expected token"),
            };
            let expr = self.parse_expr(&children[1]);
            rules.insert(name, expr);
        }
        Grammar { rules }
    }

    fn parse_expr(&self, child: &Child) -> Expr<'src> {
        match child {
            Child::Token(token) => match token.kind {
                token::Kind::Literal => {
                    Expr::Literal(&self.source[token.span.start + 1..token.span.end - 1])
                }
                token::Kind::Ident => Expr::Rule(&self.source[token.span.range()]),
                _ => panic!("unexpected token kind"),
            },
            Child::Tree(tree) => match tree.kind {
                Kind::Sequence => {
                    let mut exprs = Vec::new();
                    for child in &tree.children {
                        exprs.push(self.parse_expr(child));
                    }
                    Expr::Sequence(exprs)
                }
                Kind::Branch => {
                    let mut exprs = Vec::new();
                    for child in &tree.children {
                        exprs.push(self.parse_expr(child));
                    }
                    Expr::Choice(exprs)
                }
                Kind::Optional => {
                    let child = &tree.children[0];
                    Expr::Optional(Box::new(self.parse_expr(child)))
                }
                Kind::ZeroOrMore => {
                    let child = &tree.children[0];
                    Expr::Repeat(Box::new(self.parse_expr(child)))
                }
                _ => panic!("unexpected tree kind"),
            },
        }
    }
}

#[derive(Debug)]
pub enum Expr<'src> {
    Literal(&'src str),
    Sequence(Vec<Expr<'src>>),
    Choice(Vec<Expr<'src>>),
    Optional(Box<Expr<'src>>),
    Repeat(Box<Expr<'src>>),
    Rule(&'src str),
}
