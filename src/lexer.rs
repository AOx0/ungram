use logos::{Logos, SpannedIter};

use crate::{ring::Ring, token};

pub struct Lexer<'src, const LOOKUP: usize> {
    inner: SpannedIter<'src, token::Kind>,
    buffer_span: Ring<crate::span::Span, LOOKUP>,
    buffer_kind: Ring<token::Kind, LOOKUP>,
    last_span: crate::span::Span,
}

impl<'src, const LOOKUP: usize> Lexer<'src, LOOKUP> {
    pub fn new(source: &'src str) -> Self {
        let mut s = Self {
            inner: token::Kind::lexer(source).spanned(),
            buffer_span: Ring::new(),
            buffer_kind: Ring::new(),
            last_span: crate::span::Span::from(0..0),
        };

        for _ in 0..LOOKUP {
            s.advance();
        }

        s
    }

    pub fn source(&self) -> &'src str {
        self.inner.source()
    }

    pub fn peek_array(&self) -> &[token::Kind; LOOKUP] {
        &self.buffer_kind.data
    }

    pub fn peek_token(&self) -> token::Token {
        token::Token::new(self.buffer_span[0], self.buffer_kind[0])
    }

    pub fn peek_kind(&self) -> token::Kind {
        self.buffer_kind[0]
    }

    pub fn advance(&mut self) {
        let (token, span) = self.next_token_impl();
        self.buffer_span.push(span);
        self.buffer_kind.push(token);
    }

    pub fn next_token(&mut self) -> token::Token {
        let curr = token::Token::new(self.buffer_span[0], self.buffer_kind[0]);
        self.advance();
        curr
    }

    fn next_token_impl(&mut self) -> (token::Kind, crate::span::Span) {
        self.inner
            .next()
            .map(|(token, span)| {
                (token.unwrap_or(token::Kind::Error), {
                    let span = crate::span::Span::from(span);
                    self.last_span = span;
                    span
                })
            })
            .unwrap_or((token::Kind::Eof, self.last_span))
    }
}

impl Iterator for Lexer<'_, 1> {
    type Item = token::Token;

    fn next(&mut self) -> Option<Self::Item> {
        (self.peek_kind() != token::Kind::Eof).then(|| self.next_token())
    }
}
