use std::{borrow::Borrow, str::CharIndices};

use itertools::{Itertools, MultiPeek};

use self::token::{Token, Ty};

pub mod token;

pub struct Scanner<'a> {
    source: &'a str,
    start: usize,
    current: MultiPeek<CharIndices<'a>>,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        let start = 0;
        let current = source.char_indices().multipeek();
        let line = 1;
        Self {
            source,
            start,
            current,
            line,
        }
    }

    #[inline]
    fn offset(&mut self) -> usize {
        let offset = self
            .current
            .peek()
            .map(|&(offset, _)| offset)
            .unwrap_or(self.source.len());
        self.current.reset_peek();
        offset
    }

    fn make_token(&mut self, ty: Ty) -> Token<'a> {
        let offset = self.offset();
        let lexeme = &self.source[self.start..offset];
        self.start = offset;
        Token::new(ty, lexeme, self.line)
    }

    fn error_token(&self, message: &'static str) -> Token<'static> {
        let ty = Ty::Error;
        let lexeme = message;
        Token::new(ty, lexeme, self.line)
    }

    #[inline]
    fn peek(&mut self) -> Option<char> {
        self.current.peek().map(|&(_, c)| c)
    }

    fn reset_peek(&mut self) {
        self.current.reset_peek();
    }

    #[inline]
    fn advance(&mut self) -> Option<char> {
        self.current.next().map(|(_, c)| c)
    }

    fn matches(&mut self, expected: char) -> bool {
        if let Some(current) = self.peek() {
            if current == expected {
                self.advance();
                return true;
            }
        }
        self.reset_peek();
        false
    }

    fn skip_whitespace(&mut self) {
        'outer: loop {
            match self.peek() {
                Some(' ') | Some('\r') | Some('\t') => {
                    self.advance();
                }
                Some('\n') => {
                    self.line += 1;
                    self.advance();
                }
                Some('/') => {
                    if let Some('/') = self.peek() {
                        loop {
                            self.advance();
                            if matches!(self.peek(), Some('\n') | None) {
                                self.current.reset_peek();
                                continue 'outer;
                            }
                        }
                    }
                    break 'outer;
                }
                _ => break 'outer,
            }
        }
        self.current.reset_peek();
    }

    fn string(&mut self) -> Token<'a> {
        while !matches!(self.peek(), Some('"') | None) {
            if let Some('\n') = self.advance() {
                self.line += 1;
            }
        }

        match self.advance() {
            // the closing quote
            Some(_) => self.make_token(Ty::String),
            None => self.error_token("Unterminated string."),
        }
    }

    fn peek_is_digit(&mut self) -> bool {
        matches!(self.peek(), Some(c) if c.is_ascii_digit())
    }

    fn number(&mut self) -> Token<'a> {
        while self.peek_is_digit() {
            self.advance();
        }

        if matches!(self.peek(), Some('.')) && self.peek_is_digit() {
            self.advance();
            while self.peek_is_digit() {
                self.advance();
            }
        }
        self.reset_peek();
        self.make_token(Ty::Number)
    }

    fn identifier_type(&mut self) -> Ty {
        match self.source.as_bytes()[self.start] {
            b'a' => return self.check_keyword(1, b"nd", Ty::And),
            b'c' => return self.check_keyword(1, b"lass", Ty::Class),
            b'e' => return self.check_keyword(1, b"lse", Ty::Else),
            b'f' => {
                if self.offset() - self.start > 1 {
                    match self.source.as_bytes()[self.start + 1] {
                        b'a' => return self.check_keyword(2, b"lse", Ty::False),
                        b'o' => return self.check_keyword(2, b"r", Ty::For),
                        b'u' => return self.check_keyword(2, b"n", Ty::Fun),
                        _ => {}
                    }
                }
            }
            b'i' => return self.check_keyword(1, b"f", Ty::If),
            b'n' => return self.check_keyword(1, b"il", Ty::Nil),
            b'o' => return self.check_keyword(1, b"r", Ty::Or),
            b'p' => return self.check_keyword(1, b"rint", Ty::Print),
            b'r' => return self.check_keyword(1, b"eturn", Ty::Return),
            b's' => return self.check_keyword(1, b"uper", Ty::Super),
            b't' => {
                if self.offset() - self.start > 1 {
                    match self.source.as_bytes()[self.start + 1] {
                        b'h' => return self.check_keyword(2, b"is", Ty::This),
                        b'r' => return self.check_keyword(2, b"ue", Ty::True),
                        _ => {}
                    }
                }
            }
            b'v' => return self.check_keyword(1, b"ar", Ty::Var),
            b'w' => return self.check_keyword(1, b"hile", Ty::While),
            _ => {}
        }
        Ty::Identifier
    }

    fn check_keyword(&mut self, start: usize, rest: &[u8], ty: Ty) -> Ty {
        if &self.source.as_bytes()[self.start + start..self.offset()] == rest {
            ty
        } else {
            Ty::Identifier
        }
    }

    fn identifier(&mut self) -> Token<'a> {
        while matches!(self.peek(), Some(c) if c.is_ascii_alphanumeric()) {
            self.advance();
        }
        self.reset_peek();
        let ty = self.identifier_type();
        self.make_token(ty)
    }

    pub fn scan_token(&mut self) -> Token<'a> {
        self.skip_whitespace();
        self.start = self.offset();
        match self.advance() {
            None => self.make_token(Ty::Eof),
            Some(c) if c.is_ascii_alphabetic() => self.identifier(),
            Some(c) if c.is_ascii_digit() => self.number(),
            Some('(') => self.make_token(Ty::LeftParen),
            Some(')') => self.make_token(Ty::RightParen),
            Some('{') => self.make_token(Ty::LeftBrace),
            Some('}') => self.make_token(Ty::RightBrace),
            Some(';') => self.make_token(Ty::Semicolon),
            Some(',') => self.make_token(Ty::Comma),
            Some('.') => self.make_token(Ty::Dot),
            Some('-') => self.make_token(Ty::Minus),
            Some('+') => self.make_token(Ty::Plus),
            Some('/') => self.make_token(Ty::Slash),
            Some('*') => self.make_token(Ty::Star),
            Some('!') => {
                let token = if self.matches('=') {
                    Ty::BangEqual
                } else {
                    Ty::Bang
                };
                self.make_token(token)
            }
            Some('=') => {
                let token = if self.matches('=') {
                    Ty::EqualEqual
                } else {
                    Ty::Equal
                };
                self.make_token(token)
            }
            Some('<') => {
                let token = if self.matches('=') {
                    Ty::LessEqual
                } else {
                    Ty::Less
                };
                self.make_token(token)
            }
            Some('>') => {
                let token = if self.matches('=') {
                    Ty::GreaterEqual
                } else {
                    Ty::Greater
                };
                self.make_token(token)
            }
            Some('"') => self.string(),
            _ => self.error_token("Unexpected character."),
        }
    }
}
