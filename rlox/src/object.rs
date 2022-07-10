use crate::literal::Literal;

#[derive(Debug, PartialEq)]
pub enum Object {
    Literal(Literal),
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Literal(lit) => lit.fmt(f),
        }
    }
}

impl<T: Into<Literal>> From<T> for Object {
    fn from(lit: T) -> Self {
        Self::Literal(lit.into())
    }
}
