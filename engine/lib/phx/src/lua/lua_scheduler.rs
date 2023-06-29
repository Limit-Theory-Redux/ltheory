use super::*;
use crate::common::*;
use crate::system::*;

extern "C" {
    fn lua_gettop(L: *mut lua_State) -> i32;
    fn lua_settop(L: *mut lua_State, idx: i32);
    fn lua_tonumber(L: *mut lua_State, idx: i32) -> LuaNumber;
    fn lua_getfield(L: *mut lua_State, idx: i32, k: *const libc::c_char);
    fn luaL_unref(L: *mut lua_State, t: i32, ref_0: i32);
    fn Lua_PushRef(_: *mut Lua, _: LuaRef);
    fn Lua_ReleaseRef(_: *mut Lua, _: LuaRef);
    fn Lua_GetRef(_: *mut Lua) -> LuaRef;
    fn Lua_Call(_: *mut Lua, args: i32, rets: i32, errorHandler: i32);
    fn Lua_PushNumber(_: *mut Lua, _: f64);
    fn Lua_SetFn(_: *mut Lua, name: *const libc::c_char, _: LuaFn);
}

pub type LuaNumber = f64;
pub type LuaInteger = libc::ptrdiff_t;
pub type Lua = lua_State;
pub type LuaFn = Option<unsafe extern "C" fn(*mut Lua) -> i32>;
pub type LuaRef = LuaInteger;

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
    now: TimeStamp::zero(),
    locked: false,
};

unsafe extern "C" fn LuaScheduler_Add(L: *mut Lua) -> i32 {
    let timeToWake: f64 = lua_tonumber(L, lua_gettop(L));
    let mut elem: SchedulerElem = SchedulerElem {
        fn_0: 0,
        arg: 0,
        tCreated: this.now,
        tWake: this.now.get_relative(timeToWake),
    };

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

unsafe extern "C" fn LuaScheduler_Clear(L: *mut Lua) -> i32 {
    for elem in this.elems.iter() {
        luaL_unref(L, -10000, (*elem).fn_0 as i32);
        luaL_unref(L, -10000, (*elem).arg as i32);
    }

    this.elems.clear();

    0
}

unsafe extern "C" fn LuaScheduler_Update(L: *mut Lua) -> i32 {
    this.locked = true;

    this.elems
        .sort_by(|a: &SchedulerElem, b: &SchedulerElem| a.tWake.cmp(&b.tWake));
    this.now = TimeStamp::now();

    lua_getfield(L, -10002, c_str!("__error_handler__"));
    let handler: i32 = lua_gettop(L);

    while !this.elems.is_empty() {
        let elem = this.elems.last().unwrap();
        if this.now < (*elem).tWake {
            break;
        }

        let dt: f64 = (*elem).tCreated.get_difference(&this.now);

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
pub unsafe extern "C" fn LuaScheduler_Init(_L: *mut Lua) {
    this.elems = Vec::new();
    this.addQueue = Vec::new();
    this.now = TimeStamp::now();
    this.locked = false;
}

#[no_mangle]
pub unsafe extern "C" fn LuaScheduler_Register(L: *mut Lua) {
    Lua_SetFn(
        L,
        c_str!("Schedule"),
        Some(LuaScheduler_Add as unsafe extern "C" fn(*mut Lua) -> i32),
    );
    Lua_SetFn(
        L,
        c_str!("SchedulerClear"),
        Some(LuaScheduler_Clear as unsafe extern "C" fn(*mut Lua) -> i32),
    );
    Lua_SetFn(
        L,
        c_str!("SchedulerUpdate"),
        Some(LuaScheduler_Update as unsafe extern "C" fn(*mut Lua) -> i32),
    );
}
