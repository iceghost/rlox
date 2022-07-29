use std::rc::Rc;

use crate::{
    expr::Expr,
    literal::Literal,
    stmt::{Stmt, StmtFunction},
    token::Token,
    token_type::TokenTy,
};

#[derive(Default)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    errors: Vec<ParseError>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            ..Default::default()
        }
    }

    pub fn parse(mut self) -> Result<Vec<Stmt>> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => {
                    statements.push(stmt);
                }
                Err(err) => {
                    self.errors.push(err);
                    self.synchonize();
                }
            }
        }
        if self.errors.is_empty() {
            Ok(statements)
        } else {
            Err(ParseError::Multiple(self.errors))
        }
    }

    fn declaration(&mut self) -> Result<Stmt> {
        if self.matches([TokenTy::Var]) {
            self.var_declaration()
        } else if self.matches([TokenTy::Fun]) {
            self.function("function")
        } else {
            self.statement()
        }
    }

    fn function(&mut self, kind: &'static str) -> Result<Stmt> {
        let name = self
            .consume(TokenTy::Identifier, format!("Expect {kind} name.").into())?
            .clone();
        self.consume(
            TokenTy::LeftParen,
            format!("Expect '(' after {kind} name.").into(),
        )?;
        let mut params = Vec::new();

        if !self.check(TokenTy::RightParen) {
            loop {
                if params.len() >= 255 {
                    self.errors.push(ParseError::Custom(
                        self.peek().clone(),
                        "Can't have more than 255 parameters.".into(),
                    ));
                }

                params.push(
                    self.consume(TokenTy::Identifier, "Expect parameter name.".into())?
                        .clone(),
                );

                if !self.matches([TokenTy::Comma]) {
                    break;
                }
            }
        }
        self.consume(TokenTy::RightParen, "Expect ')' after parameters.".into())?;
        self.consume(
            TokenTy::LeftBrace,
            format!("Expect '{{' before {kind} body.").into(),
        )?;
        let body = self.block()?;

        Ok(Stmt::Function(Rc::new(StmtFunction { name, params, body })))
    }

    fn var_declaration(&mut self) -> Result<Stmt> {
        let name = self
            .consume(TokenTy::Identifier, "Expect variable name.".into())?
            .clone();

        let initializer = self
            .matches([TokenTy::Equal])
            .then(|| self.expression())
            .transpose();
        let initializer = initializer?;

        self.consume(
            TokenTy::Semicolon,
            "Expect ';' after variable declaration.".into(),
        )?;

        Ok(Stmt::Var { name, initializer })
    }

    fn statement(&mut self) -> Result<Stmt> {
        if self.matches([TokenTy::If]) {
            self.if_statement()
        } else if self.matches([TokenTy::For]) {
            self.for_statement()
        } else if self.matches([TokenTy::While]) {
            self.while_statement()
        } else if self.matches([TokenTy::Print]) {
            self.print_statement()
        } else if self.matches([TokenTy::LeftBrace]) {
            Ok(Stmt::Block(self.block()?))
        } else if self.matches([TokenTy::Return]) {
            self.return_statement()
        } else {
            self.expression_statement()
        }
    }

    fn return_statement(&mut self) -> Result<Stmt> {
        let keyword = self.previous().clone();
        let value = if !self.check(TokenTy::Semicolon) {
            self.expression()?
        } else {
            Expr::Literal(().into())
        };
        self.consume(TokenTy::Semicolon, "Expect ';' after return value.".into())?;
        Ok(Stmt::Return { keyword, value })
    }

    fn block(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = Vec::new();

        while !self.check(TokenTy::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenTy::RightBrace, "Expect '}' after block.".into())?;

        Ok(statements)
    }

    fn if_statement(&mut self) -> Result<Stmt> {
        self.consume(TokenTy::LeftParen, "Expect '(' after 'if'.".into())?;
        let condition = self.expression()?;
        self.consume(TokenTy::RightParen, "Expect ')' after if condition.".into())?;

        let then_branch = self.statement()?;
        let else_branch = self
            .matches([TokenTy::Else])
            .then(|| self.statement())
            .transpose()?;

        Ok(Stmt::If {
            condition,
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        })
    }

    fn for_statement(&mut self) -> Result<Stmt> {
        self.consume(TokenTy::LeftParen, "Expect '(' after 'if'.".into())?;
        let initializer = if self.matches([TokenTy::Semicolon]) {
            None
        } else if self.matches([TokenTy::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if !self.check(TokenTy::Semicolon) {
            self.expression()?
        } else {
            Expr::Literal(true.into())
        };
        self.consume(
            TokenTy::Semicolon,
            "Expect ';' after loop condition.".into(),
        )?;

        let increment = (!self.check(TokenTy::RightParen))
            .then(|| self.expression())
            .transpose()?;
        self.consume(TokenTy::RightParen, "Expect ')' after for clauses.".into())?;

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(increment)]);
        }

        body = Stmt::While {
            condition,
            body: Box::new(body),
        };

        if let Some(initializer) = initializer {
            body = Stmt::Block(vec![initializer, body]);
        }

        Ok(body)
    }

    fn while_statement(&mut self) -> Result<Stmt> {
        self.consume(TokenTy::LeftParen, "Expect '(' after 'if'.".into())?;
        let condition = self.expression()?;
        self.consume(TokenTy::RightParen, "Expect ')' after if condition.".into())?;

        let body = self.statement()?;

        Ok(Stmt::While {
            condition,
            body: Box::new(body),
        })
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let value = self.expression()?;
        self.consume(TokenTy::Semicolon, "Expect ';' after value.".into())?;
        Ok(Stmt::Print(value))
    }

    fn expression_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenTy::Semicolon, "Expect ';' after expression".into())?;
        Ok(Stmt::Expression(expr))
    }

    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.or()?;

        if self.matches([TokenTy::Equal]) {
            let equals = self.previous().clone();
            let value = self.assignment()?;

            if let Expr::Variable(name) = expr {
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                });
            }

            self.errors.push(ParseError::Custom(
                equals,
                "Invalid assignment target.".into(),
            ));
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr> {
        let mut expr = self.and()?;

        while self.matches([TokenTy::Or]) {
            let operator = self.previous().clone();
            let right = self.and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr> {
        let mut expr = self.equality()?;

        while self.matches([TokenTy::And]) {
            let operator = self.previous().clone();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
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
            self.call()
        }
    }

    fn call(&mut self) -> Result<Expr> {
        let mut expr = self.primary()?;

        loop {
            if self.matches([TokenTy::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr> {
        let mut arguments = Vec::new();

        if !self.check(TokenTy::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    self.errors.push(ParseError::Custom(
                        self.peek().clone(),
                        "Can't have more than 255 arguments".into(),
                    ));
                }
                arguments.push(self.expression()?);
                if !self.matches([TokenTy::Comma]) {
                    break;
                }
            }
        }

        let paren = self
            .consume(TokenTy::RightParen, "Expect ')' after arguments.".into())?
            .clone();

        Ok(Expr::Call {
            callee: Box::new(callee),
            paren,
            arguments,
        })
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
        } else if self.matches([TokenTy::Identifier]) {
            Ok(Expr::Variable(self.previous().clone()))
        } else if self.matches([TokenTy::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenTy::RightParen, "Expect ')' after expression.".into())?;
            Ok(Expr::Grouping(Box::new(expr)))
        } else {
            Err(ParseError::Custom(
                self.peek().clone(),
                "Expect expression.".into(),
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

    fn consume(&mut self, ty: TokenTy, message: std::borrow::Cow<'static, str>) -> Result<&Token> {
        if self.check(ty) {
            Ok(self.advance())
        } else {
            Err(ParseError::Custom(self.peek().clone(), message))
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

#[derive(Debug)]
pub enum ParseError {
    Custom(Token, std::borrow::Cow<'static, str>),
    Multiple(Vec<ParseError>),
}

type Result<T> = std::result::Result<T, ParseError>;
