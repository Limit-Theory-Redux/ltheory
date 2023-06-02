use crate::Convert;

impl Convert for *const libc::c_char {
    type T<'a> = &'a str where Self: 'a;

    fn convert(&self) -> Self::T<'_> {
        unsafe { std::str::from_utf8_unchecked(std::ffi::CStr::from_ptr(*self).to_bytes()) }
    }
}
