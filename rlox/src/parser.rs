use thiserror::Error;

use crate::{expr::Expr, literal::Literal, token::Token, token_type::TokenTy, Lox};

#[derive(Default)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            ..Default::default()
        }
    }

    pub fn parse(&mut self) -> Result<Expr> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut expr = self.comparison()?;

        while self.matches([TokenTy::BangEqual, TokenTy::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr> {
        let mut expr = self.term()?;

        while self.matches([
            TokenTy::Greater,
            TokenTy::GreaterEqual,
            TokenTy::Less,
            TokenTy::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr> {
        let mut expr = self.factor()?;

        while self.matches([TokenTy::Minus, TokenTy::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr> {
        let mut expr = self.unary()?;

        while self.matches([TokenTy::Slash, TokenTy::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr> {
        if self.matches([TokenTy::Bang, TokenTy::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr> {
        if self.matches([TokenTy::False]) {
            Ok(Expr::Literal(Literal::Boolean(false)))
        } else if self.matches([TokenTy::True]) {
            Ok(Expr::Literal(Literal::Boolean(true)))
        } else if self.matches([TokenTy::Nil]) {
            Ok(Expr::Literal(Literal::Nil))
        } else if self.matches([TokenTy::Number, TokenTy::String]) {
            Ok(Expr::Literal(self.previous().clone().literal.unwrap()))
        } else if self.matches([TokenTy::LeftParen]) {
            let expr = self.expression()?;
            self.comsume(TokenTy::RightParen, "Expect ')' after expression.")?;
            Ok(Expr::Grouping(Box::new(expr)))
        } else {
            Err(ParseError::Custom(
                self.peek().clone(),
                "Expect expression.".to_owned(),
            ))
        }
    }

    fn synchonize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().ty == TokenTy::Semicolon {
                return;
            }

            match self.peek().ty {
                TokenTy::Class
                | TokenTy::Fun
                | TokenTy::Var
                | TokenTy::For
                | TokenTy::If
                | TokenTy::While
                | TokenTy::Print
                | TokenTy::Return => {
                    return;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn comsume(&mut self, ty: TokenTy, message: &str) -> Result<&Token> {
        if self.check(ty) {
            Ok(self.advance())
        } else {
            Err(ParseError::Custom(self.peek().clone(), message.to_owned()))
        }
    }

    fn matches<const N: usize>(&mut self, tys: [TokenTy; N]) -> bool {
        if tys.iter().any(|&ty| self.check(ty)) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, ty: TokenTy) -> bool {
        !self.is_at_end() && self.peek().ty == ty
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().ty == TokenTy::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("{1}")]
    Custom(Token, String),
}

type Result<T> = std::result::Result<T, ParseError>;
