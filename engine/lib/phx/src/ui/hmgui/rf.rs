use std::cell::{Ref, RefCell, RefMut};
use std::ops::Deref;
use std::rc::Rc;

#[derive(Clone, Default, PartialEq, Eq)]
pub struct Rf<T>(Rc<RefCell<T>>);

impl<T> Rf<T> {
    pub fn new(value: T) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }

    pub fn as_ref(&self) -> Ref<T> {
        self.0.borrow()
    }

    pub fn as_mut(&self) -> RefMut<T> {
        self.0.borrow_mut()
    }
}
