use std::ffi::CString;

// TODO: this is a copy of some code from phx::internal module. Move internal to a separate crate

pub trait ConvertIntoString {
    fn as_str(&self) -> &str;

    fn as_string(&self) -> String {
        self.as_str().to_string()
    }

    fn as_cstring(&self) -> CString;
}

impl ConvertIntoString for *const libc::c_char {
    fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(std::ffi::CStr::from_ptr(*self).to_bytes()) }
    }

    fn as_cstring(&self) -> CString {
        let cstr = unsafe { std::ffi::CStr::from_ptr(*self) };

        cstr.into()
    }
}

#[macro_export]
macro_rules! static_string {
    ($str:expr) => {
        unsafe {
            static mut STRING_BUF: Option<std::ffi::CString> = Option::None;

            STRING_BUF = Some(std::ffi::CString::new($str).unwrap());

            STRING_BUF.as_ref().unwrap().as_ptr()
        }
    };
}
