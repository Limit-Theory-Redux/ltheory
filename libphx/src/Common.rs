use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;

#[no_mangle]
pub unsafe extern "C" fn Fatal(mut format: *const libc::c_char, mut args: ...) {
    let mut s = String::new();
    let _ = printf_compat::format(format, args.as_va_list(), printf_compat::output::fmt_write(&mut s));
    println!("FATAL: {}", s);
}

#[no_mangle]
pub unsafe extern "C" fn Warn(mut format: *const libc::c_char, mut args: ...) {
    let mut s = String::new();
    let _ = printf_compat::format(format, args.as_va_list(), printf_compat::output::fmt_write(&mut s));
    println!("WARN: {}", s);
}
