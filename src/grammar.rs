use std::{
    collections::{HashMap, HashSet},
    ops::Not,
};

use crate::{
    parser::{Child, Kind, Tree},
    token,
};

pub struct Grammar<'src> {
    pub rules: HashMap<&'src str, Expr<'src>>,
}

impl<'src> Grammar<'src> {
    pub fn follow_set_impl(
        &'src self,
        of: &str,
        parent: &'src str,
        expr: &'src Expr,
        productions: &mut HashSet<&'src str>,
    ) -> HashSet<&'src str> {
        let mut set = HashSet::new();

        match expr {
            Expr::Choice(branches) => {
                for branch in branches {
                    set.extend(self.follow_set_impl(of, parent, branch, productions))
                }
            }
            Expr::Sequence(exprs) => {
                let mut iter = exprs.iter();
                let mut last_may_be_empty = false;

                while let Some(expr) = iter.next() {
                    let is_match = last_may_be_empty || expr.produces_at_end(&Expr::Rule(of));
                    if is_match.not() {
                        continue;
                    };

                    // Up to this point we have a derivation Z -> ..Aβ, now:
                    let next = iter.clone().next();

                    // Computes the first set of self in cases where repetition of self is possible
                    // i.e `Fn*` may produce `Fn Fn`, hence FIRST(Fn) must be added to FOLLOW(Fn)
                    if let Expr::Repeat(rep) = expr
                        && rep.produces_at_end(&Expr::Rule(of))
                    {
                        set.extend(self.first_set_impl(rep, &mut HashSet::new()));
                    }

                    // - We compute the FIRST set of the following expression β
                    let perform_follow = if let Some(expr) = next {
                        let mut first = self.first_set_impl(expr, &mut HashSet::new());
                        let contains_empty = first.remove("ε");
                        set.extend(first);

                        // I believe this is redundant
                        last_may_be_empty = contains_empty || expr.may_miss();
                        contains_empty
                    } else {
                        true
                    };

                    // - If theres no following expr or FIRST(β) contains ε we compute
                    // FOLLOW(β) and add it to the FOLLOW(A)
                    if perform_follow {
                        for (sub_name, sub_rule) in self.rules.iter() {
                            let mut productions = productions.clone();
                            if !productions.insert(sub_name) {
                                continue;
                            };
                            set.extend(self.follow_set_impl(
                                parent,
                                sub_name,
                                sub_rule,
                                &mut productions,
                            ));
                        }
                    };
                }
            }
            _ => panic!("Non valid expr from {parent:?}: {expr:?}"),
        }

        set
    }

    pub fn first_set(&'src self, name: &'src str) -> HashSet<&'src str> {
        let expr = self
            .rules
            .get(name)
            .expect(&format!("rule not found {name:?}"));
        self.first_set_impl(expr, &mut HashSet::from([name]))
    }

    pub fn non_terminals(&self) -> HashSet<&str> {
        self.rules.keys().copied().collect()
    }

    pub fn first_set_impl(
        &'src self,
        expr: &'src Expr,
        productions: &mut HashSet<&'src str>,
    ) -> HashSet<&'src str> {
        let mut set: HashSet<&str> = HashSet::new();

        match expr {
            Expr::Literal(lit) => {
                set.insert(lit);
            }
            Expr::Rule(rule) => {
                if !productions.insert(rule) {
                    return set;
                }
                let expr = self
                    .rules
                    .get(rule)
                    .expect(&format!("rule not found {rule:?}"));
                set.extend(self.first_set_impl(expr, productions));
            }
            Expr::Sequence(exprs) => {
                let mut iter = exprs.iter();
                loop {
                    let Some(curr) = iter.next() else {
                        set.insert("ε"); // May have nothing as first
                        break;
                    };

                    match curr {
                        Expr::Optional(expr) | Expr::Repeat(expr) => {
                            set.extend(self.first_set_impl(expr, productions));
                        }
                        _ => {
                            set.extend(self.first_set_impl(curr, productions));
                            break;
                        }
                    }
                }
            }
            Expr::Choice(exprs) => exprs.into_iter().for_each(|expr| {
                set.extend(self.first_set_impl(expr, productions));
            }),
            Expr::Optional(expr) => return self.first_set_impl(expr, productions),
            Expr::Repeat(expr) => return self.first_set_impl(expr, productions),
        }

        set
    }
}

impl<'src> std::fmt::Debug for Grammar<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            writeln!(f, "Grammar {{")?;
            for (name, expr) in &self.rules {
                writeln!(f, "  {}: {:?}", name, expr)?;
            }
            writeln!(f, "}}")
        } else {
            write!(f, "Grammar")
        }
    }
}

pub struct GrammarBuilder<'src> {
    source: &'src str,
    tree: Tree,
}

impl<'src> GrammarBuilder<'src> {
    pub fn new(source: &'src str, tree: Tree) -> Self {
        Self { source, tree }
    }

    pub fn build(self) -> Grammar<'src> {
        let mut rules = HashMap::new();
        for child in &self.tree.children {
            let Child::Tree(Tree {
                kind: Kind::Rule,
                children,
            }) = child
            else {
                panic!("expected rule found {:?}", child);
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
                    if tree.children.len() == 1 {
                        return self.parse_expr(&tree.children[0]);
                    }

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

#[derive(Debug, PartialEq, Eq)]
pub enum Expr<'src> {
    Literal(&'src str),
    Rule(&'src str),
    Sequence(Vec<Self>),
    Choice(Vec<Self>),
    Optional(Box<Self>),
    Repeat(Box<Self>),
}

impl<'src> Expr<'src> {
    fn may_miss(&self) -> bool {
        match self {
            Expr::Optional(_) => true,
            Expr::Repeat(_) => true,
            Expr::Literal(_) => false,
            Expr::Rule(_) => false,
            Expr::Sequence(_) => false,
            Expr::Choice(_) => false,
        }
    }

    fn produces_at_end(&self, expr: &Expr) -> bool {
        match self {
            x @ Expr::Literal(_) => expr == x,
            x @ Expr::Rule(_) => expr == x,
            Expr::Sequence(exprs) => exprs.last().is_some_and(|x| expr == x),
            Expr::Choice(branches) => branches.iter().any(|x| x.produces_at_end(expr)),
            Expr::Optional(x) => x.produces_at_end(expr),
            Expr::Repeat(x) => x.produces_at_end(expr),
        }
    }
}
