use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    interpreter::{Interpreter, RuntimeError},
    lox_callable::LoxCallable,
    object::Object,
};

#[derive(Clone, PartialEq, Eq)]
pub struct Clock;

impl std::fmt::Debug for Clock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<native fn clock>")
    }
}

impl LoxCallable for Clock {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _: &mut Interpreter, _: Vec<Object>) -> Result<Object, RuntimeError> {
        Ok(SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64()
            .into())
    }
}
