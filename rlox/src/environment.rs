use std::{
    cell::{RefCell},
    collections::{hash_map::Entry, HashMap},
    rc::Rc,
};

use crate::{interpreter::RuntimeError, object::Object, token::Token};

pub type EnvironmentPointer<'a> = Rc<RefCell<Environment<'a>>>;

#[derive(Debug, Default)]
pub struct Environment<'a> {
    enclosing: Option<EnvironmentPointer<'a>>,
    values: HashMap<String, Object>,
}

impl<'a> Environment<'a> {
    pub fn new(enclosing: EnvironmentPointer<'a>) -> EnvironmentPointer<'a> {
        Rc::new(RefCell::new(Self {
            enclosing: Some(enclosing),
            ..Default::default()
        }))
    }

    pub fn define(&mut self, name: String, value: Object) -> &mut Object {
        self.values.entry(name).or_insert(value)
    }

    pub fn get(&self, name: &Token) -> Result<Object, RuntimeError> {
        if let Some(obj) = self.values.get(&name.lexeme) {
            Ok(obj.clone())
        } else if let Some(enclosing) = self.enclosing.as_ref() {
            Ok(enclosing.borrow().get(name)?)
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
                    enclosing.borrow_mut().assign(name, value)
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
