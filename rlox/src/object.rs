use crate::{literal::Literal, lox_callable::LoxCallable};

#[derive(Debug, Clone)]
pub enum Object {
    Literal(Literal),
    Callable(Box<dyn LoxCallable>),
}

impl Object {
    pub fn from_callable<T: 'static + LoxCallable>(callable: T) -> Self {
        Object::Callable(Box::new(callable))
    }
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Literal(lit) => lit.fmt(f),
            Object::Callable(callable) => callable.fmt(f),
        }
    }
}

impl<T: Into<Literal>> From<T> for Object {
    fn from(lit: T) -> Self {
        Self::Literal(lit.into())
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Literal(l0), Self::Literal(r0)) => l0 == r0,
            (Self::Callable(l0), Self::Callable(r0)) => l0 == r0,
            _ => false,
        }
    }
}
