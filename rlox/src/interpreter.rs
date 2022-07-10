use std::any::Any;

use crate::{expr::Expr, literal::Literal, object::Object, token::Token, token_type::TokenTy, Lox};

pub struct Interpreter;

impl Interpreter {
    pub fn interpret(&self, expr: &Expr) -> Result<()> {
        let obj = self.evaluate(expr)?;
        println!("{obj}");
        Ok(())
    }
    fn evaluate(&self, expr: &Expr) -> Result<Object> {
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

type Result<T> = std::result::Result<T, RuntimeError>;

pub enum RuntimeError {
    Custom(Token, std::borrow::Cow<'static, str>),
}
