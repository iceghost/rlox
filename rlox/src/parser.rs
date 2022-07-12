use crate::{expr::Expr, literal::Literal, stmt::Stmt, token::Token, token_type::TokenTy};

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
        } else {
            self.statement()
        }
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
        if self.matches([TokenTy::Print]) {
            self.print_statement()
        } else if self.matches([TokenTy::LeftBrace]) {
            Ok(Stmt::Block(self.block()?))
        } else {
            self.expression_statement()
        }
    }

    fn block(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = Vec::new();

        while !self.check(TokenTy::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenTy::RightBrace, "Expect '}' after block.".into())?;

        Ok(statements)
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
        let expr = self.equality()?;

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
