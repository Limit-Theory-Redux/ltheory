use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

/// Rc/RefCell wrapper.
///
/// Note. Run with:
///
///    cargo +nightly r --target=x86_64-unknown-linux-gnu -Zbuild-std -Zbuild-std-features=core/debug_refcell
///
/// to see original borrowing place on panic.
///
/// Change target if you are not on Linux. See https://doc.rust-lang.org/nightly/rustc/platform-support.html
/// for the list of all possible targets.
#[derive(Default, Debug, PartialEq, Eq)]
#[must_use]
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

// Note: Using #[derive(Clone)] doesn't quite work correctly, because the Rust
// compiler erroneously expects T to also implement Clone, and that isn't
// always possible.
//
// See https://github.com/rust-lang/rust/issues/41481
impl<T> Clone for Rf<T> {
    fn clone(&self) -> Self {
        Rf(self.0.clone())
    }
}
