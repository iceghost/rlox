use std::rc::Rc;

use crate::{
    environment::EnvironmentPointer, expr::Expr, literal::Literal, lox_function::LoxFunction,
    native_functions, object::Object, stmt::Stmt, token::Token, token_type::TokenTy,
};

pub struct Interpreter {
    #[allow(dead_code)]
    pub globals: EnvironmentPointer,
    pub environment: EnvironmentPointer,
}

impl Default for Interpreter {
    fn default() -> Self {
        let mut globals = EnvironmentPointer::default();
        globals.define(
            "clock".into(),
            Object::from_callable(native_functions::Clock),
        );
        let environment = globals.clone();
        Self {
            globals,
            environment,
        }
    }
}

impl Interpreter {
    pub fn interpret(&mut self, statements: &Vec<Stmt>) -> Result<()> {
        for statement in statements {
            self.execute(statement)?;
        }
        Ok(())
    }
    pub fn execute(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Expression(expr) => {
                self.evaluate(expr)?;
            }
            Stmt::Print(expr) => {
                let value = self.evaluate(expr)?;
                println!("{value}");
            }
            Stmt::Var { name, initializer } => {
                let value = initializer
                    .as_ref()
                    .map_or(Ok(().into()), |expr| self.evaluate(expr))?;
                self.environment.define(name.lexeme.to_owned(), value);
            }
            Stmt::Block(stmts) => {
                self.execute_block(stmts, EnvironmentPointer::new(self.environment.clone()))?;
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if Self::is_truthy(&self.evaluate(condition)?) {
                    self.execute(then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch)?;
                }
            }
            Stmt::While { condition, body } => {
                while Self::is_truthy(&self.evaluate(condition)?) {
                    self.execute(body)?;
                }
            }
            Stmt::Function(stmt) => {
                let function = LoxFunction::new(Rc::clone(stmt), self.environment.clone());
                self.environment
                    .define(stmt.name.lexeme.to_owned(), Object::from_callable(function));
            }
            Stmt::Return { value, .. } => {
                return Err(RuntimeError::Return(self.evaluate(value)?));
            }
        }
        Ok(())
    }

    pub fn execute_block(&mut self, statements: &[Stmt], env: EnvironmentPointer) -> Result<()> {
        let previous = self.environment.clone();
        self.environment = env;

        for stmt in statements {
            match self.execute(stmt) {
                Ok(_) => continue,
                Err(err) => {
                    self.environment = previous;
                    return Err(err);
                }
            }
        }

        self.environment = previous;
        Ok(())
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Object> {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;

                match operator.ty {
                    TokenTy::Plus => {
                        if let Ok((left, right)) =
                            Self::check_number_operands(operator, &left, &right)
                        {
                            Ok((left + right).into())
                        } else {
                            match (left, right) {
                                (
                                    Object::Literal(Literal::String(left)),
                                    Object::Literal(Literal::String(right)),
                                ) => Ok([left, right].join("").into()),
                                _ => Err(RuntimeError::Custom(
                                    operator.clone(),
                                    "Operands must be two numbers or two strings.".into(),
                                )),
                            }
                        }
                    }
                    TokenTy::Minus => {
                        let (left, right) = Self::check_number_operands(operator, &left, &right)?;
                        Ok((left - right).into())
                    }
                    TokenTy::Star => {
                        let (left, right) = Self::check_number_operands(operator, &left, &right)?;
                        Ok((left * right).into())
                    }
                    TokenTy::Slash => {
                        let (left, right) = Self::check_number_operands(operator, &left, &right)?;
                        Ok((left / right).into())
                    }
                    TokenTy::Greater => {
                        let (left, right) = Self::check_number_operands(operator, &left, &right)?;
                        Ok((left > right).into())
                    }
                    TokenTy::GreaterEqual => {
                        let (left, right) = Self::check_number_operands(operator, &left, &right)?;
                        Ok((left >= right).into())
                    }
                    TokenTy::Less => {
                        let (left, right) = Self::check_number_operands(operator, &left, &right)?;
                        Ok((left < right).into())
                    }
                    TokenTy::LessEqual => {
                        let (left, right) = Self::check_number_operands(operator, &left, &right)?;
                        Ok((left <= right).into())
                    }
                    TokenTy::EqualEqual => Ok(Self::is_equal(left, right).into()),
                    TokenTy::BangEqual => Ok((!Self::is_equal(left, right)).into()),
                    _ => unreachable!(),
                }
            }
            Expr::Grouping(expr) => self.evaluate(expr),
            Expr::Literal(lit) => Ok(Object::Literal(lit.clone())),
            Expr::Unary { operator, right } => {
                let right = self.evaluate(right)?;
                match operator.ty {
                    TokenTy::Minus => {
                        let right = Self::check_number_operand(operator, &right)?;
                        Ok((-right).into())
                    }
                    TokenTy::Bang => {
                        let right = Self::is_truthy(&right);
                        Ok((!right).into())
                    }
                    _ => unreachable!(),
                }
            }
            Expr::Variable(name) => Ok(self.environment.get(name)?),
            Expr::Assign { name, value } => {
                let value = self.evaluate(value)?;
                self.environment.assign(name, value.clone())?;
                Ok(value)
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;

                if operator.ty == TokenTy::Or {
                    if Self::is_truthy(&left) {
                        return Ok(left);
                    }
                } else if !Self::is_truthy(&left) {
                    return Ok(left);
                }

                self.evaluate(right)
            }
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                let callee = self.evaluate(callee)?;

                let arguments = arguments
                    .iter()
                    .map(|arg| self.evaluate(arg))
                    .collect::<Result<Vec<_>>>()?;

                if let Object::Callable(function) = callee {
                    if arguments.len() == function.arity() {
                        Ok(function.call(self, arguments)?)
                    } else {
                        Err(RuntimeError::Custom(
                            paren.clone(),
                            format!(
                                "Expected {} arguments but got {}.",
                                function.arity(),
                                arguments.len()
                            )
                            .into(),
                        ))
                    }
                } else {
                    Err(RuntimeError::Custom(
                        paren.clone(),
                        "Can only call functions and methods.".into(),
                    ))
                }
            }
        }
    }

    fn is_equal(left: Object, right: Object) -> bool {
        left == right
    }

    fn is_truthy(obj: &Object) -> bool {
        match obj {
            Object::Literal(Literal::Nil) => false,
            Object::Literal(Literal::Boolean(b)) => *b,
            _ => true,
        }
    }

    fn check_number_operand(operator: &Token, operand: &Object) -> Result<f64> {
        if let Object::Literal(Literal::Number(n)) = *operand {
            Ok(n)
        } else {
            Err(RuntimeError::Custom(
                operator.clone(),
                "Operand must be a number.".into(),
            ))
        }
    }

    fn check_number_operands(
        operator: &Token,
        left: &Object,
        right: &Object,
    ) -> Result<(f64, f64)> {
        match (left, right) {
            (Object::Literal(Literal::Number(left)), Object::Literal(Literal::Number(right))) => {
                Ok((*left, *right))
            }
            _ => Err(RuntimeError::Custom(
                operator.clone(),
                "Operands must be numbers.".into(),
            )),
        }
    }
}

pub type Result<T> = std::result::Result<T, RuntimeError>;

pub enum RuntimeError {
    // a hack
    Return(Object),
    Custom(Token, std::borrow::Cow<'static, str>),
}
