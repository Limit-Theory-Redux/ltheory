use ::libc;
use crate::internal::Memory::*;
extern "C" {
    fn snprintf(
        _: *mut libc::c_char,
        _: libc::size_t,
        _: *const libc::c_char,
        _: ...
    ) -> libc::c_int;
}
pub type int32_t = libc::c_int;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type State = int32;
#[no_mangle]
pub static mut State_Null: State = (0 as libc::c_int) << 0 as libc::c_int;
#[no_mangle]
pub static mut State_Changed: State = (1 as libc::c_int) << 0 as libc::c_int;
#[no_mangle]
pub static mut State_Pressed: State = (1 as libc::c_int) << 1 as libc::c_int;
#[no_mangle]
pub static mut State_Down: State = (1 as libc::c_int) << 2 as libc::c_int;
#[no_mangle]
pub static mut State_Released: State = (1 as libc::c_int) << 3 as libc::c_int;
#[no_mangle]
pub unsafe extern "C" fn State_ToString(mut state: State) -> cstr {
    static mut buffer: [libc::c_char; 512] = [0; 512];
    if state == State_Null {
        return b"State_Null\0" as *const u8 as *const libc::c_char;
    }
    let mut states: [State; 4] = [
        State_Changed,
        State_Pressed,
        State_Down,
        State_Released,
    ];
    let mut names: [cstr; 4] = [
        b"State_Changed\0" as *const u8 as *const libc::c_char,
        b"State_Pressed\0" as *const u8 as *const libc::c_char,
        b"State_Down\0" as *const u8 as *const libc::c_char,
        b"State_Released\0" as *const u8 as *const libc::c_char,
    ];
    let mut start: *mut libc::c_char = buffer.as_mut_ptr();
    let mut sep: *const libc::c_char = b"\0" as *const u8 as *const libc::c_char;
    let mut len: libc::c_int = 0 as libc::c_int;
    let mut i: libc::c_int = 0 as libc::c_int;
    while i
        < (::core::mem::size_of::<[State; 4]>())
            .wrapping_div(::core::mem::size_of::<State>())
            as libc::c_int
    {
        if state & states[i as usize] == states[i as usize] {
            len
                += snprintf(
                    start.offset(len as isize),
                    ((::core::mem::size_of::<[libc::c_char; 512]>())
                        .wrapping_div(
                            ::core::mem::size_of::<libc::c_char>(),
                        ) as libc::c_int - len) as usize,
                    b"%s%s\0" as *const u8 as *const libc::c_char,
                    sep,
                    names[i as usize],
                );
            sep = b" | \0" as *const u8 as *const libc::c_char;
            state &= !states[i as usize];
        }
        i += 1;
    }
    if state != 0 as libc::c_int {
        len
            += snprintf(
                start.offset(len as isize),
                ((::core::mem::size_of::<[libc::c_char; 512]>())
                    .wrapping_div(
                        ::core::mem::size_of::<libc::c_char>(),
                    ) as libc::c_int - len) as usize,
                b"%sUnknown (%i)\0" as *const u8 as *const libc::c_char,
                sep,
                state,
            );
    }
    return buffer.as_mut_ptr() as cstr;
}
