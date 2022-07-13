use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    rc::Rc,
};

use crate::{interpreter::RuntimeError, object::Object, token::Token};

#[derive(Debug, Default, Clone)]
pub struct EnvironmentPointer<'a>(Rc<RefCell<Environment<'a>>>);

impl<'a> EnvironmentPointer<'a> {
    pub fn new(enclosing: EnvironmentPointer<'a>) -> Self {
        Self(Rc::new(RefCell::new(Environment::new(enclosing))))
    }

    #[inline]
    pub fn define(&mut self, name: String, value: Object) {
        self.0.borrow_mut().define(name, value);
    }

    #[inline]
    pub fn get(&self, name: &Token) -> Result<Object, RuntimeError> {
        self.0.borrow().get(name)
    }

    #[inline]
    pub fn assign(&self, name: &Token, value: Object) -> Result<(), RuntimeError> {
        self.0.borrow_mut().assign(name, value)
    }
}

#[derive(Debug, Default)]
struct Environment<'a> {
    enclosing: Option<EnvironmentPointer<'a>>,
    values: HashMap<String, Object>,
}

impl<'a> Environment<'a> {
    pub fn new(enclosing: EnvironmentPointer<'a>) -> Self {
        Self {
            enclosing: Some(enclosing),
            ..Default::default()
        }
    }

    pub fn define(&mut self, name: String, value: Object) {
        self.values.entry(name).or_insert(value);
    }

    pub fn get(&self, name: &Token) -> Result<Object, RuntimeError> {
        if let Some(obj) = self.values.get(&name.lexeme) {
            Ok(obj.clone())
        } else if let Some(enclosing) = self.enclosing.as_ref() {
            Ok(enclosing.get(name)?)
        } else {
            Err(RuntimeError::Custom(
                name.clone(),
                format!("Undefined variable '{}'.", name.lexeme).into(),
            ))
        }
    }

    pub fn assign(&mut self, name: &Token, value: Object) -> Result<(), RuntimeError> {
        match self.values.entry(name.lexeme.to_owned()) {
            Entry::Occupied(mut entry) => {
                entry.insert(value);
                Ok(())
            }
            Entry::Vacant(_) => {
                if let Some(enclosing) = self.enclosing.as_mut() {
                    enclosing.assign(name, value)
                } else {
                    Err(RuntimeError::Custom(
                        name.clone(),
                        format!("Undefined variable '{}'.", name.lexeme).into(),
                    ))
                }
            }
        }
    }
}
