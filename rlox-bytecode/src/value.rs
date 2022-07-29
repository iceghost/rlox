use std::{fmt::Display, ops::Deref};

pub struct Value(f64);
impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl From<f64> for Value {
    fn from(float: f64) -> Self {
        Self(float)
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
