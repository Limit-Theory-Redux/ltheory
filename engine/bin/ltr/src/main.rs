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

#[cfg_attr(not(windows), link(name = "phx", kind = "dylib"))]
#[cfg_attr(windows, link(name = "phx.dll", kind = "dylib"))]
extern "C" {
    fn Engine_Entry(argc: i32, argv: *mut *mut libc::c_char) -> i32;
}

pub fn main() {
    let mut args: Vec<*mut libc::c_char> = Vec::new();
    for arg in ::std::env::args() {
        args.push(
            (::std::ffi::CString::new(arg))
                .expect("Failed to convert argument into CString.")
                .into_raw(),
        );
    }
    args.push(::core::ptr::null_mut());
    unsafe {
        ::std::process::exit(Engine_Entry(
            (args.len() - 1) as i32,
            args.as_mut_ptr() as *mut *mut libc::c_char,
        ))
    }
}
