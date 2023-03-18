use crate::internal::Memory::*;
use crate::Lua::*;
use crate::TimeStamp::*;
use glam::Vec3;
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

pub type TimeStamp = u64;
pub type lua_Number = f64;
pub type lua_Integer = libc::ptrdiff_t;
pub type Lua = lua_State;
pub type LuaFn = Option<unsafe extern "C" fn(*mut Lua) -> i32>;
pub type LuaRef = lua_Integer;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Scheduler {
    pub elems_size: i32,
    pub elems_capacity: i32,
    pub elems_data: *mut SchedulerElem,
    pub addQueue_size: i32,
    pub addQueue_capacity: i32,
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

static mut this: Scheduler = Scheduler {
    elems_size: 0,
    elems_capacity: 0,
    elems_data: std::ptr::null_mut(),
    addQueue_size: 0,
    addQueue_capacity: 0,
    addQueue_data: std::ptr::null_mut(),
    now: 0,
    locked: false,
};
unsafe extern "C" fn SortByWake(mut pa: *const libc::c_void, mut pb: *const libc::c_void) -> i32 {
    let mut a: *const SchedulerElem = pa as *const SchedulerElem;
    let mut b: *const SchedulerElem = pb as *const SchedulerElem;
    if (*a).tWake < (*b).tWake {
        1_i32
    } else if (*a).tWake == (*b).tWake {
        0_i32
    } else {
        -1_i32
    }
}

unsafe extern "C" fn LuaScheduler_Add(mut L: *mut Lua) -> i32 {
    let mut elem: SchedulerElem = SchedulerElem {
        fn_0: 0,
        arg: 0,
        tCreated: 0,
        tWake: 0,
    };
    let mut timeToWake: f64 = lua_tonumber(L, lua_gettop(L));
    elem.tCreated = this.now;
    elem.tWake = TimeStamp_GetRelative(this.now, timeToWake);
    lua_settop(L, -1_i32 - 1_i32);
    elem.arg = Lua_GetRef(L);
    elem.fn_0 = Lua_GetRef(L);
    if this.locked {
        if (this.addQueue_capacity == this.addQueue_size) as i32 as libc::c_long != 0 {
            this.addQueue_capacity = if this.addQueue_capacity != 0 {
                this.addQueue_capacity * 2_i32
            } else {
                1_i32
            };
            let mut elemSize: usize = ::core::mem::size_of::<SchedulerElem>();
            let mut pData: *mut *mut libc::c_void =
                &mut this.addQueue_data as *mut *mut SchedulerElem as *mut *mut libc::c_void;
            *pData = MemRealloc(
                this.addQueue_data as *mut libc::c_void,
                (this.addQueue_capacity as usize).wrapping_mul(elemSize),
            );
        }
        let fresh0 = this.addQueue_size;
        this.addQueue_size += 1;
        *(this.addQueue_data).offset(fresh0 as isize) = elem;
    } else {
        if (this.elems_capacity == this.elems_size) as libc::c_long != 0 {
            this.elems_capacity = if this.elems_capacity != 0 {
                this.elems_capacity * 2_i32
            } else {
                1_i32
            };
            let mut elemSize_0: usize = ::core::mem::size_of::<SchedulerElem>();
            let mut pData_0: *mut *mut libc::c_void =
                &mut this.elems_data as *mut *mut SchedulerElem as *mut *mut libc::c_void;
            *pData_0 = MemRealloc(
                this.elems_data as *mut libc::c_void,
                (this.elems_capacity as usize).wrapping_mul(elemSize_0),
            );
        }
        let fresh1 = this.elems_size;
        this.elems_size += 1;
        *(this.elems_data).offset(fresh1 as isize) = elem;
    }
    0_i32
}

unsafe extern "C" fn LuaScheduler_Clear(mut L: *mut Lua) -> i32 {
    let mut i: i32 = 0_i32;
    while i < this.elems_size {
        let mut elem: *mut SchedulerElem = (this.elems_data).offset(i as isize);
        luaL_unref(L, -10000_i32, (*elem).fn_0 as i32);
        luaL_unref(L, -10000_i32, (*elem).arg as i32);
        i += 1;
    }
    this.elems_size = 0_i32;
    0_i32
}

unsafe extern "C" fn LuaScheduler_Update(mut L: *mut Lua) -> i32 {
    this.locked = true;
    libc::qsort(
        this.elems_data as *mut libc::c_void,
        this.elems_size as usize,
        ::core::mem::size_of::<SchedulerElem>(),
        Some(SortByWake as unsafe extern "C" fn(*const libc::c_void, *const libc::c_void) -> i32),
    );
    this.now = TimeStamp_Get();
    lua_getfield(
        L,
        -10002_i32,
        b"__error_handler__\0" as *const u8 as *const libc::c_char,
    );
    let mut handler: i32 = lua_gettop(L);
    while this.elems_size != 0 {
        let mut elem: *mut SchedulerElem = (this.elems_data)
            .offset(this.elems_size as isize)
            .offset(-(1));
        if this.now < (*elem).tWake {
            break;
        }
        let mut dt: f64 = TimeStamp_GetDifference((*elem).tCreated, this.now);
        Lua_PushRef(L, (*elem).fn_0);
        Lua_PushNumber(L, dt);
        Lua_PushRef(L, (*elem).arg);
        Lua_Call(L, 2_i32, 0_i32, handler);
        Lua_ReleaseRef(L, (*elem).fn_0);
        Lua_ReleaseRef(L, (*elem).arg);
        this.elems_size -= 1;
    }
    lua_settop(L, -1_i32 - 1_i32);
    this.locked = false;
    while this.addQueue_size != 0 {
        this.addQueue_size -= 1;
        let mut elem_0: SchedulerElem = *(this.addQueue_data).offset(this.addQueue_size as isize);
        if (this.elems_capacity == this.elems_size) as libc::c_long != 0 {
            this.elems_capacity = if this.elems_capacity != 0 {
                this.elems_capacity * 2_i32
            } else {
                1_i32
            };
            let mut elemSize: usize = ::core::mem::size_of::<SchedulerElem>();
            let mut pData: *mut *mut libc::c_void =
                &mut this.elems_data as *mut *mut SchedulerElem as *mut *mut libc::c_void;
            *pData = MemRealloc(
                this.elems_data as *mut libc::c_void,
                (this.elems_capacity as usize).wrapping_mul(elemSize),
            );
        }
        let fresh2 = this.elems_size;
        this.elems_size += 1;
        *(this.elems_data).offset(fresh2 as isize) = elem_0;
    }
    0_i32
}

#[no_mangle]
pub unsafe extern "C" fn LuaScheduler_Init(mut _L: *mut Lua) {
    this.elems_capacity = 0_i32;
    this.elems_size = 0_i32;
    this.elems_data = std::ptr::null_mut();
    this.addQueue_capacity = 0_i32;
    this.addQueue_size = 0_i32;
    this.addQueue_data = std::ptr::null_mut();
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
