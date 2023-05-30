use std::ffi::{CStr, CString};

use crate::Convert;

impl Convert for *const libc::c_char {
    type T = String;

    fn convert(&self) -> Self::T {
        let c_str = unsafe { CStr::from_ptr(*self) };
        let c_string = CString::from(c_str);

        c_string
            .into_string()
            .expect("Cannot convert CString to String")
    }
}
