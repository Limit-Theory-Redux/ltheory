use crate::internal::Memory::*;
use crate::Math::Vec3;
use libc;

macro_rules! unwrap_or_return {
    ($value: ident) => {
        match $value {
            Some(v) => v,
            _ => return,
        }
    };
    ($value: ident, $return: expr) => {
        match $value {
            Some(v) => v,
            _ => return $return,
        }
    };
}
pub(crate) use unwrap_or_return;

#[no_mangle]
pub unsafe extern "C" fn Fatal(mut format: *const libc::c_char, mut args: ...) -> ! {
    let mut s = String::new();
    let _ = printf_compat::format(
        format,
        args.as_va_list(),
        printf_compat::output::fmt_write(&mut s),
    );
    println!("FATAL: {}", s);
    libc::abort();
}

#[no_mangle]
pub unsafe extern "C" fn Warn(mut format: *const libc::c_char, mut args: ...) {
    let mut s = String::new();
    let _ = printf_compat::format(
        format,
        args.as_va_list(),
        printf_compat::output::fmt_write(&mut s),
    );
    println!("WARN: {}", s);
}
