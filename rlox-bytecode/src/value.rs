use std::{
    fmt::Display,
    ops::{Add, Deref, Div, Mul, Neg, Sub},
};

#[derive(Clone, Copy)]
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

impl Add for Value {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let output = self.0 + rhs.0;
        output.into()
    }
}

impl Sub for Value {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        let output = self.0 - rhs.0;
        output.into()
    }
}

impl Mul for Value {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let output = self.0 * rhs.0;
        output.into()
    }
}

impl Div for Value {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        let output = self.0 / rhs.0;
        output.into()
    }
}

impl Neg for Value {
    type Output = Self;
    fn neg(self) -> Self::Output {
        let output = -self.0;
        output.into()
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
