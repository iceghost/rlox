use std::{fmt::Display, ops::Deref};

#[derive(Clone, Copy, PartialEq)]
pub enum Value {
    Bool(bool),
    Double(f64),
    Nil,
}

#[allow(unused)]
impl Value {
    pub fn as_double(self) -> Option<f64> {
        if let Self::Double(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_bool(self) -> Option<bool> {
        if let Self::Bool(v) = self {
            Some(v)
        } else {
            None
        }
    }

    #[must_use]
    pub fn is_nil(self) -> bool {
        matches!(self, Self::Nil)
    }

    #[inline]
    pub fn is_truthy(self) -> bool {
        match self {
            Value::Bool(b) => b,
            Value::Nil => false,
            _ => true,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(b) => b.fmt(f),
            Value::Double(d) => d.fmt(f),
            Value::Nil => "nil".fmt(f),
        }
    }
}

impl From<bool> for Value {
    #[inline]
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<f64> for Value {
    #[inline]
    fn from(double: f64) -> Self {
        Self::Double(double)
    }
}

impl From<()> for Value {
    #[inline]
    fn from(_: ()) -> Self {
        Self::Nil
    }
}

#[derive(Default)]
pub struct Values(Vec<Value>);

impl Values {
    pub fn write(&mut self, value: Value) {
        self.0.push(value);
    }
}

impl Deref for Values {
    type Target = [Value];

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}
