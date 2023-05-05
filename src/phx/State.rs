use crate::phx::internal::ffi;
use crate::phx::Common::*;
use crate::phx::Math::Vec3;

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
pub extern "C" fn State_ToString(mut state: State) -> *const libc::c_char {
    if state == State_Null {
        return c_str!("State_Null");
    }
    let states: [State; 4] = [State_Changed, State_Pressed, State_Down, State_Released];
    let names: [&'static str; 4] = [
        "State_Changed",
        "State_Pressed",
        "State_Down",
        "State_Released",
    ];

    let mut output = String::new();
    for i in 0..states.len() {
        if state & states[i] == states[i] {
            if output.len() != 0 {
                output += " | ";
            }
            output += names[i];
            state &= !states[i];
        }
    }
    if state != 0 {
        if output.len() != 0 {
            output += " | ";
        }
        output += format!("Unknown ({})", state).as_str();
    }

    ffi::StaticString!(output)
}
