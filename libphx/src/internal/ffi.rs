use crate::*;
use std::any::TypeId;
use std::collections::HashMap;
use std::str;

pub use std::ffi::{CStr, CString};

// pub fn NewCString(s: String) -> CString {
//     CString::new(s).unwrap()
// }

// pub fn PtrAsSlice<'a>(ptr: *const libc::c_char) -> &'a str {
//     unsafe { str::from_utf8_unchecked(CStr::from_ptr(ptr).to_bytes()) }
// }

// pub fn PtrAsString(ptr: *const libc::c_char) -> String {
//     PtrAsSlice(ptr).to_string()
// }

// pub fn SliceToNewCStr(s: &str) -> *mut libc::c_char {
//     unsafe {
//         let ptr = StrAlloc(s.len() + 1);
//         MemCpy(ptr as *mut _, s.as_ptr() as *mut _, s.len());
//         *ptr.offset(s.len() as isize) = 0;
//         ptr
//     }
// }

macro_rules! static_string {
    ($str:expr) => {
        unsafe {
            static mut STRING_BUF: Option<std::ffi::CString> = Option::None;

            STRING_BUF = Some(std::ffi::CString::new($str).unwrap());

            STRING_BUF.as_ref().unwrap().as_ptr()
        }
    };
}
pub(crate) use static_string;

// macro_rules! static_cstring {
//     ($str:expr) => {{
//         static mut STRING_BUF: CString = CString::new();

//         STRING_BUF = $str.into();

//         STRING_BUF.as_ptr()
//     }};
// }
// pub(crate) use static_cstring;
