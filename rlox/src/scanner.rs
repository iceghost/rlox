use crate::{literal::Literal, token::Token, token_type::TokenTy};

static KEYWORDS: phf::Map<&'static str, TokenTy> = phf::phf_map! {
    "and" =>    TokenTy::And,
    "class" =>  TokenTy::Class,
    "else" =>   TokenTy::Else,
    "false" =>  TokenTy::False,
    "for" =>    TokenTy::For,
    "fun" =>    TokenTy::Fun,
    "if" =>     TokenTy::If,
    "nil" =>    TokenTy::Nil,
    "or" =>     TokenTy::Or,
    "print" =>  TokenTy::Print,
    "return" => TokenTy::Return,
    "super" =>  TokenTy::Super,
    "this" =>   TokenTy::This,
    "true" =>   TokenTy::True,
    "var" =>    TokenTy::Var,
    "while" =>  TokenTy::While,
};

#[derive(Default)]
pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    pub errors: Vec<ScanError>,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            ..Default::default()
        }
    }

    pub fn scan_tokens(mut self) -> Result<Vec<Token>> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.tokens
            .push(Token::new(TokenTy::Eof, String::new(), None, self.line));
        if self.errors.is_empty() {
            Ok(self.tokens)
        } else {
            Err(ScanError::Multiple(self.errors))
        }
    }

    fn scan_token(&mut self) {
        let ch = self.advance();
        match ch {
            '(' => self.add_token(TokenTy::LeftParen),
            ')' => self.add_token(TokenTy::RightParen),
            '{' => self.add_token(TokenTy::LeftBrace),
            '}' => self.add_token(TokenTy::RightBrace),
            ',' => self.add_token(TokenTy::Comma),
            '.' => self.add_token(TokenTy::Dot),
            '-' => self.add_token(TokenTy::Minus),
            '+' => self.add_token(TokenTy::Plus),
            ';' => self.add_token(TokenTy::Semicolon),
            '*' => self.add_token(TokenTy::Star),
            '!' => {
                let ty = if self.matches('=') {
                    TokenTy::BangEqual
                } else {
                    TokenTy::Bang
                };
                self.add_token(ty);
            }
            '=' => {
                let ty = if self.matches('=') {
                    TokenTy::EqualEqual
                } else {
                    TokenTy::Equal
                };
                self.add_token(ty);
            }
            '<' => {
                let ty = if self.matches('=') {
                    TokenTy::LessEqual
                } else {
                    TokenTy::Less
                };
                self.add_token(ty);
            }
            '>' => {
                let ty = if self.matches('=') {
                    TokenTy::GreaterEqual
                } else {
                    TokenTy::Greater
                };
                self.add_token(ty);
            }
            '/' => {
                if self.matches('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenTy::Slash);
                }
            }
            // skip
            ' ' | '\r' | '\t' => {}
            '\n' => {
                self.line += 1;
            }
            '"' => {
                self.string();
            }
            ch if ch.is_ascii_digit() => {
                self.number();
            }
            ch if ch.is_ascii_alphabetic() => {
                while self.peek().is_ascii_alphanumeric() {
                    self.advance();
                }

                let text = &self.source.as_bytes()[self.start..self.current];
                let text = String::from_utf8_lossy(text);
                if let Some(&ty) = KEYWORDS.get(&text) {
                    self.add_token(ty);
                } else {
                    self.add_token(TokenTy::Identifier);
                }
            }
            _ => self
                .errors
                .push(ScanError::Custom(self.line, "Unexpected character.".into())),
        }
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let value = &self.source.as_bytes()[self.start..self.current];
        let value = String::from_utf8_lossy(value);
        let value: f64 = value.parse().unwrap();
        self.add_literal(TokenTy::Number, Literal::Number(value));
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source.as_bytes()[self.current + 1].into()
        }
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            self.errors
                .push(ScanError::Custom(self.line, "Unterminated string.".into()));
        }

        // closing "
        self.advance();

        // trim
        let value = &self.source.as_bytes()[self.start + 1..self.current - 1];
        let value = String::from_utf8_lossy(value).into_owned();
        self.add_literal(TokenTy::String, Literal::String(value));
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source.as_bytes()[self.current].into()
        }
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source.as_bytes()[self.current] != expected as u8 {
            false
        } else {
            self.current += 1;
            true
        }
    }

    fn advance(&mut self) -> char {
        let ch = self.source.as_bytes()[self.current].into();
        self.current += 1;
        ch
    }

    #[inline]
    fn add_token(&mut self, ty: TokenTy) {
        self.add_token_or_literal(ty, None)
    }

    #[inline]
    fn add_literal(&mut self, ty: TokenTy, literal: Literal) {
        self.add_token_or_literal(ty, Some(literal))
    }

    fn add_token_or_literal(&mut self, ty: TokenTy, literal: Option<Literal>) {
        let text = &self.source.as_bytes()[self.start..self.current];
        let text = String::from_utf8_lossy(text).into_owned();
        self.tokens.push(Token::new(ty, text, literal, self.line))
    }

    #[inline]
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}

type Result<T> = std::result::Result<T, ScanError>;

pub enum ScanError {
    Custom(usize, std::borrow::Cow<'static, str>),
    Multiple(Vec<ScanError>),
}
