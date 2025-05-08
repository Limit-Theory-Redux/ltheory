mod convert;
mod memory;

pub use convert::*;
pub use memory::*;

/// Store C string in the static thread local storage.
/// Main use case - sending strings from Rust to Lua.
#[macro_export]
macro_rules! static_cstring {
    ($str:expr) => {
        {
            thread_local! { static STRING_BUF: std::cell::RefCell<std::ffi::CString> = Default::default(); }
            STRING_BUF.replace($str);
            STRING_BUF.with_borrow(|buf| buf.as_ptr())
        }
    };
}

/// Convert Rust string to C one and store it in the static thread local storage.
/// Maine use case - sending strings from Rust to Lua.
#[macro_export]
macro_rules! static_string {
    ($str:expr) => {
        internal::static_cstring!(std::ffi::CString::new($str).unwrap());
    };
}

#[macro_export]
macro_rules! MemNew {
    ($x:ty) => {
        mem_alloc(std::mem::size_of::<$x>()) as *mut $x
    };
}
