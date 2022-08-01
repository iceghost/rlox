#[derive(Clone, Copy)]
pub struct Token<'a> {
    ty: Ty,
    lexeme: &'a str,
    line: usize,
}

impl<'a> Token<'a> {
    pub fn new(ty: Ty, lexeme: &'a str, line: usize) -> Self {
        Self { ty, lexeme, line }
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn lexeme(&self) -> &str {
        self.lexeme
    }

    pub fn ty(&self) -> Ty {
        self.ty
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ty {
    // single character
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // one or two character
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // literals
    Identifier,
    String,
    Number,

    // keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Error,
    Eof,
}
