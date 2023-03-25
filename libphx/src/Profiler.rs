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

#[derive(Clone)]
#[repr(C)]
pub struct Profiler {
    pub map: *mut HashMap,
    pub stackIndex: i32,
    pub stack: [*mut Scope; 128],
    pub scopeList: Vec<*mut Scope>,
    pub start: TimeStamp,
}

static mut this: Profiler = Profiler {
    map: std::ptr::null_mut(),
    stackIndex: 0,
    stack: [std::ptr::null_mut(); 128],
    scopeList: Vec::new(),
    start: 0,
};

static mut profiling: bool = false;

unsafe extern "C" fn Scope_Create(name: *const libc::c_char) -> *mut Scope {
    let mut scope = MemNew!(Scope);
    (*scope).name = StrDup(name);
    (*scope).last = 0 as TimeStamp;
    (*scope).frame = 0 as TimeStamp;
    (*scope).total = 0 as TimeStamp;
    (*scope).count = 0.0f64;
    (*scope).mean = 0.0f64;
    (*scope).var = 0.0f64;
    (*scope).min = 1e30f64;
    (*scope).max = -1e30f64;
    this.scopeList.push(scope);
    scope
}

unsafe extern "C" fn Scope_Free(scope: *mut Scope) {
    StrFree((*scope).name);
    MemDelete!(scope);
}

unsafe extern "C" fn SortScopes(pa: *const libc::c_void, pb: *const libc::c_void) -> i32 {
    let mut a: *const Scope = *(pa as *mut *const Scope);
    let mut b: *const Scope = *(pb as *mut *const Scope);
    if (*b).total < (*a).total {
        -1
    } else if (*b).total == (*a).total {
        0
    } else {
        1
    }
}

unsafe extern "C" fn Profiler_GetScope(name: *const libc::c_char) -> *mut Scope {
    let mut scope: *mut Scope = HashMap_GetRaw(this.map, name as usize as u64) as *mut Scope;
    if !scope.is_null() {
        return scope;
    }
    scope = Scope_Create(name);
    HashMap_SetRaw(this.map, name as usize as u64, scope as *mut _);
    scope
}

unsafe extern "C" fn Profiler_SignalHandler(_s: Signal) {
    Profiler_Backtrace();
}

#[no_mangle]
pub unsafe extern "C" fn Profiler_Enable() {
    profiling = true;
    this.map = HashMap_Create(
        std::mem::size_of::<*mut libc::c_void>() as libc::c_ulong as u32,
        (2 * 1024) as u32,
    );
    this.scopeList = Vec::new();
    this.scopeList.reserve(1024);
    this.stackIndex = -1;
    this.start = TimeStamp_Get();
    Profiler_Begin(b"[Root]\0" as *const u8 as *const libc::c_char);
    Signal_AddHandlerAll(Some(
        Profiler_SignalHandler as unsafe extern "C" fn(Signal) -> (),
    ));
}

#[no_mangle]
pub unsafe extern "C" fn Profiler_Disable() {
    if this.stackIndex != 0 {
        Fatal(
            b"Profiler_Disable: Cannot stop profiler from within a profiled section\0" as *const u8
                as *const libc::c_char,
        );
    }
    Profiler_End();
    let mut total: f64 = TimeStamp_GetElapsed(this.start);
    let mut i: i32 = 0;
    while i < this.scopeList.len() as i32 {
        let mut scope: &mut Scope = &mut *this.scopeList[i as usize];
        (*scope).var /= (*scope).count - 1.0f64;
        (*scope).var = f64::sqrt((*scope).var);
        i += 1;
    }
    libc::qsort(
        this.scopeList.as_mut_ptr() as *mut _,
        this.scopeList.len() as i32 as usize,
        std::mem::size_of::<*mut Scope>(),
        Some(SortScopes as unsafe extern "C" fn(*const libc::c_void, *const libc::c_void) -> i32),
    );
    libc::puts(
        b"-- PHX PROFILER -------------------------------------\0" as *const u8
            as *const libc::c_char,
    );
    let mut cumulative: f64 = 0.0;
    let mut i_0: i32 = 0;
    while i_0 < this.scopeList.len() as i32 {
        let mut scope: &mut Scope = &mut *this.scopeList[i_0 as usize];
        let mut scopeTotal: f64 = TimeStamp_ToDouble((*scope).total);
        cumulative += scopeTotal;
        if !(scopeTotal / total < 0.01f64 && (*scope).max < 0.01f64) {
            libc::printf(
                b"%*.1f%% %*.0f%% %*.0fms  [%*.2f, %*.2f] %*.2f  / %*.2f  (%*.0f%%)  |  %s\n\0"
                    as *const u8 as *const libc::c_char,
                5,
                100.0f64 * (scopeTotal / total),
                4,
                100.0f64 * (cumulative / total),
                6,
                1000.0f64 * scopeTotal,
                6,
                1000.0f64 * (*scope).min,
                6,
                1000.0f64 * (*scope).max,
                6,
                1000.0f64 * (*scope).mean,
                5,
                1000.0f64 * (*scope).var,
                4,
                100.0f64 * ((*scope).var / (*scope).mean),
                (*scope).name,
            );
        }
        i_0 += 1;
    }
    libc::puts(
        b"-----------------------------------------------------\0" as *const u8
            as *const libc::c_char,
    );

    for scope in this.scopeList.iter() {
        Scope_Free(*scope);
    }
    HashMap_Free(this.map);
    profiling = false;
    Signal_RemoveHandlerAll(Some(
        Profiler_SignalHandler as unsafe extern "C" fn(Signal) -> (),
    ));
}

#[no_mangle]
pub unsafe extern "C" fn Profiler_Begin(name: *const libc::c_char) {
    if !profiling {
        return;
    }
    if this.stackIndex + 1 >= 128 {
        Profiler_Backtrace();
        Fatal(
            b"Profiler_Begin: Maximum stack depth exceeded\0" as *const u8 as *const libc::c_char,
        );
    }
    let mut now: TimeStamp = TimeStamp_Get();
    if this.stackIndex >= 0 {
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
    if this.stackIndex < 0 {
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
    if this.stackIndex >= 0 {
        let mut curr: *mut Scope = this.stack[this.stackIndex as usize];
        (*curr).last = now;
    }
}

#[no_mangle]
pub unsafe extern "C" fn Profiler_SetValue(_name: *const libc::c_char, _value: i32) {}

#[no_mangle]
pub unsafe extern "C" fn Profiler_LoopMarker() {
    if !profiling {
        return;
    }
    let mut i: i32 = 0;
    while i < this.scopeList.len() as i32 {
        let mut scope: &mut Scope = &mut *this.scopeList[i as usize];
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
            (*scope).frame = 0 as TimeStamp;
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
    let mut i: i32 = 0;
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
