use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::Vec3;
use libc;

pub type Modifier = i32;

#[no_mangle]
pub static Modifier_Null: Modifier = 0 << 0;

#[no_mangle]
pub static Modifier_Alt: Modifier = 1 << 0;

#[no_mangle]
pub static Modifier_Ctrl: Modifier = 1 << 1;

#[no_mangle]
pub static Modifier_Shift: Modifier = 1 << 2;

#[no_mangle]
pub unsafe extern "C" fn Modifier_ToString(mut modifier: Modifier) -> *const libc::c_char {
    static mut buffer: [libc::c_char; 512] = [0; 512];
    if modifier == Modifier_Null {
        return c_str!("Modifier_Null");
    }
    let modifiers: [Modifier; 3] = [Modifier_Alt, Modifier_Ctrl, Modifier_Shift];
    let names: [*const libc::c_char; 3] = [
        c_str!("Modifier_Alt"),
        c_str!("Modifier_Ctrl"),
        c_str!("Modifier_Shift"),
    ];
    let start: *mut libc::c_char = buffer.as_mut_ptr();
    let mut sep: *const libc::c_char = c_str!("");
    let mut len: i32 = 0;
    let mut i: i32 = 0;
    while i < modifiers.len() as i32 {
        if modifier & modifiers[i as usize] == modifiers[i as usize] {
            len += libc::snprintf(
                start.offset(len as isize),
                (buffer.len() as i32 - len) as usize,
                c_str!("%s%s"),
                sep,
                names[i as usize],
            );
            sep = c_str!(" | ");
            modifier &= !modifiers[i as usize];
        }
        i += 1;
    }
    if modifier != 0 {
        len += libc::snprintf(
            start.offset(len as isize),
            (buffer.len() as i32 - len) as usize,
            c_str!("%sUnknown (%i)"),
            sep,
            modifier,
        );
    }
    buffer.as_mut_ptr() as *const libc::c_char
}
