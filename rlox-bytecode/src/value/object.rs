use std::any::Any;
use std::fmt::Display;
use std::ptr::NonNull;

#[repr(transparent)]
pub struct Object<T: ?Sized>(NonNull<Inner<T>>);

impl<T: 'static> From<Object<T>> for Object<dyn Any> {
    #[inline]
    fn from(obj: Object<T>) -> Self {
        Self(obj.0)
    }
}

impl<T: ?Sized> Clone for Object<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T: ?Sized> Copy for Object<T> {}

impl<T: ?Sized> std::ops::Deref for Object<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &self.0.as_ref().data }
    }
}

impl<T> Object<T> {
    // object should always live in the heap
    pub fn new(data: T) -> Self {
        let inner = Box::new(Inner::new(data));
        let ptr = unsafe { NonNull::new_unchecked(Box::into_raw(inner)) };
        Self(ptr)
    }
}

impl<T: ?Sized> Object<T> {
    pub fn set_next(&mut self, obj: Option<Object<dyn Any>>) {
        unsafe {
            self.0.as_mut().next = obj;
        }
    }

    pub fn next(&self) -> Option<Object<dyn Any>> {
        unsafe { self.0.as_ref().next }
    }

    pub fn drop_inner(&self) {
        unsafe {
            let ptr = self.0.as_ptr();
            let _ = Box::from_raw(ptr);
        }
    }
}

impl<T: PartialEq> PartialEq for Object<T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.0.as_ref().data == other.0.as_ref().data }
    }
}

impl<T: Display> Display for Object<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe { self.0.as_ref().data.fmt(f) }
    }
}

struct Inner<T: ?Sized> {
    next: Option<Object<dyn Any>>,
    data: T,
}

impl<T: ?Sized> std::ops::Deref for Inner<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> Inner<T> {
    pub fn new(data: T) -> Self {
        let next = None;
        Self { data, next }
    }
}
