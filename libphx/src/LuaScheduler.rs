use crate::internal::Memory::*;
use crate::Common::*;
use crate::Lua::*;
use crate::Math::Vec3;
use crate::TimeStamp::*;
use libc;

extern "C" {
    fn lua_gettop(L: *mut lua_State) -> i32;
    fn lua_settop(L: *mut lua_State, idx: i32);
    fn lua_tonumber(L: *mut lua_State, idx: i32) -> lua_Number;
    fn lua_getfield(L: *mut lua_State, idx: i32, k: *const libc::c_char);
    fn luaL_unref(L: *mut lua_State, t: i32, ref_0: i32);
    fn Lua_PushRef(_: *mut Lua, _: LuaRef);
    fn Lua_ReleaseRef(_: *mut Lua, _: LuaRef);
    fn Lua_GetRef(_: *mut Lua) -> LuaRef;
    fn Lua_Call(_: *mut Lua, args: i32, rets: i32, errorHandler: i32);
    fn Lua_PushNumber(_: *mut Lua, _: f64);
    fn Lua_SetFn(_: *mut Lua, name: *const libc::c_char, _: LuaFn);
    fn TimeStamp_Get() -> TimeStamp;
    fn TimeStamp_GetDifference(start: TimeStamp, end: TimeStamp) -> f64;
    fn TimeStamp_GetRelative(start: TimeStamp, seconds: f64) -> TimeStamp;
}

pub type lua_Number = f64;
pub type lua_Integer = libc::ptrdiff_t;
pub type Lua = lua_State;
pub type LuaFn = Option<unsafe extern "C" fn(*mut Lua) -> i32>;
pub type LuaRef = lua_Integer;

#[derive(Clone)]
#[repr(C)]
pub struct Scheduler {
    pub elems: Vec<SchedulerElem>,
    pub addQueue: Vec<SchedulerElem>,
    pub now: TimeStamp,
    pub locked: bool,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SchedulerElem {
    pub fn_0: LuaRef,
    pub arg: LuaRef,
    pub tCreated: TimeStamp,
    pub tWake: TimeStamp,
}

static mut this: Scheduler = Scheduler {
    elems: Vec::new(),
    addQueue: Vec::new(),
    now: 0,
    locked: false,
};

unsafe extern "C" fn SortByWake(mut pa: *const libc::c_void, mut pb: *const libc::c_void) -> i32 {
    let mut a: *const SchedulerElem = pa as *const SchedulerElem;
    let mut b: *const SchedulerElem = pb as *const SchedulerElem;
    if (*a).tWake < (*b).tWake {
        1
    } else if (*a).tWake == (*b).tWake {
        0
    } else {
        -1
    }
}

unsafe extern "C" fn LuaScheduler_Add(mut L: *mut Lua) -> i32 {
    let mut elem: SchedulerElem = SchedulerElem {
        fn_0: 0,
        arg: 0,
        tCreated: 0,
        tWake: 0,
    };

    let timeToWake: f64 = lua_tonumber(L, lua_gettop(L));
    elem.tCreated = this.now;
    elem.tWake = TimeStamp_GetRelative(this.now, timeToWake);
    lua_settop(L, -1 - 1);

    elem.arg = Lua_GetRef(L);
    elem.fn_0 = Lua_GetRef(L);

    if this.locked {
        this.addQueue.push(elem);
    } else {
        this.elems.push(elem);
    }

    0
}

unsafe extern "C" fn LuaScheduler_Clear(mut L: *mut Lua) -> i32 {
    for elem in this.elems.iter() {
        luaL_unref(L, -10000, (*elem).fn_0 as i32);
        luaL_unref(L, -10000, (*elem).arg as i32);
    }

    this.elems.clear();

    0
}

unsafe extern "C" fn LuaScheduler_Update(mut L: *mut Lua) -> i32 {
    this.locked = true;

    libc::qsort(
        this.elems.as_mut_ptr() as *mut _,
        this.elems.len(),
        std::mem::size_of::<SchedulerElem>(),
        Some(SortByWake as unsafe extern "C" fn(*const libc::c_void, *const libc::c_void) -> i32),
    );
    this.now = TimeStamp_Get();

    lua_getfield(
        L,
        -10002,
        b"__error_handler__\0" as *const u8 as *const libc::c_char,
    );
    let mut handler: i32 = lua_gettop(L);

    while !this.elems.is_empty() {
        let elem = this.elems.last().unwrap();
        if this.now < (*elem).tWake {
            break;
        }

        let mut dt: f64 = TimeStamp_GetDifference((*elem).tCreated, this.now);

        Lua_PushRef(L, (*elem).fn_0);
        Lua_PushNumber(L, dt);
        Lua_PushRef(L, (*elem).arg);
        Lua_Call(L, 2, 0, handler);

        Lua_ReleaseRef(L, (*elem).fn_0);
        Lua_ReleaseRef(L, (*elem).arg);
        this.elems.pop();
    }
    lua_settop(L, -1 - 1);

    this.locked = false;

    while !this.addQueue.is_empty() {
        this.elems.push(this.addQueue.pop().unwrap());
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn LuaScheduler_Init(mut _L: *mut Lua) {
    this.elems = Vec::new();
    this.addQueue = Vec::new();
    this.now = TimeStamp_Get();
    this.locked = false;
}

#[no_mangle]
pub unsafe extern "C" fn LuaScheduler_Register(mut L: *mut Lua) {
    Lua_SetFn(
        L,
        b"Schedule\0" as *const u8 as *const libc::c_char,
        Some(LuaScheduler_Add as unsafe extern "C" fn(*mut Lua) -> i32),
    );
    Lua_SetFn(
        L,
        b"SchedulerClear\0" as *const u8 as *const libc::c_char,
        Some(LuaScheduler_Clear as unsafe extern "C" fn(*mut Lua) -> i32),
    );
    Lua_SetFn(
        L,
        b"SchedulerUpdate\0" as *const u8 as *const libc::c_char,
        Some(LuaScheduler_Update as unsafe extern "C" fn(*mut Lua) -> i32),
    );
}
