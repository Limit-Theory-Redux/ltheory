use crate::{common::*, static_string, Convert};

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
pub extern "C" fn Modifier_ToString(mut modifier: Modifier) -> *const libc::c_char {
    static_string!(modifier_to_string(modifier))
}

pub fn modifier_to_string(mut modifier: Modifier) -> String {
    if modifier == Modifier_Null {
        return "Modifier_Null".into();
    }
    let modifiers: [Modifier; 3] = [Modifier_Alt, Modifier_Ctrl, Modifier_Shift];
    let names: [&str; 3] = ["Modifier_Alt", "Modifier_Ctrl", "Modifier_Shift"];

    let mut output = String::new();
    for i in 0..modifiers.len() {
        if modifier & modifiers[i] == modifiers[i] {
            if output.len() != 0 {
                output += " | ";
            }
            output += names[i];
            modifier &= !modifiers[i];
        }
    }
    if modifier != 0 {
        if output.len() != 0 {
            output += " | ";
        }
        output += format!("Unknown ({modifier})").as_str();
    }

    output
}
