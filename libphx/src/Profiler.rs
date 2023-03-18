use crate::internal::Memory::*;
use crate::Common::*;
use crate::HashMap::*;
use crate::Math::Vec3;
use crate::Signal::*;
use crate::TimeStamp::*;
use libc;
use std::io::{self, Write};

pub type Signal = i32;
pub type SignalHandler = Option<unsafe extern "C" fn(Signal) -> ()>;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Scope {
    pub name: *const libc::c_char,
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
    pub stackIndex: i32,
    pub stack: [*mut Scope; 128],
    pub scopeList_size: i32,
    pub scopeList_capacity: i32,
    pub scopeList_data: *mut *mut Scope,
    pub start: TimeStamp,
}

static mut this: Profiler = Profiler {
    map: std::ptr::null_mut(),
    stackIndex: 0,
    stack: [std::ptr::null_mut(); 128],
    scopeList_size: 0,
    scopeList_capacity: 0,
    scopeList_data: std::ptr::null_mut(),
    start: 0,
};

static mut profiling: bool = false;

unsafe extern "C" fn Scope_Create(mut name: *const libc::c_char) -> *mut Scope {
    let mut scope: *mut Scope = MemAlloc(::core::mem::size_of::<Scope>()) as *mut Scope;
    (*scope).name = StrDup(name);
    (*scope).last = 0_i32 as TimeStamp;
    (*scope).frame = 0_i32 as TimeStamp;
    (*scope).total = 0_i32 as TimeStamp;
    (*scope).count = 0.0f64;
    (*scope).mean = 0.0f64;
    (*scope).var = 0.0f64;
    (*scope).min = 1e30f64;
    (*scope).max = -1e30f64;
    if (this.scopeList_capacity == this.scopeList_size) as i32 as libc::c_long != 0 {
        this.scopeList_capacity = if this.scopeList_capacity != 0 {
            this.scopeList_capacity * 2_i32
        } else {
            1_i32
        };
        let mut elemSize: usize = ::core::mem::size_of::<*mut Scope>();
        let mut pData: *mut *mut libc::c_void =
            &mut this.scopeList_data as *mut *mut *mut Scope as *mut *mut libc::c_void;
        *pData = MemRealloc(
            this.scopeList_data as *mut libc::c_void,
            (this.scopeList_capacity as usize).wrapping_mul(elemSize),
        );
    }
    let fresh0 = this.scopeList_size;
    this.scopeList_size += 1;
    let ref mut fresh1 = *(this.scopeList_data).offset(fresh0 as isize);
    *fresh1 = scope;
    scope
}

unsafe extern "C" fn Scope_Free(mut scope: *mut Scope) {
    StrFree((*scope).name);
    MemFree(scope as *const libc::c_void);
}

unsafe extern "C" fn SortScopes(mut pa: *const libc::c_void, mut pb: *const libc::c_void) -> i32 {
    let mut a: *const Scope = *(pa as *mut *const Scope);
    let mut b: *const Scope = *(pb as *mut *const Scope);
    if (*b).total < (*a).total {
        -1_i32
    } else if (*b).total == (*a).total {
        0_i32
    } else {
        1_i32
    }
}

unsafe extern "C" fn Profiler_GetScope(mut name: *const libc::c_char) -> *mut Scope {
    let mut scope: *mut Scope = HashMap_GetRaw(this.map, name as usize as u64) as *mut Scope;
    if !scope.is_null() {
        return scope;
    }
    scope = Scope_Create(name);
    HashMap_SetRaw(this.map, name as usize as u64, scope as *mut libc::c_void);
    scope
}

unsafe extern "C" fn Profiler_SignalHandler(mut _s: Signal) {
    Profiler_Backtrace();
}

#[no_mangle]
pub unsafe extern "C" fn Profiler_Enable() {
    profiling = true;
    this.map = HashMap_Create(
        ::core::mem::size_of::<*mut libc::c_void>() as libc::c_ulong as u32,
        (2_i32 * 1024_i32) as u32,
    );
    this.scopeList_capacity = 0_i32;
    this.scopeList_size = 0_i32;
    this.scopeList_data = std::ptr::null_mut();
    if (this.scopeList_capacity < 1024_i32) as libc::c_long != 0 {
        this.scopeList_capacity = 1024_i32;
        let mut elemSize: usize = ::core::mem::size_of::<*mut Scope>();
        let mut pData: *mut *mut libc::c_void =
            &mut this.scopeList_data as *mut *mut *mut Scope as *mut *mut libc::c_void;
        *pData = MemRealloc(
            this.scopeList_data as *mut libc::c_void,
            (this.scopeList_capacity as usize).wrapping_mul(elemSize),
        );
    }
    this.stackIndex = -1_i32;
    this.start = TimeStamp_Get();
    Profiler_Begin(b"[Root]\0" as *const u8 as *const libc::c_char);
    Signal_AddHandlerAll(Some(
        Profiler_SignalHandler as unsafe extern "C" fn(Signal) -> (),
    ));
}

#[no_mangle]
pub unsafe extern "C" fn Profiler_Disable() {
    if this.stackIndex != 0_i32 {
        Fatal(
            b"Profiler_Disable: Cannot stop profiler from within a profiled section\0" as *const u8
                as *const libc::c_char,
        );
    }
    Profiler_End();
    let mut total: f64 = TimeStamp_GetElapsed(this.start);
    let mut i: i32 = 0_i32;
    while i < this.scopeList_size {
        let mut scope: *mut Scope = *(this.scopeList_data).offset(i as isize);
        (*scope).var /= (*scope).count - 1.0f64;
        (*scope).var = f64::sqrt((*scope).var);
        i += 1;
    }
    libc::qsort(
        this.scopeList_data as *mut libc::c_void,
        this.scopeList_size as usize,
        ::core::mem::size_of::<*mut Scope>(),
        Some(SortScopes as unsafe extern "C" fn(*const libc::c_void, *const libc::c_void) -> i32),
    );
    libc::puts(
        b"-- PHX PROFILER -------------------------------------\0" as *const u8
            as *const libc::c_char,
    );
    let mut cumulative: f64 = 0_i32 as f64;
    let mut i_0: i32 = 0_i32;
    while i_0 < this.scopeList_size {
        let mut scope_0: *mut Scope = *(this.scopeList_data).offset(i_0 as isize);
        let mut scopeTotal: f64 = TimeStamp_ToDouble((*scope_0).total);
        cumulative += scopeTotal;
        if !(scopeTotal / total < 0.01f64 && (*scope_0).max < 0.01f64) {
            libc::printf(
                b"%*.1f%% %*.0f%% %*.0fms  [%*.2f, %*.2f] %*.2f  / %*.2f  (%*.0f%%)  |  %s\n\0"
                    as *const u8 as *const libc::c_char,
                5_i32,
                100.0f64 * (scopeTotal / total),
                4_i32,
                100.0f64 * (cumulative / total),
                6_i32,
                1000.0f64 * scopeTotal,
                6_i32,
                1000.0f64 * (*scope_0).min,
                6_i32,
                1000.0f64 * (*scope_0).max,
                6_i32,
                1000.0f64 * (*scope_0).mean,
                5_i32,
                1000.0f64 * (*scope_0).var,
                4_i32,
                100.0f64 * ((*scope_0).var / (*scope_0).mean),
                (*scope_0).name,
            );
        }
        i_0 += 1;
    }
    libc::puts(
        b"-----------------------------------------------------\0" as *const u8
            as *const libc::c_char,
    );

    let mut i_1: i32 = 0_i32;
    while i_1 < this.scopeList_size {
        Scope_Free(*(this.scopeList_data).offset(i_1 as isize));
        i_1 += 1;
    }
    MemFree(this.scopeList_data as *const libc::c_void);
    HashMap_Free(this.map);
    profiling = false;
    Signal_RemoveHandlerAll(Some(
        Profiler_SignalHandler as unsafe extern "C" fn(Signal) -> (),
    ));
}

#[no_mangle]
pub unsafe extern "C" fn Profiler_Begin(mut name: *const libc::c_char) {
    if !profiling {
        return;
    }
    if this.stackIndex + 1_i32 >= 128_i32 {
        Profiler_Backtrace();
        Fatal(
            b"Profiler_Begin: Maximum stack depth exceeded\0" as *const u8 as *const libc::c_char,
        );
    }
    let mut now: TimeStamp = TimeStamp_Get();
    if this.stackIndex >= 0_i32 {
        let mut prev: *mut Scope = this.stack[this.stackIndex as usize];
        (*prev).frame =
            (*prev).frame.wrapping_add(now.wrapping_sub((*prev).last)) as TimeStamp as TimeStamp;
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
    if this.stackIndex < 0_i32 {
        Profiler_Backtrace();
        Fatal(
            b"Profiler_End: Attempting to pop an empty stack\0" as *const u8 as *const libc::c_char,
        );
    }
    let mut now: TimeStamp = TimeStamp_Get();
    let mut prev: *mut Scope = this.stack[this.stackIndex as usize];
    (*prev).frame =
        (*prev).frame.wrapping_add(now.wrapping_sub((*prev).last)) as TimeStamp as TimeStamp;
    this.stackIndex -= 1;
    if this.stackIndex >= 0_i32 {
        let mut curr: *mut Scope = this.stack[this.stackIndex as usize];
        (*curr).last = now;
    }
}

#[no_mangle]
pub unsafe extern "C" fn Profiler_SetValue(mut _name: *const libc::c_char, mut _value: i32) {}

#[no_mangle]
pub unsafe extern "C" fn Profiler_LoopMarker() {
    if !profiling {
        return;
    }
    let mut i: i32 = 0_i32;
    while i < this.scopeList_size {
        let mut scope: *mut Scope = *(this.scopeList_data).offset(i as isize);
        if (*scope).frame as f64 > 0.0f64 {
            (*scope).total = (*scope).total.wrapping_add((*scope).frame) as TimeStamp as TimeStamp;
            let mut frame: f64 = TimeStamp_ToDouble((*scope).frame);
            (*scope).min = f64::min((*scope).min, frame);
            (*scope).max = f64::max((*scope).max, frame);
            (*scope).count += 1.0f64;
            let mut d1: f64 = frame - (*scope).mean;
            (*scope).mean += d1 / (*scope).count;
            let mut d2: f64 = frame - (*scope).mean;
            (*scope).var += d1 * d2;
            (*scope).frame = 0_i32 as TimeStamp;
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn Profiler_Backtrace() {
    if !profiling {
        return;
    }
    libc::puts(b"PHX Profiler Backtrace:\0" as *const u8 as *const libc::c_char);
    let mut i: i32 = 0_i32;
    while i <= this.stackIndex {
        let mut index: i32 = this.stackIndex - i;
        libc::printf(
            b"  [%i] %s\n\0" as *const u8 as *const libc::c_char,
            index,
            (*this.stack[index as usize]).name,
        );
        i += 1;
    }
    io::stdout().flush().unwrap();
}
