use crate::internal::Memory::*;
use std::any::TypeId;
use std::collections::HashMap;
use std::str;

pub use std::ffi::{CStr, CString};

pub fn NewCString(s: String) -> CString {
    CString::new(s).unwrap()
}

pub fn PtrAsSlice<'a>(ptr: *const libc::c_char) -> &'a str {
    unsafe { str::from_utf8_unchecked(CStr::from_ptr(ptr).to_bytes()) }
}

pub fn PtrAsString(ptr: *const libc::c_char) -> String {
    PtrAsSlice(ptr).to_string()
}

pub fn SliceToNewCStr(s: &str) -> *mut libc::c_char {
    unsafe {
        let ptr = StrAlloc(s.len() + 1);
        MemCpy(ptr as *mut _, s.as_ptr() as *mut _, s.len());
        *ptr.offset(s.len() as isize) = 0;
        ptr
    }
}

macro_rules! StaticString {
    ($str:expr) => {{
        fn f() {}
        fn get_key_for<T: 'static>(_: T) -> std::any::TypeId {
            std::any::TypeId::of::<T>()
        }
        crate::internal::ffi::StaticCStringKey(
            get_key_for(f),
            std::ffi::CString::new($str).unwrap(),
        )
    }};
}
pub(crate) use StaticString;

macro_rules! StaticCString {
    ($str:expr) => {{
        fn f() {}
        fn get_key_for<T: 'static>(_: T) -> std::any::TypeId {
            std::any::TypeId::of::<T>()
        }
        crate::internal::ffi::StaticCStringKey(get_key_for(f), $str.into())
    }};
}
pub(crate) use StaticCString;

pub fn StaticCStringKey(key: TypeId, s: CString) -> *const libc::c_char {
    static mut STRINGS_MAP: Option<HashMap<TypeId, CString>> = None;
    unsafe {
        STRINGS_MAP.get_or_insert(HashMap::new());
        STRINGS_MAP.as_mut().unwrap().insert(key, s);
        STRINGS_MAP.as_ref().unwrap().get(&key).unwrap().as_ptr()
    }
}
