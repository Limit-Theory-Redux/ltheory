use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
use crate::PhxSignal::*;
use std::io::{self, Write};

extern "C" {
    pub type HashMap;
    pub type __sFILEX;
    fn Fatal(_: cstr, _: ...);
    fn qsort(
        __base: *mut libc::c_void,
        __nel: libc::size_t,
        __width: libc::size_t,
        __compar: Option::<
            unsafe extern "C" fn(*const libc::c_void, *const libc::c_void) -> libc::c_int,
        >,
    );
    fn HashMap_Create(keySize: u32, capacity: u32) -> *mut HashMap;
    fn HashMap_Free(_: *mut HashMap);
    fn HashMap_GetRaw(_: *mut HashMap, keyHash: u64) -> *mut libc::c_void;
    fn HashMap_SetRaw(_: *mut HashMap, keyHash: u64, value: *mut libc::c_void);
    fn sqrt(_: f64) -> f64;
    fn Signal_AddHandlerAll(_: SignalHandler);
    fn Signal_RemoveHandlerAll(_: SignalHandler);
    fn fflush(_: *mut FILE) -> libc::c_int;
    fn printf(_: *const libc::c_char, _: ...) -> libc::c_int;
    fn puts(_: *const libc::c_char) -> libc::c_int;
    fn TimeStamp_Get() -> TimeStamp;
    fn TimeStamp_GetElapsed(start: TimeStamp) -> f64;
    fn TimeStamp_ToDouble(_: TimeStamp) -> f64;
}
pub type __i64_t = libc::c_longlong;
pub type __darwin_off_t = __i64_t;
pub type cstr = *const libc::c_char;
pub type TimeStamp = u64;
pub type Signal = libc::c_int;
pub type SignalHandler = Option::<unsafe extern "C" fn(Signal) -> ()>;
pub type FILE = __sFILE;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sFILE {
    pub _p: *mut libc::c_uchar,
    pub _r: libc::c_int,
    pub _w: libc::c_int,
    pub _flags: libc::c_short,
    pub _file: libc::c_short,
    pub _bf: __sbuf,
    pub _lbfsize: libc::c_int,
    pub _cookie: *mut libc::c_void,
    pub _close: Option::<unsafe extern "C" fn(*mut libc::c_void) -> libc::c_int>,
    pub _read: Option::<
        unsafe extern "C" fn(
            *mut libc::c_void,
            *mut libc::c_char,
            libc::c_int,
        ) -> libc::c_int,
    >,
    pub _seek: Option::<
        unsafe extern "C" fn(*mut libc::c_void, fpos_t, libc::c_int) -> fpos_t,
    >,
    pub _write: Option::<
        unsafe extern "C" fn(
            *mut libc::c_void,
            *const libc::c_char,
            libc::c_int,
        ) -> libc::c_int,
    >,
    pub _ub: __sbuf,
    pub _extra: *mut __sFILEX,
    pub _ur: libc::c_int,
    pub _ubuf: [libc::c_uchar; 3],
    pub _nbuf: [libc::c_uchar; 1],
    pub _lb: __sbuf,
    pub _blksize: libc::c_int,
    pub _offset: fpos_t,
}
pub type fpos_t = __darwin_off_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sbuf {
    pub _base: *mut libc::c_uchar,
    pub _size: libc::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Scope {
    pub name: cstr,
    pub last: TimeStamp,
    pub frame: TimeStamp,
    pub total: TimeStamp,
    pub count: f64,
    pub mean: f64,
    pub var: f64,
    pub min: f64,
    pub max: f64,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Profiler {
    pub map: *mut HashMap,
    pub stackIndex: libc::c_int,
    pub stack: [*mut Scope; 128],
    pub scopeList_size: i32,
    pub scopeList_capacity: i32,
    pub scopeList_data: *mut *mut Scope,
    pub start: TimeStamp,
}


#[inline]
unsafe extern "C" fn Max(
    mut a: f64,
    mut b: f64,
) -> f64 {
    return if a > b { a } else { b };
}
#[inline]
unsafe extern "C" fn Min(
    mut a: f64,
    mut b: f64,
) -> f64 {
    return if a < b { a } else { b };
}
#[inline]
unsafe extern "C" fn Sqrt(mut t: f64) -> f64 {
    return sqrt(t);
}

static mut this: Profiler = Profiler {
    map: 0 as *const HashMap as *mut HashMap,
    stackIndex: 0,
    stack: [0 as *const Scope as *mut Scope; 128],
    scopeList_size: 0,
    scopeList_capacity: 0,
    scopeList_data: 0 as *const *mut Scope as *mut *mut Scope,
    start: 0,
};
static mut profiling: bool = 0 as libc::c_int != 0;
unsafe extern "C" fn Scope_Create(mut name: cstr) -> *mut Scope {
    let mut scope: *mut Scope = MemAlloc(
        ::core::mem::size_of::<Scope>() as usize,
    ) as *mut Scope;
    (*scope).name = StrDup(name);
    (*scope).last = 0 as libc::c_int as TimeStamp;
    (*scope).frame = 0 as libc::c_int as TimeStamp;
    (*scope).total = 0 as libc::c_int as TimeStamp;
    (*scope).count = 0.0f64;
    (*scope).mean = 0.0f64;
    (*scope).var = 0.0f64;
    (*scope).min = 1e30f64;
    (*scope).max = -1e30f64;
    if (this.scopeList_capacity == this.scopeList_size) as libc::c_int
        as libc::c_long != 0
    {
        this
            .scopeList_capacity = if this.scopeList_capacity != 0 {
            this.scopeList_capacity * 2 as libc::c_int
        } else {
            1 as libc::c_int
        };
        let mut elemSize: usize = ::core::mem::size_of::<*mut Scope>();
        let mut pData: *mut *mut libc::c_void = &mut this.scopeList_data
            as *mut *mut *mut Scope as *mut *mut libc::c_void;
        *pData = MemRealloc(
            this.scopeList_data as *mut libc::c_void,
            (this.scopeList_capacity as usize).wrapping_mul(elemSize as usize),
        );
    }
    let fresh0 = this.scopeList_size;
    this.scopeList_size = this.scopeList_size + 1;
    let ref mut fresh1 = *(this.scopeList_data).offset(fresh0 as isize);
    *fresh1 = scope;
    return scope;
}
unsafe extern "C" fn Scope_Free(mut scope: *mut Scope) {
    StrFree((*scope).name);
    MemFree(scope as *const libc::c_void);
}
unsafe extern "C" fn SortScopes(
    mut pa: *const libc::c_void,
    mut pb: *const libc::c_void,
) -> libc::c_int {
    let mut a: *const Scope = *(pa as *mut *const Scope);
    let mut b: *const Scope = *(pb as *mut *const Scope);
    return if (*b).total < (*a).total {
        -(1 as libc::c_int)
    } else if (*b).total == (*a).total {
        0 as libc::c_int
    } else {
        1 as libc::c_int
    };
}
unsafe extern "C" fn Profiler_GetScope(mut name: cstr) -> *mut Scope {
    let mut scope: *mut Scope = HashMap_GetRaw(this.map, name as libc::size_t as u64)
        as *mut Scope;
    if !scope.is_null() {
        return scope;
    }
    scope = Scope_Create(name);
    HashMap_SetRaw(this.map, name as libc::size_t as u64, scope as *mut libc::c_void);
    return scope;
}
unsafe extern "C" fn Profiler_SignalHandler(mut s: Signal) {
    Profiler_Backtrace();
}
#[no_mangle]
pub unsafe extern "C" fn Profiler_Enable() {
    profiling = 1 as libc::c_int != 0;
    this
        .map = HashMap_Create(
        ::core::mem::size_of::<*mut libc::c_void>() as libc::c_ulong as u32,
        (2 as libc::c_int * 1024 as libc::c_int) as u32,
    );
    this.scopeList_capacity = 0 as libc::c_int;
    this.scopeList_size = 0 as libc::c_int;
    this.scopeList_data = 0 as *mut *mut Scope;
    if (this.scopeList_capacity < 1024 as libc::c_int) as libc::c_long
        != 0
    {
        this.scopeList_capacity = 1024 as libc::c_int;
        let mut elemSize: usize = ::core::mem::size_of::<*mut Scope>();
        let mut pData: *mut *mut libc::c_void = &mut this.scopeList_data
            as *mut *mut *mut Scope as *mut *mut libc::c_void;
        *pData = MemRealloc(
            this.scopeList_data as *mut libc::c_void,
            (this.scopeList_capacity as usize).wrapping_mul(elemSize as usize),
        );
    }
    this.stackIndex = -(1 as libc::c_int);
    this.start = TimeStamp_Get();
    Profiler_Begin(b"[Root]\0" as *const u8 as *const libc::c_char);
    Signal_AddHandlerAll(
        Some(Profiler_SignalHandler as unsafe extern "C" fn(Signal) -> ()),
    );
}
#[no_mangle]
pub unsafe extern "C" fn Profiler_Disable() {
    if this.stackIndex != 0 as libc::c_int {
        Fatal(
            b"Profiler_Disable: Cannot stop profiler from within a profiled section\0"
                as *const u8 as *const libc::c_char,
        );
    }
    Profiler_End();
    let mut total: f64 = TimeStamp_GetElapsed(this.start);
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < this.scopeList_size {
        let mut scope: *mut Scope = *(this.scopeList_data).offset(i as isize);
        (*scope).var /= (*scope).count - 1.0f64;
        (*scope).var = Sqrt((*scope).var);
        i += 1;
    }
    qsort(
        this.scopeList_data as *mut libc::c_void,
        this.scopeList_size as libc::size_t,
        ::core::mem::size_of::<*mut Scope>(),
        Some(
            SortScopes
                as unsafe extern "C" fn(
                    *const libc::c_void,
                    *const libc::c_void,
                ) -> libc::c_int,
        ),
    );
    puts(
        b"-- PHX PROFILER -------------------------------------\0" as *const u8
            as *const libc::c_char,
    );
    let mut cumulative: f64 = 0 as libc::c_int as f64;
    let mut i_0: libc::c_int = 0 as libc::c_int;
    while i_0 < this.scopeList_size {
        let mut scope_0: *mut Scope = *(this.scopeList_data).offset(i_0 as isize);
        let mut scopeTotal: f64 = TimeStamp_ToDouble((*scope_0).total);
        cumulative += scopeTotal;
        if !(scopeTotal / total < 0.01f64 && (*scope_0).max < 0.01f64) {
            printf(
                b"%*.1f%% %*.0f%% %*.0fms  [%*.2f, %*.2f] %*.2f  / %*.2f  (%*.0f%%)  |  %s\n\0"
                    as *const u8 as *const libc::c_char,
                5 as libc::c_int,
                100.0f64 * (scopeTotal / total),
                4 as libc::c_int,
                100.0f64 * (cumulative / total),
                6 as libc::c_int,
                1000.0f64 * scopeTotal,
                6 as libc::c_int,
                1000.0f64 * (*scope_0).min,
                6 as libc::c_int,
                1000.0f64 * (*scope_0).max,
                6 as libc::c_int,
                1000.0f64 * (*scope_0).mean,
                5 as libc::c_int,
                1000.0f64 * (*scope_0).var,
                4 as libc::c_int,
                100.0f64 * ((*scope_0).var / (*scope_0).mean),
                (*scope_0).name,
            );
        }
        i_0 += 1;
    }
    puts(
        b"-----------------------------------------------------\0" as *const u8
            as *const libc::c_char,
    );
    
    let mut i_1: libc::c_int = 0 as libc::c_int;
    while i_1 < this.scopeList_size {
        Scope_Free(*(this.scopeList_data).offset(i_1 as isize));
        i_1 += 1;
    }
    MemFree(this.scopeList_data as *const libc::c_void);
    HashMap_Free(this.map);
    profiling = 0 as libc::c_int != 0;
    Signal_RemoveHandlerAll(
        Some(Profiler_SignalHandler as unsafe extern "C" fn(Signal) -> ()),
    );
}
#[no_mangle]
pub unsafe extern "C" fn Profiler_Begin(mut name: cstr) {
    if !profiling {
        return;
    }
    if this.stackIndex + 1 as libc::c_int >= 128 as libc::c_int {
        Profiler_Backtrace();
        Fatal(
            b"Profiler_Begin: Maximum stack depth exceeded\0" as *const u8
                as *const libc::c_char,
        );
    }
    let mut now: TimeStamp = TimeStamp_Get();
    if this.stackIndex >= 0 as libc::c_int {
        let mut prev: *mut Scope = this.stack[this.stackIndex as usize];
        (*prev)
            .frame = ((*prev).frame as libc::c_ulonglong)
            .wrapping_add(now.wrapping_sub((*prev).last)) as TimeStamp as TimeStamp;
        (*prev).last = now;
    }
    this.stackIndex += 1;
    let mut curr: *mut Scope = Profiler_GetScope(name);
    this.stack[this.stackIndex as usize] = curr;
    (*curr).last = now;
}
#[no_mangle]
pub unsafe extern "C" fn Profiler_End() {
    if !profiling {
        return;
    }
    if this.stackIndex < 0 as libc::c_int {
        Profiler_Backtrace();
        Fatal(
            b"Profiler_End: Attempting to pop an empty stack\0" as *const u8
                as *const libc::c_char,
        );
    }
    let mut now: TimeStamp = TimeStamp_Get();
    let mut prev: *mut Scope = this.stack[this.stackIndex as usize];
    (*prev)
        .frame = ((*prev).frame as libc::c_ulonglong)
        .wrapping_add(now.wrapping_sub((*prev).last)) as TimeStamp as TimeStamp;
    this.stackIndex -= 1;
    if this.stackIndex >= 0 as libc::c_int {
        let mut curr: *mut Scope = this.stack[this.stackIndex as usize];
        (*curr).last = now;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Profiler_SetValue(mut name: cstr, mut value: libc::c_int) {}
#[no_mangle]
pub unsafe extern "C" fn Profiler_LoopMarker() {
    if !profiling {
        return;
    }
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < this.scopeList_size {
        let mut scope: *mut Scope = *(this.scopeList_data).offset(i as isize);
        if (*scope).frame as f64 > 0.0f64 {
            (*scope)
                .total = ((*scope).total as libc::c_ulonglong)
                .wrapping_add((*scope).frame) as TimeStamp as TimeStamp;
            let mut frame: f64 = TimeStamp_ToDouble((*scope).frame);
            (*scope).min = Min((*scope).min, frame);
            (*scope).max = Max((*scope).max, frame);
            (*scope).count += 1.0f64;
            let mut d1: f64 = frame - (*scope).mean;
            (*scope).mean += d1 / (*scope).count;
            let mut d2: f64 = frame - (*scope).mean;
            (*scope).var += d1 * d2;
            (*scope).frame = 0 as libc::c_int as TimeStamp;
        }
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Profiler_Backtrace() {
    if !profiling {
        return;
    }
    puts(b"PHX Profiler Backtrace:\0" as *const u8 as *const libc::c_char);
    let mut i: libc::c_int = 0 as libc::c_int;
    while i <= this.stackIndex {
        let mut index: libc::c_int = this.stackIndex - i;
        printf(
            b"  [%i] %s\n\0" as *const u8 as *const libc::c_char,
            index,
            (*this.stack[index as usize]).name,
        );
        i += 1;
    }
    io::stdout().flush().unwrap();
}
