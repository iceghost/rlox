use crate::{
    interpreter::{Interpreter, RuntimeError},
    object::Object,
};

pub trait LoxCallable: std::fmt::Debug + BoxedPartialEq + BoxedClone {
    fn arity(&self) -> usize;
    fn call(&self, intpr: &mut Interpreter, args: Vec<Object>) -> Result<Object, RuntimeError>;
}

pub trait BoxedClone {
    fn clone_box(&self) -> Box<dyn LoxCallable>;
}

impl<T: 'static + Clone + LoxCallable> BoxedClone for T {
    fn clone_box(&self) -> Box<dyn LoxCallable> {
        Box::new(self.clone())
    }
}

pub trait BoxedPartialEq {
    fn eq_box(&self, other: &dyn LoxCallable) -> bool;
}

impl<T: 'static + LoxCallable> BoxedPartialEq for T {
    fn eq_box(&self, other: &dyn LoxCallable) -> bool {
        std::ptr::eq(
            self as *const _ as *const (),
            other as *const _ as *const (),
        )
    }
}

impl Clone for Box<dyn LoxCallable> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl PartialEq for Box<dyn LoxCallable> {
    fn eq(&self, other: &Self) -> bool {
        self.eq_box(other.as_ref())
    }
}
