use crate::{
    environment::{Environment, EnvironmentPointer},
    expr::Expr,
    literal::Literal,
    object::Object,
    stmt::Stmt,
    token::Token,
    token_type::TokenTy,
};

#[derive(Default)]
pub struct Interpreter<'a> {
    environment: EnvironmentPointer<'a>,
}

impl<'a> Interpreter<'a> {
    pub fn interpret(&mut self, statements: &Vec<Stmt>) -> Result<()> {
        for statement in statements {
            self.execute(statement)?;
        }
        Ok(())
    }
    fn execute(&mut self, stmt: &Stmt) -> Result<()> {
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
                self.environment
                    .borrow_mut()
                    .define(name.lexeme.to_owned(), value);
            }
            Stmt::Block(stmts) => {
                self.execute_block(stmts, Environment::new(self.environment.clone()))?;
            }
        }
        Ok(())
    }

    fn execute_block(&mut self, statements: &[Stmt], env: EnvironmentPointer<'a>) -> Result<()> {
        let previous = self.environment.clone();
        self.environment = env;

        for stmt in statements {
            match self.execute(stmt) {
                Ok(_) => continue,
                Err(err) => {
                    self.environment = previous;
                    return Err(err);
                },
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
                            Ok(Object::Literal(Literal::Number(left + right)))
                        } else {
                            match (left, right) {
                                (
                                    Object::Literal(Literal::String(left)),
                                    Object::Literal(Literal::String(right)),
                                ) => Ok(Object::Literal(Literal::String([left, right].join("")))),
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
                    TokenTy::EqualEqual => Ok(Object::Literal(Literal::Boolean(Self::is_equal(
                        left, right,
                    )))),
                    TokenTy::BangEqual => Ok(Object::Literal(Literal::Boolean(!Self::is_equal(
                        left, right,
                    )))),
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
            Expr::Variable(name) => Ok(self.environment.borrow().get(name)?),
            Expr::Assign { name, value } => {
                let value = self.evaluate(value)?;
                self.environment.borrow_mut().assign(name, value.clone())?;
                Ok(value)
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
    Custom(Token, std::borrow::Cow<'static, str>),
}
