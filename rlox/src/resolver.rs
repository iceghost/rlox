use std::collections::HashMap;

use crate::{
    expr::Expr,
    interpreter::Interpreter,
    stmt::{Stmt, StmtFunction},
    token::Token,
};

pub struct Resolver<'intpt> {
    interpreter: &'intpt mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    errors: Vec<ResolveError>,
    function_ty: FunctionType,
}

pub enum ResolveError {
    Custom(Token, std::borrow::Cow<'static, str>),
    Multiple(Vec<ResolveError>),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FunctionType {
    None,
    Function,
}

pub type Result<T> = std::result::Result<T, ResolveError>;

impl<'intpt> Resolver<'intpt> {
    pub fn new(interpreter: &'intpt mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: Default::default(),
            errors: Default::default(),
            function_ty: FunctionType::None,
        }
    }

    pub fn resolve(mut self, statements: &[Stmt]) -> Result<()> {
        self.resolve_block(statements);

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(ResolveError::Multiple(self.errors))
        }
    }

    fn resolve_block(&mut self, statements: &[Stmt]) {
        for statement in statements {
            self.resolve_statement(statement);
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(Default::default());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn resolve_statement(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Block(statements) => {
                self.begin_scope();
                self.resolve_block(statements);
                self.end_scope();
            }
            Stmt::Var { name, initializer } => {
                self.declare(name);
                if let Some(initializer) = initializer {
                    self.resolve_expression(initializer);
                }
                self.define(name);
            }
            Stmt::Function(statement) => {
                self.declare(&statement.name);
                self.define(&statement.name);
                self.resolve_function(statement, FunctionType::Function);
            }
            Stmt::Expression(expression) => {
                self.resolve_expression(expression);
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expression(condition);
                self.resolve_statement(then_branch);
                if let Some(else_branch) = else_branch {
                    self.resolve_statement(else_branch);
                }
            }
            Stmt::Print(expression) => {
                self.resolve_expression(expression);
            }
            Stmt::Return { value, keyword } => {
                if self.function_ty == FunctionType::None {
                    self.errors.push(ResolveError::Custom(
                        keyword.clone(),
                        "Can't return from top-level code.".into(),
                    ))
                }
                self.resolve_expression(value);
            }
            Stmt::While { condition, body } => {
                self.resolve_expression(condition);
                self.resolve_statement(body);
            }
        }
    }

    fn resolve_function(&mut self, function: &StmtFunction, function_ty: FunctionType) {
        let enclosing_function = self.function_ty;
        self.function_ty = function_ty;
        self.begin_scope();
        for param in &function.params {
            self.declare(param);
            self.define(param);
        }
        self.end_scope();
        self.function_ty = enclosing_function;
    }

    fn declare(&mut self, name: &Token) -> Option<()> {
        let scope = self.scopes.last_mut()?;
        if scope.insert(name.lexeme.to_owned(), false).is_some() {
            self.errors.push(ResolveError::Custom(
                name.clone(),
                "Already a variable with this name in this scope.".into(),
            ));
        }
        Some(())
    }

    fn define(&mut self, name: &Token) -> Option<()> {
        let scope = self.scopes.last_mut()?;
        let variable = scope.get_mut(&name.lexeme).expect("undeclared variable");
        *variable = true;
        Some(())
    }

    fn resolve_expression(&mut self, expression: &Expr) {
        match expression {
            Expr::Variable(name) => {
                let scope = self.scopes.last();
                if let Some(scope) = scope {
                    if let Some(false) = scope.get(&name.lexeme) {
                        self.errors.push(ResolveError::Custom(
                            name.clone(),
                            "Can't read local variable in its own initializer.".into(),
                        ));
                    }
                }
                self.resolve_local(expression, name);
            }
            Expr::Assign { name, value } => {
                self.resolve_expression(value);
                self.resolve_local(expression, name);
            }
            Expr::Binary { left, right, .. } => {
                self.resolve_expression(left);
                self.resolve_expression(right);
            }
            Expr::Call {
                callee, arguments, ..
            } => {
                self.resolve_expression(callee);
                for argument in arguments {
                    self.resolve_expression(argument);
                }
            }
            Expr::Grouping(expression) => self.resolve_expression(expression),
            Expr::Literal(_) => {}
            Expr::Logical { left, right, .. } => {
                self.resolve_expression(left);
                self.resolve_expression(right);
            }
            Expr::Unary { right, .. } => {
                self.resolve_expression(right);
            }
        }
    }

    fn resolve_local(&mut self, expression: &Expr, name: &Token) {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.resolve(expression, i);
                return;
            }
        }
    }
}
