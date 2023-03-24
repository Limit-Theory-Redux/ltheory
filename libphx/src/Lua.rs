use crate::internal::Memory::*;
use crate::Common::*;
use crate::LuaScheduler::*;
use crate::Math::Vec3;
use crate::Resource::*;
use crate::ResourceType::*;
use crate::Signal::*;
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
pub type __builtin_va_list = *mut libc::c_char;
pub type ResourceType = i32;
pub type va_list = __builtin_va_list;
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
pub type lua_Hook = Option<unsafe extern "C" fn(*mut lua_State, *mut lua_Debug) -> ()>;
pub type Lua = lua_State;
pub type LuaFn = Option<unsafe extern "C" fn(*mut Lua) -> i32>;
pub type LuaRef = lua_Integer;
pub type Signal = i32;
pub type SignalHandler = Option<unsafe extern "C" fn(Signal) -> ()>;

pub const kErrorHandler: *const libc::c_char =
    b"function __error_handler__ (e)  return debug.traceback(e, 1)end\0" as *const u8
        as *const libc::c_char;

static mut initialized: bool = false;

static mut activeInstance: *mut Lua = std::ptr::null_mut();

static mut cSignal: Signal = 0;

unsafe extern "C" fn Lua_BacktraceHook(mut this: *mut Lua, _: *mut lua_Debug) {
    lua_sethook(this, None, 0, 0);
    luaL_where(this, 0);
    lua_pushstring(
        this,
        StrAdd(
            b"Received Signal: \0" as *const u8 as *const libc::c_char,
            Signal_ToString(cSignal),
        ),
    );
    lua_error(this);
}

unsafe extern "C" fn Lua_SignalHandler(mut sig: Signal) {
    if activeInstance.is_null() {
        return;
    }
    // if sig == Signal_Abrt || sig == Signal_Segv {
    Lua_Backtrace();
    // } else {
    //     cSignal = sig;
    //     lua_sethook(
    //         activeInstance,
    //         Some(Lua_BacktraceHook as unsafe extern "C" fn(*mut Lua, *mut lua_Debug) -> ()),
    //         1 << 0 | 1 << 1 | 1 << 3,
    //         1,
    //     );
    //     Signal_IgnoreDefault();
    // };
}

unsafe extern "C" fn Lua_PCall(
    mut this: *mut Lua,
    mut args: i32,
    mut rets: i32,
    mut errorHandler: i32,
) {
    let mut prev: *mut Lua = activeInstance;
    activeInstance = this;
    let mut result: i32 = lua_pcall(this, args, rets, errorHandler);
    if result != 0 {
        if result == 4 {
            Fatal(
                b"Lua_PCall: Lua returned a memory allocation error\0" as *const u8
                    as *const libc::c_char,
            );
        } else if result == 5 {
            Fatal(
                b"Lua_PCall: Lua errored while attempting to run the error handler\0" as *const u8
                    as *const libc::c_char,
            );
        } else if result == 2 {
            let mut error: *const libc::c_char = lua_tolstring(this, -1, std::ptr::null_mut());
            println!("{}", std::ffi::CStr::from_ptr(error).to_str().unwrap());
            Fatal(
                b"Lua_PCall: Lua returned error message: %s\0" as *const u8 as *const libc::c_char,
                error,
            );
        } else {
            Fatal(
                b"Lua_PCall: Lua returned an invalid error code (corruption?)\0" as *const u8
                    as *const libc::c_char,
            );
        }
    }
    activeInstance = prev;
}

unsafe extern "C" fn Lua_CallBarrier(mut this: *mut Lua) -> i32 {
    let mut args: i32 = lua_gettop(this) - 1;
    lua_call(this, args, -1);
    lua_gettop(this)
}

unsafe extern "C" fn Lua_InitExtensions(mut this: *mut Lua) {
    Lua_SetFn(
        this,
        b"Call\0" as *const u8 as *const libc::c_char,
        Some(Lua_CallBarrier as unsafe extern "C" fn(*mut Lua) -> i32),
    );
    LuaScheduler_Init(this);
    LuaScheduler_Register(this);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_Create() -> *mut Lua {
    if !initialized {
        initialized = true;
        Signal_AddHandlerAll(Some(
            Lua_SignalHandler as unsafe extern "C" fn(Signal) -> (),
        ));
    }
    let mut this: *mut Lua = luaL_newstate();
    luaL_openlibs(this);
    Lua_InitExtensions(this);
    if luaL_loadstring(this, kErrorHandler) != 0 || lua_pcall(this, 0, -1, 0) != 0 {
        Fatal(b"Lua_Create: failed to load error handler\0" as *const u8 as *const libc::c_char);
    }
    this
}

#[no_mangle]
pub unsafe extern "C" fn Lua_CreateThread(mut this: *mut Lua) -> *mut Lua {
    lua_newthread(this)
}

#[no_mangle]
pub unsafe extern "C" fn Lua_Free(mut this: *mut Lua) {
    lua_close(this);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_GetActive() -> *mut Lua {
    activeInstance
}

#[no_mangle]
pub unsafe extern "C" fn Lua_DoFile(mut this: *mut Lua, mut name: *const libc::c_char) {
    Lua_LoadFile(this, name);
    Lua_PCall(this, 0, 0, 0);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_DoString(mut this: *mut Lua, mut code: *const libc::c_char) {
    Lua_LoadString(this, code);
    Lua_PCall(this, 0, 0, 0);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_LoadFile(mut this: *mut Lua, mut name: *const libc::c_char) {
    let mut path: *const libc::c_char = Resource_GetPath(ResourceType_Script, name);
    if luaL_loadfile(this, path) != 0 {
        Fatal(
            b"Lua_LoadFile: failed to load <%s>:\n%s\0" as *const u8 as *const libc::c_char,
            path,
            lua_tolstring(this, -1, std::ptr::null_mut()),
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn Lua_LoadString(mut this: *mut Lua, mut code: *const libc::c_char) {
    if luaL_loadstring(this, code) != 0 {
        Fatal(
            b"Lua_LoadString: failed to load string:\n%s\0" as *const u8 as *const libc::c_char,
            lua_tolstring(this, -1, std::ptr::null_mut()),
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn Lua_Call(
    mut this: *mut Lua,
    mut args: i32,
    mut rets: i32,
    mut errorHandler: i32,
) {
    Lua_PCall(this, args, rets, errorHandler);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_PushGlobal(mut this: *mut Lua, mut name: *const libc::c_char) {
    lua_getfield(this, -10002, name);
    if lua_type(this, lua_gettop(this)) == 0 {
        Fatal(
            b"Lua_PushGlobal: failed to find global key <%s>\0" as *const u8 as *const libc::c_char,
            name,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn Lua_PushNumber(mut this: *mut Lua, mut value: f64) {
    lua_pushnumber(this, value);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_PushPtr(mut this: *mut Lua, mut value: *mut libc::c_void) {
    lua_pushlightuserdata(this, value);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_PushStr(mut this: *mut Lua, mut value: *const libc::c_char) {
    lua_pushstring(this, value);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_PushThread(mut this: *mut Lua, mut thread: *mut Lua) {
    lua_pushthread(thread);
    lua_xmove(thread, this, 1);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_SetBool(
    mut this: *mut Lua,
    mut name: *const libc::c_char,
    mut value: bool,
) {
    lua_pushboolean(this, value as i32);
    lua_setfield(this, -10002, name);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_SetFn(
    mut this: *mut Lua,
    mut name: *const libc::c_char,
    mut fn_0: LuaFn,
) {
    lua_pushcclosure(this, fn_0, 0);
    lua_setfield(this, -10002, name);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_SetNumber(
    mut this: *mut Lua,
    mut name: *const libc::c_char,
    mut value: f64,
) {
    lua_pushnumber(this, value);
    lua_setfield(this, -10002, name);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_SetPtr(
    mut this: *mut Lua,
    mut name: *const libc::c_char,
    mut value: *mut libc::c_void,
) {
    lua_pushlightuserdata(this, value);
    lua_setfield(this, -10002, name);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_SetStr(
    mut this: *mut Lua,
    mut name: *const libc::c_char,
    mut value: *const libc::c_char,
) {
    lua_pushstring(this, value);
    lua_setfield(this, -10002, name);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_TransferStack(mut src: *mut Lua, mut dst: *mut Lua, mut count: i32) {
    lua_xmove(src, dst, count);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_GetRef(mut this: *mut Lua) -> LuaRef {
    luaL_ref(this, -10000) as LuaRef
}

#[no_mangle]
pub unsafe extern "C" fn Lua_ReleaseRef(mut this: *mut Lua, mut ref_0: LuaRef) {
    luaL_unref(this, -10000, ref_0 as i32);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_PushRef(mut this: *mut Lua, mut ref_0: LuaRef) {
    lua_rawgeti(this, -10000, ref_0 as i32);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_GCFull(mut this: *mut Lua) {
    lua_gc(this, 2, 0);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_GCSetActive(mut this: *mut Lua, mut active: bool) {
    if active {
        lua_gc(this, 1, 0);
    } else {
        lua_gc(this, 0, 0);
    };
}

#[no_mangle]
pub unsafe extern "C" fn Lua_GCStep(mut this: *mut Lua) {
    lua_gc(this, 5, 0);
}

#[no_mangle]
pub unsafe extern "C" fn Lua_GetMemory(mut this: *mut Lua) -> i32 {
    lua_gc(this, 3, 0) * 1024 + lua_gc(this, 4, 0)
}

#[inline]
unsafe extern "C" fn Lua_ToString(
    mut this: *mut Lua,
    mut name: *const libc::c_char,
) -> *const libc::c_char {
    let mut type_0: i32 = lua_type(this, -1);
    let mut typeName: *const libc::c_char = lua_typename(this, type_0);
    let mut strValue: *const libc::c_char = std::ptr::null();
    let mut isNull: bool = false;
    if luaL_callmeta(
        this,
        -1,
        b"__tostring\0" as *const u8 as *const libc::c_char,
    ) != 0
    {
        strValue = StrDup(lua_tolstring(this, -1, std::ptr::null_mut()));
        lua_settop(this, -1 - 1);
    } else {
        let mut current_block_14: u64;
        match type_0 {
            0 => {
                current_block_14 = 12136430868992966025;
            }
            1 => {
                strValue = if lua_toboolean(this, -1) != 0 {
                    b"True\0" as *const u8 as *const libc::c_char
                } else {
                    b"False\0" as *const u8 as *const libc::c_char
                };
                current_block_14 = 11584701595673473500;
            }
            3 => {
                strValue = lua_tolstring(this, -1, std::ptr::null_mut());
                current_block_14 = 11584701595673473500;
            }
            4 => {
                strValue = lua_tolstring(this, -1, std::ptr::null_mut());
                current_block_14 = 11584701595673473500;
            }
            2 => {
                strValue = StrFormat(
                    b"0x%p\0" as *const u8 as *const libc::c_char,
                    lua_touserdata(this, -1),
                );
                current_block_14 = 11584701595673473500;
            }
            5 => {
                strValue = StrFormat(
                    b"0x%p\0" as *const u8 as *const libc::c_char,
                    lua_topointer(this, -1),
                );
                current_block_14 = 11584701595673473500;
            }
            6 => {
                strValue = StrFormat(
                    b"0x%p\0" as *const u8 as *const libc::c_char,
                    lua_topointer(this, -1),
                );
                current_block_14 = 11584701595673473500;
            }
            7 => {
                strValue = StrFormat(
                    b"0x%p\0" as *const u8 as *const libc::c_char,
                    lua_touserdata(this, -1),
                );
                current_block_14 = 11584701595673473500;
            }
            8 => {
                strValue = StrFormat(
                    b"0x%p\0" as *const u8 as *const libc::c_char,
                    lua_topointer(this, -1),
                );
                current_block_14 = 11584701595673473500;
            }
            10 => {
                strValue = StrFormat(
                    b"0x%p\0" as *const u8 as *const libc::c_char,
                    lua_topointer(this, -1),
                );
                current_block_14 = 11584701595673473500;
            }
            _ => Fatal(
                b"Lua_ToString: Unexpected type %i\0" as *const u8 as *const libc::c_char,
                type_0,
            ),
        }
        match current_block_14 {
            12136430868992966025 => {
                strValue = b"nil\0" as *const u8 as *const libc::c_char;
                isNull = true;
            }
            _ => {}
        }
    }
    let mut pre: *const libc::c_char = if isNull as i32 != 0 {
        b"\x1B[91;1m\0" as *const u8 as *const libc::c_char
    } else {
        b"\0" as *const u8 as *const libc::c_char
    };
    let mut app: *const libc::c_char = if isNull as i32 != 0 {
        b"\x1B[0m\0" as *const u8 as *const libc::c_char
    } else {
        b"\0" as *const u8 as *const libc::c_char
    };
    StrFormat(
        b"%s      %-10s %-16s = %s%s\0" as *const u8 as *const libc::c_char,
        pre,
        typeName,
        name,
        strValue,
        app,
    )
}

// pub unsafe fn Lua_GetCurrentLocation() {
//     let mut this: *mut Lua = activeInstance;
//     if this.is_null() {
//         return;
//     }

//     let mut result: i32 = lua_getstack(this, iStack, &mut ar);
//     if result == 0 {
//         return;
//     }
//     result = lua_getinfo(
//         this,
//         b"nSluf\0" as *const u8 as *const libc::c_char,
//         &mut ar,
//     );
//     if result == 0 {
//         Fatal(b"Lua_GetStack: lua_getinfo failed.\0" as *const u8 as *const libc::c_char);
//     }
// }

#[no_mangle]
pub unsafe extern "C" fn Lua_Backtrace() {
    let mut this: *mut Lua = activeInstance;
    if this.is_null() {
        return;
    }
    let mut stack: Vec<*const libc::c_char> = Vec::new();
    let mut iStack: i32 = 0;
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
        let mut result: i32 = lua_getstack(this, iStack, &mut ar);
        if result == 0 {
            break;
        }
        result = lua_getinfo(
            this,
            b"nSluf\0" as *const u8 as *const libc::c_char,
            &mut ar,
        );
        if result == 0 {
            Fatal(b"Lua_GetStack: lua_getinfo failed.\0" as *const u8 as *const libc::c_char);
        }
        let mut variablesPrinted: i32 = 0;
        let mut funcName: *const libc::c_char = ar.name;
        let mut fileName: *const libc::c_char = ar.source;
        let mut line: i32 = ar.currentline;
        if *fileName.offset(0) as i32 != '@' as i32 {
            fileName = b"<string>\0" as *const u8 as *const libc::c_char;
            line = -1;
        }
        if *fileName.offset(0) as i32 == '@' as i32 {
            fileName = fileName.offset(1);
        }
        if StrEqual(ar.what, b"C\0" as *const u8 as *const libc::c_char) {
            fileName = b"<native>\0" as *const u8 as *const libc::c_char;
        }
        if StrEqual(ar.what, b"main\0" as *const u8 as *const libc::c_char) {
            funcName = b"<main>\0" as *const u8 as *const libc::c_char;
        }
        if funcName.is_null() {
            funcName = b"<null>\0" as *const u8 as *const libc::c_char;
        }
        let stackFrame: *const libc::c_char = if line > 0 {
            StrFormat(
                b"  #%i %s at %s:%i\0" as *const u8 as *const libc::c_char,
                iStack,
                funcName,
                fileName,
                line,
            )
        } else {
            StrFormat(
                b"  #%i %s at %s\0" as *const u8 as *const libc::c_char,
                iStack,
                funcName,
                fileName,
            )
        };
        stack.push(stackFrame);

        let mut iUp: i32 = 1;
        loop {
            let name: *const libc::c_char = lua_getupvalue(this, -1, iUp);
            if name.is_null() {
                break;
            }

            if iUp == 1 {
                stack.push(StrDup(
                    b"    [Upvalues]\0" as *const u8 as *const libc::c_char,
                ));
            }

            let upValue: *const libc::c_char = Lua_ToString(this, name);
            stack.push(upValue);
            lua_settop(this, -1 - 1);
            variablesPrinted += 1;
            iUp += 1;
        }

        let mut iLocal: i32 = 1;
        loop {
            let mut name_0: *const libc::c_char = lua_getlocal(this, &mut ar, iLocal);
            if name_0.is_null() {
                break;
            }

            if iLocal == 1 {
                stack.push(StrDup(
                    b"    [Locals]\0" as *const u8 as *const libc::c_char,
                ));
            }

            let local: *const libc::c_char = Lua_ToString(this, name_0);
            stack.push(local);

            lua_settop(this, -1 - 1);
            variablesPrinted += 1;
            iLocal += 1;
        }

        if variablesPrinted > 0 {
            stack.push(StrDup(b"\0" as *const u8 as *const libc::c_char));
        }
        lua_settop(this, -1 - 1);
        iStack += 1;
    }

    Warn(b"Lua Backtrace:\0" as *const u8 as *const libc::c_char);
    for stackFrame in stack.iter() {
        Warn(*stackFrame);
        StrFree(*stackFrame);
    }
}
