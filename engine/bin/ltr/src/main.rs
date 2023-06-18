// TODO: https://users.rust-lang.org/t/question-how-to-dllexport-nvoptimusenablement-symbol-to-auto-select-dedicated-nvidia-gpu/8617/3
// /* On Windows, request usage of the dedicated GPU if the machine switches
//  * between on-board and dedicated GPUs dynamically. Only works when exported
//  * by the exe, not when exported by a dll. */
//  #if WINDOWS
//  extern "C" {
//    __declspec(dllexport) ulong NvOptimusEnablement = 0x00000001;
//    __declspec(dllexport) int AmdPowerXpressRequestHighPerformance = 1;
//  }
// #endif

use std::ffi::CString;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "./script/Main.lua")]
    entry_point: String,
    #[arg(short, long)]
    app_name: Option<String>,
}

#[cfg_attr(not(windows), link(name = "phx", kind = "dylib"))]
#[cfg_attr(windows, link(name = "phx.dll", kind = "dylib"))]
extern "C" {
    fn Engine_Entry(entry_point: *const libc::c_char, app_name: *mut *const libc::c_char);
}

pub fn main() {
    let cli = Cli::parse();

    let entry_point = CString::new(cli.entry_point)
        .expect("Failed to convert entry_point argument into CString.")
        .into_raw();

    if let Some(app_name) = cli.app_name {
        let mut cstr = CString::new(app_name)
            .expect("Failed to convert app_name argument into CString.")
            .into_raw() as *const libc::c_char;
        let cstr_ptr: *mut *const libc::c_char = &mut cstr;

        unsafe { Engine_Entry(entry_point as *const libc::c_char, cstr_ptr) }
    } else {
        unsafe {
            Engine_Entry(
                entry_point as *const libc::c_char,
                std::ptr::null_mut::<*const libc::c_char>(),
            )
        }
    };
}
