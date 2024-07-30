#[derive(Debug, PartialEq, Eq, Clone, Copy, logos::Logos, Default)]
#[repr(u8)]
pub enum Kind {
    #[regex("[ \t\r\n]+", logos::skip)]
    Ignored,

    #[regex(r"[a-zA-Z0-9_]+")]
    Ident,

    #[token("=")]
    Equal,

    #[token(":")]
    Colon,

    #[token("*")]
    Star,

    #[token("?")]
    Question,

    #[regex(r"'[^']*'")]
    Literal,

    #[regex("//.*", logos::skip)]
    Comment,

    #[token("(", |_| Paren::Open)]
    #[token(")", |_| Paren::Close)]
    Paren(Paren),

    #[token("|")]
    Pipe,

    Error,

    #[default]
    Eof,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Paren {
    Open,
    Close,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Token {
    pub span: crate::span::Span,
    pub kind: Kind,
}

impl Token {
    pub fn new(span: crate::span::Span, kind: Kind) -> Self {
        Self { span, kind }
    }
}
