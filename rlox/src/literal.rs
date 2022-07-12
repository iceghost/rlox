use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    String(Cow<'static, str>),
    Boolean(bool),
    Nil,
}

impl From<f64> for Literal {
    fn from(n: f64) -> Self {
        Self::Number(n)
    }
}

impl From<String> for Literal {
    fn from(s: String) -> Self {
        Self::String(s.into())
    }
}

impl From<bool> for Literal {
    fn from(b: bool) -> Self {
        Self::Boolean(b)
    }
}

impl From<()> for Literal {
    fn from(_: ()) -> Self {
        Self::Nil
    }
}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Number(n) => n.fmt(f),
            Literal::String(s) => s.fmt(f),
            Literal::Boolean(b) => b.fmt(f),
            Literal::Nil => "nil".fmt(f),
        }
    }
}
