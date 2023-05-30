use std::ffi::{CString, NulError};

use crate::Convert;

impl Convert for String {
    type T = *const libc::c_char;

    fn convert(&self) -> Self::T {
        let c_res = CString::new(self.as_str()).expect("Cannot convert String to CString");

        let lua_res = c_res.as_ptr();

        // Lua is responsible of cleaning memory
        std::mem::forget(c_res);

        lua_res
    }
}

impl Convert for str {
    type T = *const libc::c_char;

    fn convert(&self) -> Self::T {
        let c_res = CString::new(self).expect("Cannot convert &str to CString");

        let lua_res = c_res.as_ptr();

        // Lua is responsible of cleaning memory
        std::mem::forget(c_res);

        lua_res
    }
}
