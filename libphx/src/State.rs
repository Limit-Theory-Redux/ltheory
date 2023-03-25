use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::Vec3;
use libc;

pub type State = i32;

#[no_mangle]
pub static State_Null: State = 0 << 0;

#[no_mangle]
pub static State_Changed: State = 1 << 0;

#[no_mangle]
pub static State_Pressed: State = 1 << 1;

#[no_mangle]
pub static State_Down: State = 1 << 2;

#[no_mangle]
pub static State_Released: State = 1 << 3;

#[no_mangle]
pub unsafe extern "C" fn State_ToString(mut state: State) -> *const libc::c_char {
    static mut buffer: [libc::c_char; 512] = [0; 512];
    if state == State_Null {
        return c_str!("State_Null");
    }
    let mut states: [State; 4] = [State_Changed, State_Pressed, State_Down, State_Released];
    let mut names: [*const libc::c_char; 4] = [
        c_str!("State_Changed"),
        c_str!("State_Pressed"),
        c_str!("State_Down"),
        c_str!("State_Released"),
    ];
    let mut start: *mut libc::c_char = buffer.as_mut_ptr();
    let mut sep: *const libc::c_char = c_str!("");
    let mut len: i32 = 0;
    let mut i = 0;
    while i < states.len() {
        if state & states[i as usize] == states[i as usize] {
            len += libc::snprintf(
                start.offset(len as isize),
                (buffer.len() as i32 - len) as usize,
                c_str!("%s%s"),
                sep,
                names[i as usize],
            );
            sep = c_str!(" | ");
            state &= !states[i as usize];
        }
        i += 1;
    }
    if state != 0 {
        len += libc::snprintf(
            start.offset(len as isize),
            (buffer.len() as i32 - len) as usize,
            c_str!("%sUnknown (%i)"),
            sep,
            state,
        );
    }
    buffer.as_mut_ptr() as *const libc::c_char
}
