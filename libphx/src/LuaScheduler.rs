use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
extern "C" {
    pub type lua_State;
    fn qsort(
        __base: *mut libc::c_void,
        __nel: libc::size_t,
        __width: libc::size_t,
        __compar: Option::<
            unsafe extern "C" fn(*const libc::c_void, *const libc::c_void) -> libc::c_int,
        >,
    );
    fn lua_gettop(L: *mut lua_State) -> libc::c_int;
    fn lua_settop(L: *mut lua_State, idx: libc::c_int);
    fn lua_tonumber(L: *mut lua_State, idx: libc::c_int) -> lua_Number;
    fn lua_getfield(L: *mut lua_State, idx: libc::c_int, k: *const libc::c_char);
    fn luaL_unref(L: *mut lua_State, t: libc::c_int, ref_0: libc::c_int);
    fn Lua_PushRef(_: *mut Lua, _: LuaRef);
    fn Lua_ReleaseRef(_: *mut Lua, _: LuaRef);
    fn Lua_GetRef(_: *mut Lua) -> LuaRef;
    fn Lua_Call(
        _: *mut Lua,
        args: libc::c_int,
        rets: libc::c_int,
        errorHandler: libc::c_int,
    );
    fn Lua_PushNumber(_: *mut Lua, _: libc::c_double);
    fn Lua_SetFn(_: *mut Lua, name: cstr, _: LuaFn);
    fn TimeStamp_Get() -> TimeStamp;
    fn TimeStamp_GetDifference(start: TimeStamp, end: TimeStamp) -> libc::c_double;
    fn TimeStamp_GetRelative(start: TimeStamp, seconds: libc::c_double) -> TimeStamp;
}
pub type int32_t = libc::c_int;
pub type uint64_t = libc::c_ulonglong;
pub type __darwin_ptrdiff_t = libc::c_long;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type uint64 = uint64_t;
pub type TimeStamp = uint64;
pub type ptrdiff_t = __darwin_ptrdiff_t;
pub type lua_Number = libc::c_double;
pub type lua_Integer = ptrdiff_t;
pub type Lua = lua_State;
pub type LuaFn = Option::<unsafe extern "C" fn(*mut Lua) -> libc::c_int>;
pub type LuaRef = lua_Integer;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Scheduler {
    pub elems_size: int32,
    pub elems_capacity: int32,
    pub elems_data: *mut SchedulerElem,
    pub addQueue_size: int32,
    pub addQueue_capacity: int32,
    pub addQueue_data: *mut SchedulerElem,
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

static mut self_0: Scheduler = Scheduler {
    elems_size: 0,
    elems_capacity: 0,
    elems_data: 0 as *const SchedulerElem as *mut SchedulerElem,
    addQueue_size: 0,
    addQueue_capacity: 0,
    addQueue_data: 0 as *const SchedulerElem as *mut SchedulerElem,
    now: 0,
    locked: false,
};
unsafe extern "C" fn SortByWake(
    mut pa: *const libc::c_void,
    mut pb: *const libc::c_void,
) -> libc::c_int {
    let mut a: *const SchedulerElem = pa as *const SchedulerElem;
    let mut b: *const SchedulerElem = pb as *const SchedulerElem;
    return if (*a).tWake < (*b).tWake {
        1 as libc::c_int
    } else if (*a).tWake == (*b).tWake {
        0 as libc::c_int
    } else {
        -(1 as libc::c_int)
    };
}
unsafe extern "C" fn LuaScheduler_Add(mut L: *mut Lua) -> libc::c_int {
    let mut elem: SchedulerElem = SchedulerElem {
        fn_0: 0,
        arg: 0,
        tCreated: 0,
        tWake: 0,
    };
    let mut timeToWake: libc::c_double = lua_tonumber(L, lua_gettop(L));
    elem.tCreated = self_0.now;
    elem.tWake = TimeStamp_GetRelative(self_0.now, timeToWake);
    lua_settop(L, -(1 as libc::c_int) - 1 as libc::c_int);
    elem.arg = Lua_GetRef(L);
    elem.fn_0 = Lua_GetRef(L);
    if self_0.locked {
        if (self_0.addQueue_capacity == self_0.addQueue_size) as libc::c_int
            as libc::c_long != 0
        {
            self_0
                .addQueue_capacity = if self_0.addQueue_capacity != 0 {
                self_0.addQueue_capacity * 2 as libc::c_int
            } else {
                1 as libc::c_int
            };
            let mut elemSize: usize = ::core::mem::size_of::<SchedulerElem>();
            let mut pData: *mut *mut libc::c_void = &mut self_0.addQueue_data
                as *mut *mut SchedulerElem as *mut *mut libc::c_void;
            *pData = MemRealloc(
                self_0.addQueue_data as *mut libc::c_void,
                (self_0.addQueue_capacity as usize).wrapping_mul(elemSize as usize),
            );
        }
        let fresh0 = self_0.addQueue_size;
        self_0.addQueue_size = self_0.addQueue_size + 1;
        *(self_0.addQueue_data).offset(fresh0 as isize) = elem;
    } else {
        if (self_0.elems_capacity == self_0.elems_size) as libc::c_int as libc::c_long
            != 0
        {
            self_0
                .elems_capacity = if self_0.elems_capacity != 0 {
                self_0.elems_capacity * 2 as libc::c_int
            } else {
                1 as libc::c_int
            };
            let mut elemSize_0: usize = ::core::mem::size_of::<SchedulerElem>();
            let mut pData_0: *mut *mut libc::c_void = &mut self_0.elems_data
                as *mut *mut SchedulerElem as *mut *mut libc::c_void;
            *pData_0 = MemRealloc(
                self_0.elems_data as *mut libc::c_void,
                (self_0.elems_capacity as usize).wrapping_mul(elemSize_0 as usize),
            );
        }
        let fresh1 = self_0.elems_size;
        self_0.elems_size = self_0.elems_size + 1;
        *(self_0.elems_data).offset(fresh1 as isize) = elem;
    }
    return 0 as libc::c_int;
}
unsafe extern "C" fn LuaScheduler_Clear(mut L: *mut Lua) -> libc::c_int {
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < self_0.elems_size {
        let mut elem: *mut SchedulerElem = (self_0.elems_data).offset(i as isize);
        luaL_unref(L, -(10000 as libc::c_int), (*elem).fn_0 as libc::c_int);
        luaL_unref(L, -(10000 as libc::c_int), (*elem).arg as libc::c_int);
        i += 1;
    }
    self_0.elems_size = 0 as libc::c_int;
    return 0 as libc::c_int;
}
unsafe extern "C" fn LuaScheduler_Update(mut L: *mut Lua) -> libc::c_int {
    self_0.locked = 1 as libc::c_int != 0;
    qsort(
        self_0.elems_data as *mut libc::c_void,
        self_0.elems_size as libc::size_t,
        ::core::mem::size_of::<SchedulerElem>() as usize,
        Some(
            SortByWake
                as unsafe extern "C" fn(
                    *const libc::c_void,
                    *const libc::c_void,
                ) -> libc::c_int,
        ),
    );
    self_0.now = TimeStamp_Get();
    lua_getfield(
        L,
        -(10002 as libc::c_int),
        b"__error_handler__\0" as *const u8 as *const libc::c_char,
    );
    let mut handler: libc::c_int = lua_gettop(L);
    while self_0.elems_size != 0 {
        let mut elem: *mut SchedulerElem = (self_0.elems_data)
            .offset(self_0.elems_size as isize)
            .offset(-(1));
        if self_0.now < (*elem).tWake {
            break;
        }
        let mut dt: libc::c_double = TimeStamp_GetDifference(
            (*elem).tCreated,
            self_0.now,
        );
        Lua_PushRef(L, (*elem).fn_0);
        Lua_PushNumber(L, dt);
        Lua_PushRef(L, (*elem).arg);
        Lua_Call(L, 2 as libc::c_int, 0 as libc::c_int, handler);
        Lua_ReleaseRef(L, (*elem).fn_0);
        Lua_ReleaseRef(L, (*elem).arg);
        self_0.elems_size -= 1;
    }
    lua_settop(L, -(1 as libc::c_int) - 1 as libc::c_int);
    self_0.locked = 0 as libc::c_int != 0;
    while self_0.addQueue_size != 0 {
        self_0.addQueue_size -= 1;
        let mut elem_0: SchedulerElem = *(self_0.addQueue_data)
            .offset(self_0.addQueue_size as isize);
        if (self_0.elems_capacity == self_0.elems_size) as libc::c_int as libc::c_long
            != 0
        {
            self_0
                .elems_capacity = if self_0.elems_capacity != 0 {
                self_0.elems_capacity * 2 as libc::c_int
            } else {
                1 as libc::c_int
            };
            let mut elemSize: usize = ::core::mem::size_of::<SchedulerElem>();
            let mut pData: *mut *mut libc::c_void = &mut self_0.elems_data
                as *mut *mut SchedulerElem as *mut *mut libc::c_void;
            *pData = MemRealloc(
                self_0.elems_data as *mut libc::c_void,
                (self_0.elems_capacity as usize).wrapping_mul(elemSize as usize),
            );
        }
        let fresh2 = self_0.elems_size;
        self_0.elems_size = self_0.elems_size + 1;
        *(self_0.elems_data).offset(fresh2 as isize) = elem_0;
    }
    return 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn LuaScheduler_Init(mut L: *mut Lua) {
    self_0.elems_capacity = 0 as libc::c_int;
    self_0.elems_size = 0 as libc::c_int;
    self_0.elems_data = 0 as *mut SchedulerElem;
    self_0.addQueue_capacity = 0 as libc::c_int;
    self_0.addQueue_size = 0 as libc::c_int;
    self_0.addQueue_data = 0 as *mut SchedulerElem;
    self_0.now = TimeStamp_Get();
    self_0.locked = 0 as libc::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn LuaScheduler_Register(mut L: *mut Lua) {
    Lua_SetFn(
        L,
        b"Schedule\0" as *const u8 as *const libc::c_char,
        Some(LuaScheduler_Add as unsafe extern "C" fn(*mut Lua) -> libc::c_int),
    );
    Lua_SetFn(
        L,
        b"SchedulerClear\0" as *const u8 as *const libc::c_char,
        Some(LuaScheduler_Clear as unsafe extern "C" fn(*mut Lua) -> libc::c_int),
    );
    Lua_SetFn(
        L,
        b"SchedulerUpdate\0" as *const u8 as *const libc::c_char,
        Some(LuaScheduler_Update as unsafe extern "C" fn(*mut Lua) -> libc::c_int),
    );
}
