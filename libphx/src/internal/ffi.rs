use crate::*;
use std::any::TypeId;
use std::collections::HashMap;
use std::str;

pub use std::ffi::{CStr, CString};

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

#[allow(unused_macros)]
macro_rules! static_cstring {
    ($str:expr) => {
        unsafe {
            static mut STRING_BUF: Option<std::ffi::CString> = Option::None;

            STRING_BUF = Some($str);

            STRING_BUF.as_ref().unwrap().as_ptr()
        }
    };
}
pub(crate) use static_cstring;
