mod convert;
mod memory;

pub use convert::*;
pub use memory::*;

#[macro_export]
macro_rules! static_cstring {
    ($str:expr) => {
        {
            use std::borrow::Borrow;
            thread_local! { static STRING_BUF: std::cell::RefCell<Option<std::ffi::CString>> = Default::default(); }
            STRING_BUF.replace(Some($str));
            STRING_BUF.with_borrow(|buf| buf.borrow().as_ref().unwrap().as_ptr())
        }
    };
}

#[macro_export]
macro_rules! static_string {
    ($str:expr) => {
        internal::static_cstring!(std::ffi::CString::new($str).unwrap());
    };
}

#[macro_export]
macro_rules! MemNew {
    ($x:ty) => {
        MemAlloc(std::mem::size_of::<$x>()) as *mut $x
    };
}

#[macro_export]
macro_rules! MemNewZero {
    ($x:ty) => {
        MemAllocZero(std::mem::size_of::<$x>()) as *mut $x
    };
}

#[macro_export]
macro_rules! MemNewArray {
    ($x:ty, $s:expr) => {
        MemAlloc(std::mem::size_of::<$x>().wrapping_mul($s as usize)) as *mut $x
    };
}

#[macro_export]
macro_rules! MemNewArrayZero {
    ($x:ty, $s:expr) => {
        MemAllocZero(std::mem::size_of::<$x>().wrapping_mul($s as usize)) as *mut $x
    };
}

#[macro_export]
macro_rules! MemDelete {
    ($v:ident) => {
        $v.drop_in_place();
        MemFree($v as *mut _)
    };
}
