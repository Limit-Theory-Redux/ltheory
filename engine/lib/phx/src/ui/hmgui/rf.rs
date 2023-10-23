use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

/// Rc/RefCell wrapper.
///
/// Note. Run with:
///
///    cargo +nightly r --target=x86_64-unknown-linux-gnu -Zbuild-std -Zbuild-std-features=core/debug_refcell
///
/// to see original borrowing place on panic.
/// Change target if you are not on Linux.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Rf<T>(Rc<RefCell<T>>);

impl<T> Rf<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }

    #[track_caller]
    #[inline]
    pub fn as_ref(&self) -> Ref<T> {
        self.0.borrow()
    }

    #[track_caller]
    #[inline]
    pub fn as_mut(&self) -> RefMut<T> {
        self.0.borrow_mut()
    }
}

impl<T> From<T> for Rf<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}
