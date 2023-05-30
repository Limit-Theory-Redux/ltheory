use crate::common::*;
use crate::lua_scheduler::*;
use crate::*;

use crate::resource::*;
use crate::resource_type::*;
use crate::signal::*;
use libc;

extern "C" {
    pub type lua_State;
    fn lua_close(L: *mut lua_State);
    fn lua_newthread(L: *mut lua_State) -> *mut lua_State;
    fn lua_gettop(L: *mut lua_State) -> i32;
    fn lua_settop(L: *mut lua_State, idx: i32);
    fn lua_xmove(from: *mut lua_State, to: *mut lua_State, n: i32);
    fn lua_type(L: *mut lua_State, idx: i32) -> i32;
    fn lua_typename(L: *mut lua_State, tp: i32) -> *const libc::c_char;
    fn lua_toboolean(L: *mut lua_State, idx: i32) -> i32;
    fn lua_tolstring(L: *mut lua_State, idx: i32, len: *mut usize) -> *const libc::c_char;
    fn lua_touserdata(L: *mut lua_State, idx: i32) -> *mut libc::c_void;
    fn lua_topointer(L: *mut lua_State, idx: i32) -> *const libc::c_void;
    fn lua_pushnumber(L: *mut lua_State, n: lua_Number);
    fn lua_pushstring(L: *mut lua_State, s: *const libc::c_char);
    fn lua_pushcclosure(L: *mut lua_State, fn_0: lua_CFunction, n: i32);
    fn lua_pushboolean(L: *mut lua_State, b: i32);
    fn lua_pushlightuserdata(L: *mut lua_State, p: *mut libc::c_void);
    fn lua_pushthread(L: *mut lua_State) -> i32;
    fn lua_getfield(L: *mut lua_State, idx: i32, k: *const libc::c_char);
    fn lua_rawgeti(L: *mut lua_State, idx: i32, n: i32);
    fn lua_setfield(L: *mut lua_State, idx: i32, k: *const libc::c_char);
    fn lua_call(L: *mut lua_State, nargs: i32, nresults: i32);
    fn lua_pcall(L: *mut lua_State, nargs: i32, nresults: i32, errfunc: i32) -> i32;
    fn lua_gc(L: *mut lua_State, what: i32, data: i32) -> i32;
    fn lua_error(L: *mut lua_State) -> i32;
    fn lua_getstack(L: *mut lua_State, level: i32, ar: *mut lua_Debug) -> i32;
    fn lua_getinfo(L: *mut lua_State, what: *const libc::c_char, ar: *mut lua_Debug) -> i32;
    fn lua_getlocal(L: *mut lua_State, ar: *const lua_Debug, n: i32) -> *const libc::c_char;
    fn lua_getupvalue(L: *mut lua_State, funcindex: i32, n: i32) -> *const libc::c_char;
    fn lua_sethook(L: *mut lua_State, func: lua_Hook, mask: i32, count: i32) -> i32;
    fn luaL_loadstring(L: *mut lua_State, s: *const libc::c_char) -> i32;
    fn luaL_callmeta(L: *mut lua_State, obj: i32, e: *const libc::c_char) -> i32;
    fn luaL_where(L: *mut lua_State, lvl: i32);
    fn luaL_ref(L: *mut lua_State, t: i32) -> i32;
    fn luaL_unref(L: *mut lua_State, t: i32, ref_0: i32);
    fn luaL_loadfile(L: *mut lua_State, filename: *const libc::c_char) -> i32;
    fn luaL_openlibs(L: *mut lua_State);
    fn luaL_newstate() -> *mut lua_State;
}
pub type va_list = *mut libc::c_char;
pub type lua_CFunction = Option<unsafe extern "C" fn(*mut lua_State) -> i32>;
pub type lua_Number = f64;
pub type lua_Integer = libc::ptrdiff_t;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct lua_Debug {
    pub event: i32,
    pub name: *const libc::c_char,
    pub namewhat: *const libc::c_char,
    pub what: *const libc::c_char,
    pub source: *const libc::c_char,
    pub currentline: i32,
    pub nups: i32,
    pub linedefined: i32,
    pub lastlinedefined: i32,
    pub short_src: [libc::c_char; 60],
    pub i_ci: i32,
}
pub type lua_Hook = Option<extern "C" fn(*mut lua_State, *mut lua_Debug) -> ()>;
pub type Lua = lua_State;
pub type LuaFn = Option<unsafe extern "C" fn(*mut Lua) -> i32>;
pub type LuaRef = lua_Integer;
pub type Signal = i32;
pub type SignalHandler = Option<extern "C" fn(Signal) -> ()>;

pub const kErrorHandler: *const libc::c_char =
    c_str!("function __error_handler__ (e)  return debug.traceback(e, 1)end");

static mut initialized: bool = false;

static mut activeInstance: *mut Lua = std::ptr::null_mut();

static mut cSignal: Signal = 0;

unsafe extern "C" fn Lua_BacktraceHook(this: *mut Lua, _: *mut lua_Debug) {
    let msg = format!("Received Signal: {}", signal_to_string(cSignal));

    lua_sethook(this, None, 0, 0);
    luaL_where(this, 0);
    lua_pushstring(this, msg.convert());
    lua_error(this);
}

extern "C" fn Lua_SignalHandler(_sig: Signal) {
    unsafe {
        if activeInstance.is_null() {
            return;
        }
        // if sig == Signal_Abrt || sig == Signal_Segv {
        /* NOTE : The implementation of abort() causes the program to forcefully
         *        exit as soon as the signal handler returns. Thus Lua_BacktraceHook
         *        will never get a chance to be called and we have to dump a trace
         *        now. */
        Lua_Backtrace();
        // } else {
        //     cSignal = sig;
        //     lua_sethook(
        //         activeInstance,
        //         Some(Lua_BacktraceHook as extern "C" fn(*mut Lua, *mut lua_Debug) -> ()),
        //         LUA_MASKCALL | LUA_MASKRET | LUA_MASKCOUNT,
        //         1,
        //     );
        //     Signal_IgnoreDefault();
        // };
    }
}

unsafe extern "C" fn Lua_PCall(this: *mut Lua, args: i32, rets: i32, errorHandler: i32) {
    let prev: *mut Lua = activeInstance;
    activeInstance = this;
    let result: i32 = lua_pcall(this, args, rets, errorHandler);
    if result != 0 {
        if result == 4 {
            CFatal!("Lua_PCall: Lua returned a memory allocation error");
        } else if result == 5 {
            CFatal!("Lua_PCall: Lua errored while attempting to run the error handler");
        } else if result == 2 {
            let error: *const libc::c_char = lua_tolstring(this, -1, std::ptr::null_mut());
            println!("{}", std::ffi::CStr::from_ptr(error).to_str().unwrap());
            CFatal!("Lua_PCall: Lua returned error message: %s", error);
        } else {
            CFatal!("Lua_PCall: Lua returned an invalid error code (corruption?)");
        }
    }
    activeInstance = prev;
}

unsafe extern "C" fn Lua_CallBarrier(this: *mut Lua) -> i32 {
    let args: i32 = lua_gettop(this) - 1;
    lua_call(this, args, -1);
    lua_gettop(this)
}

unsafe extern "C" fn Lua_InitExtensions(this: *mut Lua) {
    Lua_SetFn(
        this,
        c_str!("Call"),
        Some(Lua_CallBarrier as unsafe extern "C" fn(*mut Lua) -> i32),
    );
    LuaScheduler_Init(this);
    LuaScheduler_Register(this);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_Create() -> *mut Lua {
    if !initialized {
        initialized = true;
        Signal_AddHandlerAll(Lua_SignalHandler);
    }
    let this: *mut Lua = luaL_newstate();
    luaL_openlibs(this);
    Lua_InitExtensions(this);
    if luaL_loadstring(this, kErrorHandler) != 0 || lua_pcall(this, 0, -1, 0) != 0 {
        CFatal!("Lua_Create: failed to load error handler");
    }
    this
}

#[no_mangle]
pub unsafe extern "C" fn Lua_CreateThread(this: *mut Lua) -> *mut Lua {
    lua_newthread(this)
}

#[no_mangle]
pub unsafe extern "C" fn Lua_Free(this: *mut Lua) {
    lua_close(this);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_GetActive() -> *mut Lua {
    activeInstance
}

#[no_mangle]
pub unsafe extern "C" fn Lua_DoFile(this: *mut Lua, name: *const libc::c_char) {
    Lua_LoadFile(this, name);
    Lua_PCall(this, 0, 0, 0);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_DoString(this: *mut Lua, code: *const libc::c_char) {
    Lua_LoadString(this, code);
    Lua_PCall(this, 0, 0, 0);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_LoadFile(this: *mut Lua, name: *const libc::c_char) {
    let path: *const libc::c_char = Resource_GetPath(ResourceType_Script, name);
    if luaL_loadfile(this, path) != 0 {
        CFatal!(
            "Lua_LoadFile: failed to load <%s>:\n%s",
            path,
            lua_tolstring(this, -1, std::ptr::null_mut()),
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn Lua_LoadString(this: *mut Lua, code: *const libc::c_char) {
    if luaL_loadstring(this, code) != 0 {
        CFatal!(
            "Lua_LoadString: failed to load string:\n%s",
            lua_tolstring(this, -1, std::ptr::null_mut()),
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn Lua_Call(this: *mut Lua, args: i32, rets: i32, errorHandler: i32) {
    Lua_PCall(this, args, rets, errorHandler);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_PushGlobal(this: *mut Lua, name: *const libc::c_char) {
    lua_getfield(this, -10002, name);
    if lua_type(this, lua_gettop(this)) == 0 {
        CFatal!("Lua_PushGlobal: failed to find global key <%s>", name,);
    }
}

#[no_mangle]
pub unsafe extern "C" fn Lua_PushNumber(this: *mut Lua, value: f64) {
    lua_pushnumber(this, value);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_PushPtr(this: *mut Lua, value: *mut libc::c_void) {
    lua_pushlightuserdata(this, value);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_PushStr(this: *mut Lua, value: *const libc::c_char) {
    lua_pushstring(this, value);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_PushThread(this: *mut Lua, thread: *mut Lua) {
    lua_pushthread(thread);
    lua_xmove(thread, this, 1);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_SetBool(this: *mut Lua, name: *const libc::c_char, value: bool) {
    lua_pushboolean(this, value as i32);
    lua_setfield(this, -10002, name);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_SetFn(this: *mut Lua, name: *const libc::c_char, fn_0: LuaFn) {
    lua_pushcclosure(this, fn_0, 0);
    lua_setfield(this, -10002, name);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_SetNumber(this: *mut Lua, name: *const libc::c_char, value: f64) {
    lua_pushnumber(this, value);
    lua_setfield(this, -10002, name);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_SetPtr(
    this: *mut Lua,
    name: *const libc::c_char,
    value: *mut libc::c_void,
) {
    lua_pushlightuserdata(this, value);
    lua_setfield(this, -10002, name);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_SetStr(
    this: *mut Lua,
    name: *const libc::c_char,
    value: *const libc::c_char,
) {
    lua_pushstring(this, value);
    lua_setfield(this, -10002, name);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_TransferStack(src: *mut Lua, dst: *mut Lua, count: i32) {
    lua_xmove(src, dst, count);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_GetRef(this: *mut Lua) -> LuaRef {
    luaL_ref(this, -10000) as LuaRef
}

#[no_mangle]
pub unsafe extern "C" fn Lua_ReleaseRef(this: *mut Lua, ref_0: LuaRef) {
    luaL_unref(this, -10000, ref_0 as i32);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_PushRef(this: *mut Lua, ref_0: LuaRef) {
    lua_rawgeti(this, -10000, ref_0 as i32);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_GCFull(this: *mut Lua) {
    lua_gc(this, 2, 0);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_GCSetActive(this: *mut Lua, active: bool) {
    if active {
        lua_gc(this, 1, 0);
    } else {
        lua_gc(this, 0, 0);
    };
}

#[no_mangle]
pub unsafe extern "C" fn Lua_GCStep(this: *mut Lua) {
    lua_gc(this, 5, 0);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_GetMemory(this: *mut Lua) -> i32 {
    lua_gc(this, 3, 0) * 1024 + lua_gc(this, 4, 0)
}

#[inline]
unsafe extern "C" fn Lua_ToString(
    this: *mut Lua,
    name: *const libc::c_char,
) -> *const libc::c_char {
    lua_to_string(this, &name.convert()).convert()
}

unsafe fn lua_to_string(this: *mut Lua, name: &str) -> String {
    let type_0: i32 = lua_type(this, -1);
    let type_name = lua_typename(this, type_0).convert();
    let mut str_value = String::new();
    let mut is_null: bool = false;

    if luaL_callmeta(this, -1, c_str!("__tostring")) != 0 {
        str_value = lua_tolstring(this, -1, std::ptr::null_mut()).convert();
        lua_settop(this, -1 - 1);
    } else {
        let current_block_14: u64;
        match type_0 {
            0 => {
                current_block_14 = 12136430868992966025;
            }
            1 => {
                str_value = if lua_toboolean(this, -1) != 0 {
                    "True"
                } else {
                    "False"
                }
                .into();
                current_block_14 = 11584701595673473500;
            }
            3 => {
                str_value = lua_tolstring(this, -1, std::ptr::null_mut()).convert();
                current_block_14 = 11584701595673473500;
            }
            4 => {
                str_value = lua_tolstring(this, -1, std::ptr::null_mut()).convert();
                current_block_14 = 11584701595673473500;
            }
            2 => {
                str_value = format!("{:p}", lua_touserdata(this, -1));
                current_block_14 = 11584701595673473500;
            }
            5 => {
                str_value = format!("{:p}", lua_topointer(this, -1));
                current_block_14 = 11584701595673473500;
            }
            6 => {
                str_value = format!("{:p}", lua_topointer(this, -1));
                current_block_14 = 11584701595673473500;
            }
            7 => {
                str_value = format!("{:p}", lua_touserdata(this, -1));
                current_block_14 = 11584701595673473500;
            }
            8 => {
                str_value = format!("{:p}", lua_topointer(this, -1));
                current_block_14 = 11584701595673473500;
            }
            10 => {
                str_value = format!("{:p}", lua_topointer(this, -1));
                current_block_14 = 11584701595673473500;
            }
            _ => CFatal!("Lua_ToString: Unexpected type %i", type_0),
        }
        match current_block_14 {
            12136430868992966025 => {
                str_value = "nil".into();
                is_null = true;
            }
            _ => {}
        }
    }
    let pre = if is_null as i32 != 0 {
        "\x1B[91;1m"
    } else {
        ""
    };
    let app = if is_null as i32 != 0 { "\x1B[0m" } else { "" };

    format!("{pre}      {type_name:-10} {name:-16} = {str_value}{app}")
}

// pub fn Lua_GetCurrentLocation() {
//     let this: *mut Lua = activeInstance;
//     if this.is_null() {
//         return;
//     }

//     let result: i32 = lua_getstack(this, iStack, &mut ar);
//     if result == 0 {
//         return;
//     }
//     result = lua_getinfo(
//         this,
//         c_str!("nSluf"),
//         &mut ar,
//     );
//     if result == 0 {
//         CFatal!("Lua_GetStack: lua_getinfo failed.");
//     }
// }

#[no_mangle]
pub unsafe extern "C" fn Lua_Backtrace() {
    let this: *mut Lua = activeInstance;

    if this.is_null() {
        return;
    }

    let mut stack = Vec::new();
    let mut i_stack: i32 = 0;

    loop {
        let mut ar: lua_Debug = lua_Debug {
            event: 0,
            name: std::ptr::null(),
            namewhat: std::ptr::null(),
            what: std::ptr::null(),
            source: std::ptr::null(),
            currentline: 0,
            nups: 0,
            linedefined: 0,
            lastlinedefined: 0,
            short_src: [0; 60],
            i_ci: 0,
        };
        let mut result: i32 = lua_getstack(this, i_stack, &mut ar);

        if result == 0 {
            break;
        }

        result = lua_getinfo(this, c_str!("nSluf"), &mut ar);

        if result == 0 {
            CFatal!("Lua_GetStack: lua_getinfo failed.");
        }

        let mut variablesPrinted: i32 = 0;
        let mut func_name = ar.name.convert();
        let mut file_name = ar.source.convert();
        let mut line: i32 = ar.currentline;

        if !file_name.starts_with('@') {
            file_name = "<string>".into();
            line = -1;
        }
        if file_name.starts_with('@') {
            file_name.remove(0);
        }
        if ar.what.convert() == "C" {
            file_name = "<native>".into();
        }
        if ar.what.convert() == "main" {
            func_name = "<main>".into();
        }
        if func_name.is_empty() {
            func_name = "<null>".into();
        }
        let stack_frame = if line > 0 {
            format!("  #{} {} at {}:{}", i_stack, func_name, file_name, line)
        } else {
            format!("  #{} {} at {}", i_stack, func_name, file_name)
        };

        stack.push(stack_frame);

        let mut i_up: i32 = 1;
        loop {
            let name = lua_getupvalue(this, -1, i_up).convert();

            if name.is_empty() {
                break;
            }

            if i_up == 1 {
                stack.push("    [Upvalues]".into());
            }

            let up_value = lua_to_string(this, &name);

            stack.push(up_value);

            lua_settop(this, -1 - 1);

            variablesPrinted += 1;
            i_up += 1;
        }

        let mut i_local: i32 = 1;
        loop {
            let name_0 = lua_getlocal(this, &mut ar, i_local).convert();
            if name_0.is_empty() {
                break;
            }

            if i_local == 1 {
                stack.push("    [Locals]".into());
            }

            let local = lua_to_string(this, &name_0);
            stack.push(local);

            lua_settop(this, -1 - 1);
            variablesPrinted += 1;
            i_local += 1;
        }

        if variablesPrinted > 0 {
            stack.push("".into());
        }
        lua_settop(this, -1 - 1);
        i_stack += 1;
    }

    CWarn!("Lua Backtrace:");
    for stack_frame in stack {
        CWarn!("%s", &stack_frame);
    }
}
