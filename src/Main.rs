#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]
#![feature(extern_types)]
use ::phx::*;
extern "C" {
    pub type lua_State;
    fn Directory_Change(cwd: cstr) -> bool;
    fn Fatal(_: cstr, _: ...);
    fn Engine_Init(glVersionMajor: libc::c_int, glVersionMinor: libc::c_int);
    fn Engine_Free();
    fn File_Exists(path: cstr) -> bool;
    fn Lua_Create() -> *mut Lua;
    fn Lua_Free(_: *mut Lua);
    fn Lua_DoFile(_: *mut Lua, name: cstr);
    fn Lua_SetBool(_: *mut Lua, name: cstr, _: bool);
    fn Lua_SetNumber(_: *mut Lua, name: cstr, _: libc::c_double);
    fn Lua_SetStr(_: *mut Lua, name: cstr, _: cstr);
}
pub type cstr = *const libc::c_char;
pub type Lua = lua_State;
unsafe fn main_0(
    mut argc: libc::c_int,
    mut argv: *mut *mut libc::c_char,
) -> libc::c_int {
    Engine_Init(2 as libc::c_int, 1 as libc::c_int);
    let mut lua: *mut Lua = Lua_Create();
    let mut entryPoint: *const libc::c_char = b"./script/Main.lua\0" as *const u8
        as *const libc::c_char;
    if !File_Exists(entryPoint) {
        Directory_Change(b"../\0" as *const u8 as *const libc::c_char);
        if !File_Exists(entryPoint) {
            Fatal(
                b"can't find script entrypoint <%s>\0" as *const u8
                    as *const libc::c_char,
                entryPoint,
            );
        }
    }
    Lua_SetBool(
        lua,
        b"__debug__\0" as *const u8 as *const libc::c_char,
        0 as libc::c_int > 0 as libc::c_int,
    );
    Lua_SetBool(
        lua,
        b"__embedded__\0" as *const u8 as *const libc::c_char,
        1 as libc::c_int != 0,
    );
    Lua_SetNumber(
        lua,
        b"__checklevel__\0" as *const u8 as *const libc::c_char,
        0 as libc::c_int as libc::c_double,
    );
    if argc >= 2 as libc::c_int {
        Lua_SetStr(
            lua,
            b"__app__\0" as *const u8 as *const libc::c_char,
            *argv.offset(1 as libc::c_int as isize) as cstr,
        );
    }
    Lua_DoFile(lua, b"./script/Main\0" as *const u8 as *const libc::c_char);
    Lua_Free(lua);
    Engine_Free();
    return 0 as libc::c_int;
}
pub fn main() {
    let mut args: Vec::<*mut libc::c_char> = Vec::new();
    for arg in ::std::env::args() {
        args.push(
            (::std::ffi::CString::new(arg))
                .expect("Failed to convert argument into CString.")
                .into_raw(),
        );
    }
    args.push(::core::ptr::null_mut());
    unsafe {
        ::std::process::exit(
            main_0(
                (args.len() - 1) as libc::c_int,
                args.as_mut_ptr() as *mut *mut libc::c_char,
            ) as i32,
        )
    }
}
