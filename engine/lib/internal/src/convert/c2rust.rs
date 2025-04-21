use std::ffi::CString;

pub trait ConvertIntoString {
    fn as_str(&self) -> &str;

    fn as_string(&self) -> String {
        self.as_str().to_string()
    }

    fn as_cstring(&self) -> CString;
}

impl ConvertIntoString for *const libc::c_char {
    fn as_str(&self) -> &str {
        if self.is_null() {
            return "<null>";
        }

        unsafe { std::str::from_utf8_unchecked(std::ffi::CStr::from_ptr(*self).to_bytes()) }
    }

    fn as_cstring(&self) -> CString {
        let cstr = unsafe { std::ffi::CStr::from_ptr(*self) };

        cstr.into()
    }
}
