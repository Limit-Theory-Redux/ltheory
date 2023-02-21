use crate::internal::Memory::*;
use crate::PhxSignal::*;
use crate::ResourceType::*;
use glam::Vec3;
use libc;

extern "C" {
    pub type lua_State;
    fn strcmp(_: *const libc::c_char, _: *const libc::c_char) -> i32;
    fn Fatal(_: cstr, _: ...);
    fn Warn(_: cstr, _: ...);
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
    fn LuaScheduler_Register(_: *mut Lua);
    fn LuaScheduler_Init(_: *mut Lua);
    fn luaL_newstate() -> *mut lua_State;
    fn Signal_AddHandlerAll(_: SignalHandler);
    fn Signal_ToString(_: Signal) -> cstr;
    fn Signal_IgnoreDefault();
    fn Resource_GetPath(_: ResourceType, name: cstr) -> cstr;
}
pub type __builtin_va_list = *mut libc::c_char;
pub type __darwin_ptrdiff_t = libc::c_long;
pub type cstr = *const libc::c_char;
pub type ResourceType = i32;
pub type va_list = __builtin_va_list;
pub type ptrdiff_t = __darwin_ptrdiff_t;
pub type lua_CFunction = Option<unsafe extern "C" fn(*mut lua_State) -> i32>;
pub type lua_Number = f64;
pub type lua_Integer = ptrdiff_t;

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

#[no_mangle]
pub static mut kErrorHandler: cstr =
    b"function __error_handler__ (e)  return debug.traceback(e, 1)end\0" as *const u8
        as *const libc::c_char;
static mut initialized: bool = 0 as i32 != 0;
static mut activeInstance: *mut Lua = 0 as *const Lua as *mut Lua;
static mut cSignal: Signal = 0 as i32;
unsafe extern "C" fn Lua_BacktraceHook(mut this: *mut Lua, _: *mut lua_Debug) {
    lua_sethook(this, None, 0 as i32, 0 as i32);
    luaL_where(this, 0 as i32);
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
    if sig == Signal_Abrt || sig == Signal_Segv {
        Lua_Backtrace();
    } else {
        cSignal = sig;
        lua_sethook(
            activeInstance,
            Some(Lua_BacktraceHook as unsafe extern "C" fn(*mut Lua, *mut lua_Debug) -> ()),
            (1 as i32) << 0 as i32 | (1 as i32) << 1 as i32 | (1 as i32) << 3 as i32,
            1 as i32,
        );
        Signal_IgnoreDefault();
    };
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
    if result != 0 as i32 {
        if result == 4 as i32 {
            Fatal(
                b"Lua_PCall: Lua returned a memory allocation error\0" as *const u8
                    as *const libc::c_char,
            );
        } else if result == 5 as i32 {
            Fatal(
                b"Lua_PCall: Lua errored while attempting to run the error handler\0" as *const u8
                    as *const libc::c_char,
            );
        } else if result == 2 as i32 {
            let mut error: cstr = lua_tolstring(this, -(1 as i32), 0 as *mut usize);
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
    let mut args: i32 = lua_gettop(this) - 1 as i32;
    lua_call(this, args, -(1 as i32));
    return lua_gettop(this);
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
        initialized = 1 as i32 != 0;
        Signal_AddHandlerAll(Some(
            Lua_SignalHandler as unsafe extern "C" fn(Signal) -> (),
        ));
    }
    let mut this: *mut Lua = luaL_newstate();
    luaL_openlibs(this);
    Lua_InitExtensions(this);
    if luaL_loadstring(this, kErrorHandler) != 0
        || lua_pcall(this, 0 as i32, -(1 as i32), 0 as i32) != 0
    {
        Fatal(b"Lua_Create: failed to load error handler\0" as *const u8 as *const libc::c_char);
    }
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Lua_CreateThread(mut this: *mut Lua) -> *mut Lua {
    return lua_newthread(this);
}
#[no_mangle]
pub unsafe extern "C" fn Lua_Free(mut this: *mut Lua) {
    lua_close(this);
}
#[no_mangle]
pub unsafe extern "C" fn Lua_GetActive() -> *mut Lua {
    return activeInstance;
}
#[no_mangle]
pub unsafe extern "C" fn Lua_DoFile(mut this: *mut Lua, mut name: cstr) {
    Lua_LoadFile(this, name);
    Lua_PCall(this, 0 as i32, 0 as i32, 0 as i32);
}
#[no_mangle]
pub unsafe extern "C" fn Lua_DoString(mut this: *mut Lua, mut code: cstr) {
    Lua_LoadString(this, code);
    Lua_PCall(this, 0 as i32, 0 as i32, 0 as i32);
}
#[no_mangle]
pub unsafe extern "C" fn Lua_LoadFile(mut this: *mut Lua, mut name: cstr) {
    let mut path: cstr = Resource_GetPath(ResourceType_Script, name);
    if luaL_loadfile(this, path) != 0 {
        Fatal(
            b"Lua_LoadFile: failed to load <%s>:\n%s\0" as *const u8 as *const libc::c_char,
            path,
            lua_tolstring(this, -(1 as i32), 0 as *mut usize),
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn Lua_LoadString(mut this: *mut Lua, mut code: cstr) {
    if luaL_loadstring(this, code) != 0 {
        Fatal(
            b"Lua_LoadString: failed to load string:\n%s\0" as *const u8 as *const libc::c_char,
            lua_tolstring(this, -(1 as i32), 0 as *mut usize),
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
pub unsafe extern "C" fn Lua_PushGlobal(mut this: *mut Lua, mut name: cstr) {
    lua_getfield(this, -(10002 as i32), name);
    if lua_type(this, lua_gettop(this)) == 0 as i32 {
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
pub unsafe extern "C" fn Lua_PushStr(mut this: *mut Lua, mut value: cstr) {
    lua_pushstring(this, value);
}
#[no_mangle]
pub unsafe extern "C" fn Lua_PushThread(mut this: *mut Lua, mut thread: *mut Lua) {
    lua_pushthread(thread);
    lua_xmove(thread, this, 1 as i32);
}
#[no_mangle]
pub unsafe extern "C" fn Lua_SetBool(mut this: *mut Lua, mut name: cstr, mut value: bool) {
    lua_pushboolean(this, value as i32);
    lua_setfield(this, -(10002 as i32), name);
}
#[no_mangle]
pub unsafe extern "C" fn Lua_SetFn(mut this: *mut Lua, mut name: cstr, mut fn_0: LuaFn) {
    lua_pushcclosure(this, fn_0, 0 as i32);
    lua_setfield(this, -(10002 as i32), name);
}
#[no_mangle]
pub unsafe extern "C" fn Lua_SetNumber(mut this: *mut Lua, mut name: cstr, mut value: f64) {
    lua_pushnumber(this, value);
    lua_setfield(this, -(10002 as i32), name);
}
#[no_mangle]
pub unsafe extern "C" fn Lua_SetPtr(
    mut this: *mut Lua,
    mut name: cstr,
    mut value: *mut libc::c_void,
) {
    lua_pushlightuserdata(this, value);
    lua_setfield(this, -(10002 as i32), name);
}
#[no_mangle]
pub unsafe extern "C" fn Lua_SetStr(mut this: *mut Lua, mut name: cstr, mut value: cstr) {
    lua_pushstring(this, value);
    lua_setfield(this, -(10002 as i32), name);
}
#[no_mangle]
pub unsafe extern "C" fn Lua_TransferStack(mut src: *mut Lua, mut dst: *mut Lua, mut count: i32) {
    lua_xmove(src, dst, count);
}
#[no_mangle]
pub unsafe extern "C" fn Lua_GetRef(mut this: *mut Lua) -> LuaRef {
    return luaL_ref(this, -(10000 as i32)) as LuaRef;
}
#[no_mangle]
pub unsafe extern "C" fn Lua_ReleaseRef(mut this: *mut Lua, mut ref_0: LuaRef) {
    luaL_unref(this, -(10000 as i32), ref_0 as i32);
}
#[no_mangle]
pub unsafe extern "C" fn Lua_PushRef(mut this: *mut Lua, mut ref_0: LuaRef) {
    lua_rawgeti(this, -(10000 as i32), ref_0 as i32);
}
#[no_mangle]
pub unsafe extern "C" fn Lua_GCFull(mut this: *mut Lua) {
    lua_gc(this, 2 as i32, 0 as i32);
}
#[no_mangle]
pub unsafe extern "C" fn Lua_GCSetActive(mut this: *mut Lua, mut active: bool) {
    if active {
        lua_gc(this, 1 as i32, 0 as i32);
    } else {
        lua_gc(this, 0 as i32, 0 as i32);
    };
}
#[no_mangle]
pub unsafe extern "C" fn Lua_GCStep(mut this: *mut Lua) {
    lua_gc(this, 5 as i32, 0 as i32);
}
#[no_mangle]
pub unsafe extern "C" fn Lua_GetMemory(mut this: *mut Lua) -> i32 {
    return lua_gc(this, 3 as i32, 0 as i32) * 1024 as i32 + lua_gc(this, 4 as i32, 0 as i32);
}
#[inline]
unsafe extern "C" fn Lua_ToString(mut this: *mut Lua, mut name: cstr) -> cstr {
    let mut type_0: i32 = lua_type(this, -(1 as i32));
    let mut typeName: cstr = lua_typename(this, type_0);
    let mut strValue: cstr = 0 as cstr;
    let mut isNull: bool = 0 as i32 != 0;
    if luaL_callmeta(
        this,
        -(1 as i32),
        b"__tostring\0" as *const u8 as *const libc::c_char,
    ) != 0
    {
        strValue = StrDup(lua_tolstring(this, -(1 as i32), 0 as *mut usize));
        lua_settop(this, -(1 as i32) - 1 as i32);
    } else {
        let mut current_block_14: u64;
        match type_0 {
            0 => {
                current_block_14 = 12136430868992966025;
            }
            1 => {
                strValue = if lua_toboolean(this, -(1 as i32)) != 0 {
                    b"True\0" as *const u8 as *const libc::c_char
                } else {
                    b"False\0" as *const u8 as *const libc::c_char
                };
                current_block_14 = 11584701595673473500;
            }
            3 => {
                strValue = lua_tolstring(this, -(1 as i32), 0 as *mut usize);
                current_block_14 = 11584701595673473500;
            }
            4 => {
                strValue = lua_tolstring(this, -(1 as i32), 0 as *mut usize);
                current_block_14 = 11584701595673473500;
            }
            2 => {
                strValue = StrFormat(
                    b"0x%p\0" as *const u8 as *const libc::c_char,
                    lua_touserdata(this, -(1 as i32)),
                );
                current_block_14 = 11584701595673473500;
            }
            5 => {
                strValue = StrFormat(
                    b"0x%p\0" as *const u8 as *const libc::c_char,
                    lua_topointer(this, -(1 as i32)),
                );
                current_block_14 = 11584701595673473500;
            }
            6 => {
                strValue = StrFormat(
                    b"0x%p\0" as *const u8 as *const libc::c_char,
                    lua_topointer(this, -(1 as i32)),
                );
                current_block_14 = 11584701595673473500;
            }
            7 => {
                strValue = StrFormat(
                    b"0x%p\0" as *const u8 as *const libc::c_char,
                    lua_touserdata(this, -(1 as i32)),
                );
                current_block_14 = 11584701595673473500;
            }
            8 => {
                strValue = StrFormat(
                    b"0x%p\0" as *const u8 as *const libc::c_char,
                    lua_topointer(this, -(1 as i32)),
                );
                current_block_14 = 11584701595673473500;
            }
            10 => {
                strValue = StrFormat(
                    b"0x%p\0" as *const u8 as *const libc::c_char,
                    lua_topointer(this, -(1 as i32)),
                );
                current_block_14 = 11584701595673473500;
            }
            _ => {
                Fatal(
                    b"Lua_ToString: Unexpected type %i\0" as *const u8 as *const libc::c_char,
                    type_0,
                );
                current_block_14 = 12136430868992966025;
            }
        }
        match current_block_14 {
            12136430868992966025 => {
                strValue = b"nil\0" as *const u8 as *const libc::c_char;
                isNull = 1 as i32 != 0;
            }
            _ => {}
        }
    }
    let mut pre: cstr = if isNull as i32 != 0 {
        b"\x1B[91;1m\0" as *const u8 as *const libc::c_char
    } else {
        b"\0" as *const u8 as *const libc::c_char
    };
    let mut app: cstr = if isNull as i32 != 0 {
        b"\x1B[0m\0" as *const u8 as *const libc::c_char
    } else {
        b"\0" as *const u8 as *const libc::c_char
    };
    return StrFormat(
        b"%s      %-10s %-16s = %s%s\0" as *const u8 as *const libc::c_char,
        pre,
        typeName,
        name,
        strValue,
        app,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Lua_Backtrace() {
    let mut this: *mut Lua = activeInstance;
    if this.is_null() {
        return;
    }
    let mut stack_size: i32 = 0;
    let mut stack_capacity: i32 = 0;
    let mut stack_data: *mut cstr = 0 as *mut cstr;
    stack_capacity = 0 as i32;
    stack_size = 0 as i32;
    stack_data = 0 as *mut cstr;
    let mut iStack: i32 = 0 as i32;
    loop {
        let mut ar: lua_Debug = lua_Debug {
            event: 0,
            name: 0 as *const libc::c_char,
            namewhat: 0 as *const libc::c_char,
            what: 0 as *const libc::c_char,
            source: 0 as *const libc::c_char,
            currentline: 0,
            nups: 0,
            linedefined: 0,
            lastlinedefined: 0,
            short_src: [0; 60],
            i_ci: 0,
        };
        let mut result: i32 = lua_getstack(this, iStack, &mut ar);
        if result == 0 as i32 {
            break;
        }
        result = lua_getinfo(
            this,
            b"nSluf\0" as *const u8 as *const libc::c_char,
            &mut ar,
        );
        if result == 0 as i32 {
            Fatal(b"Lua_GetStack: lua_getinfo failed.\0" as *const u8 as *const libc::c_char);
        }
        let mut variablesPrinted: i32 = 0 as i32;
        let mut funcName: cstr = ar.name;
        let mut fileName: cstr = ar.source;
        let mut line: i32 = ar.currentline;
        if *fileName.offset(0) as i32 != '@' as i32 {
            fileName = b"<string>\0" as *const u8 as *const libc::c_char;
            line = -(1 as i32);
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
        let mut stackFrame: cstr = if line > 0 as i32 {
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
        if (stack_capacity == stack_size) as libc::c_long != 0 {
            stack_capacity = if stack_capacity != 0 {
                stack_capacity * 2 as i32
            } else {
                1 as i32
            };
            let mut elemSize: usize = ::core::mem::size_of::<cstr>() as usize;
            let mut pData: *mut *mut libc::c_void =
                &mut stack_data as *mut *mut cstr as *mut *mut libc::c_void;
            *pData = MemRealloc(
                stack_data as *mut libc::c_void,
                (stack_capacity as usize).wrapping_mul(elemSize as usize),
            );
        }
        let fresh4 = stack_size;
        stack_size = stack_size + 1;
        let ref mut fresh5 = *stack_data.offset(fresh4 as isize);
        *fresh5 = stackFrame;
        let mut iUp: i32 = 1 as i32;
        loop {
            let mut name: cstr = lua_getupvalue(this, -(1 as i32), iUp);
            if name.is_null() {
                break;
            }
            if iUp == 1 as i32 {
                if (stack_capacity == stack_size) as libc::c_long != 0 {
                    stack_capacity = if stack_capacity != 0 {
                        stack_capacity * 2 as i32
                    } else {
                        1 as i32
                    };
                    let mut elemSize_0: usize = ::core::mem::size_of::<cstr>();
                    let mut pData_0: *mut *mut libc::c_void =
                        &mut stack_data as *mut *mut cstr as *mut *mut libc::c_void;
                    *pData_0 = MemRealloc(
                        stack_data as *mut libc::c_void,
                        (stack_capacity as usize).wrapping_mul(elemSize_0 as usize),
                    );
                }
                let fresh6 = stack_size;
                stack_size = stack_size + 1;
                let ref mut fresh7 = *stack_data.offset(fresh6 as isize);
                *fresh7 = StrDup(b"    [Upvalues]\0" as *const u8 as *const libc::c_char);
            }
            let mut upValue: cstr = Lua_ToString(this, name);
            if (stack_capacity == stack_size) as libc::c_long != 0 {
                stack_capacity = if stack_capacity != 0 {
                    stack_capacity * 2 as i32
                } else {
                    1 as i32
                };
                let mut elemSize_1: usize = ::core::mem::size_of::<cstr>();
                let mut pData_1: *mut *mut libc::c_void =
                    &mut stack_data as *mut *mut cstr as *mut *mut libc::c_void;
                *pData_1 = MemRealloc(
                    stack_data as *mut libc::c_void,
                    (stack_capacity as usize).wrapping_mul(elemSize_1 as usize),
                );
            }
            let fresh8 = stack_size;
            stack_size = stack_size + 1;
            let ref mut fresh9 = *stack_data.offset(fresh8 as isize);
            *fresh9 = upValue;
            lua_settop(this, -(1 as i32) - 1 as i32);
            variablesPrinted += 1;
            iUp += 1;
        }
        let mut iLocal: i32 = 1 as i32;
        loop {
            let mut name_0: cstr = lua_getlocal(this, &mut ar, iLocal);
            if name_0.is_null() {
                break;
            }
            if iLocal == 1 as i32 {
                if (stack_capacity == stack_size) as libc::c_long != 0 {
                    stack_capacity = if stack_capacity != 0 {
                        stack_capacity * 2 as i32
                    } else {
                        1 as i32
                    };
                    let mut elemSize_2: usize = ::core::mem::size_of::<cstr>();
                    let mut pData_2: *mut *mut libc::c_void =
                        &mut stack_data as *mut *mut cstr as *mut *mut libc::c_void;
                    *pData_2 = MemRealloc(
                        stack_data as *mut libc::c_void,
                        (stack_capacity as usize).wrapping_mul(elemSize_2 as usize),
                    );
                }
                let fresh10 = stack_size;
                stack_size = stack_size + 1;
                let ref mut fresh11 = *stack_data.offset(fresh10 as isize);
                *fresh11 = StrDup(b"    [Locals]\0" as *const u8 as *const libc::c_char);
            }
            let mut local: cstr = Lua_ToString(this, name_0);
            if (stack_capacity == stack_size) as libc::c_long != 0 {
                stack_capacity = if stack_capacity != 0 {
                    stack_capacity * 2 as i32
                } else {
                    1 as i32
                };
                let mut elemSize_3: usize = ::core::mem::size_of::<cstr>();
                let mut pData_3: *mut *mut libc::c_void =
                    &mut stack_data as *mut *mut cstr as *mut *mut libc::c_void;
                *pData_3 = MemRealloc(
                    stack_data as *mut libc::c_void,
                    (stack_capacity as usize).wrapping_mul(elemSize_3 as usize),
                );
            }
            let fresh12 = stack_size;
            stack_size = stack_size + 1;
            let ref mut fresh13 = *stack_data.offset(fresh12 as isize);
            *fresh13 = local;
            lua_settop(this, -(1 as i32) - 1 as i32);
            variablesPrinted += 1;
            iLocal += 1;
        }
        if variablesPrinted > 0 as i32 {
            if (stack_capacity == stack_size) as libc::c_long != 0 {
                stack_capacity = if stack_capacity != 0 {
                    stack_capacity * 2 as i32
                } else {
                    1 as i32
                };
                let mut elemSize_4: usize = ::core::mem::size_of::<cstr>();
                let mut pData_4: *mut *mut libc::c_void =
                    &mut stack_data as *mut *mut cstr as *mut *mut libc::c_void;
                *pData_4 = MemRealloc(
                    stack_data as *mut libc::c_void,
                    (stack_capacity as usize).wrapping_mul(elemSize_4 as usize),
                );
            }
            let fresh14 = stack_size;
            stack_size = stack_size + 1;
            let ref mut fresh15 = *stack_data.offset(fresh14 as isize);
            *fresh15 = StrDup(b"\0" as *const u8 as *const libc::c_char);
        }
        lua_settop(this, -(1 as i32) - 1 as i32);
        iStack += 1;
    }
    Warn(b"Lua Backtrace:\0" as *const u8 as *const libc::c_char);
    let mut stackFrame_0: *mut cstr = stack_data;
    let mut __iterend: *mut cstr = stack_data.offset(stack_size as isize);
    while stackFrame_0 < __iterend {
        Warn(*stackFrame_0);
        StrFree(*stackFrame_0);
        stackFrame_0 = stackFrame_0.offset(1);
    }
    MemFree(stack_data as *const libc::c_void);
}
