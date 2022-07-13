use std::rc::Rc;

use crate::{
    environment::EnvironmentPointer,
    interpreter::{Interpreter, RuntimeError},
    lox_callable::LoxCallable,
    object::Object,
    stmt::StmtFunction,
};

#[derive(Clone)]
pub struct LoxFunction {
    closure: EnvironmentPointer,
    declaration: Rc<StmtFunction>,
}

impl LoxFunction {
    pub fn new(declaration: Rc<StmtFunction>, closure: EnvironmentPointer) -> Self {
        Self {
            declaration,
            closure,
        }
    }
}

impl std::fmt::Debug for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "<fn {}>", self.declaration.name.lexeme)
    }
}

impl LoxCallable for LoxFunction {
    fn arity(&self) -> usize {
        self.declaration.params.len()
    }

    fn call(&self, intpr: &mut Interpreter, args: Vec<Object>) -> Result<Object, RuntimeError> {
        let mut environment = EnvironmentPointer::new(self.closure.clone());
        for (token, value) in self.declaration.params.iter().zip(args.into_iter()) {
            environment.define(token.lexeme.to_owned(), value.clone());
        }
        match intpr.execute_block(&self.declaration.body, environment) {
            Err(RuntimeError::Return(val)) => Ok(val),
            otherwise => otherwise.map(|_| ().into()),
        }
    }
}
