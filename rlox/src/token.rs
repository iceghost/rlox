use crate::{literal::Literal, token_type::TokenTy};

#[derive(Debug, Clone)]
pub struct Token {
    pub ty: TokenTy,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub line: usize,
}

impl Token {
    pub fn new(ty: TokenTy, lexeme: String, literal: Option<Literal>, line: usize) -> Self {
        Token {
            ty,
            lexeme,
            literal,
            line,
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.literal {
            Some(literal) => f.write_fmt(format_args!("{:?} {} {}", self.ty, self.lexeme, literal)),
            None => f.write_fmt(format_args!("{:?} {}", self.ty, self.lexeme)),
        }
    }
}
