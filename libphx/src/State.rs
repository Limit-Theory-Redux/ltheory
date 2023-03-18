use crate::internal::Memory::*;
use glam::Vec3;
use libc;

extern "C" {
}
pub type State = i32;

#[no_mangle]
pub static State_Null: State = 0_i32 << 0_i32;

#[no_mangle]
pub static State_Changed: State = 1_i32 << 0_i32;

#[no_mangle]
pub static State_Pressed: State = 1_i32 << 1_i32;

#[no_mangle]
pub static State_Down: State = 1_i32 << 2_i32;

#[no_mangle]
pub static State_Released: State = 1_i32 << 3_i32;

#[no_mangle]
pub unsafe extern "C" fn State_ToString(mut state: State) -> *const libc::c_char {
    static mut buffer: [libc::c_char; 512] = [0; 512];
    if state == State_Null {
        return b"State_Null\0" as *const u8 as *const libc::c_char;
    }
    let mut states: [State; 4] = [State_Changed, State_Pressed, State_Down, State_Released];
    let mut names: [*const libc::c_char; 4] = [
        b"State_Changed\0" as *const u8 as *const libc::c_char,
        b"State_Pressed\0" as *const u8 as *const libc::c_char,
        b"State_Down\0" as *const u8 as *const libc::c_char,
        b"State_Released\0" as *const u8 as *const libc::c_char,
    ];
    let mut start: *mut libc::c_char = buffer.as_mut_ptr();
    let mut sep: *const libc::c_char = b"\0" as *const u8 as *const libc::c_char;
    let mut len: i32 = 0_i32;
    let mut i: i32 = 0_i32;
    while i
        < (::core::mem::size_of::<[State; 4]>()).wrapping_div(::core::mem::size_of::<State>())
            as i32
    {
        if state & states[i as usize] == states[i as usize] {
            len += libc::snprintf(
                start.offset(len as isize),
                ((::core::mem::size_of::<[libc::c_char; 512]>())
                    .wrapping_div(::core::mem::size_of::<libc::c_char>()) as i32
                    - len) as usize,
                b"%s%s\0" as *const u8 as *const libc::c_char,
                sep,
                names[i as usize],
            );
            sep = b" | \0" as *const u8 as *const libc::c_char;
            state &= !states[i as usize];
        }
        i += 1;
    }
    if state != 0_i32 {
        len += libc::snprintf(
            start.offset(len as isize),
            ((::core::mem::size_of::<[libc::c_char; 512]>())
                .wrapping_div(::core::mem::size_of::<libc::c_char>()) as i32
                - len) as usize,
            b"%sUnknown (%i)\0" as *const u8 as *const libc::c_char,
            sep,
            state,
        );
    }
    buffer.as_mut_ptr() as *const libc::c_char
}
