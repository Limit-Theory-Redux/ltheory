use crate::internal::*;

impl ConvertIntoString for *const libc::c_char {
    fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(std::ffi::CStr::from_ptr(*self).to_bytes()) }
    }

    fn as_cstring(&self) -> CString {
        let cstr = unsafe { std::ffi::CStr::from_ptr(*self) };

        cstr.into()
    }
}
