use std::cmp::Ordering;
use std::ffi::CStr;

use internal::*;
use tracing::{info, warn};

use super::*;
use crate::common::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Scope {
    pub name: *const libc::c_char,
    pub last: TimeStamp,
    pub frame: f64,
    pub total: f64,
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
    start: TimeStamp::zero(),
};

static mut profiling: bool = false;

unsafe extern "C" fn Scope_Create(name: *const libc::c_char) -> *mut Scope {
    let scope = MemNew!(Scope);
    (*scope).name = StrDup(name);
    (*scope).last = TimeStamp::zero();
    (*scope).frame = 0.0;
    (*scope).total = 0.0;
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

unsafe extern "C" fn Profiler_GetScope(name: *const libc::c_char) -> *mut Scope {
    let mut scope: *mut Scope = HashMap_GetRaw(this.map, name as usize as u64) as *mut Scope;
    if !scope.is_null() {
        return scope;
    }
    scope = Scope_Create(name);
    HashMap_SetRaw(this.map, name as usize as u64, scope as *mut _);
    scope
}

extern "C" fn Profiler_SignalHandler(_: Signal) {
    unsafe {
        Profiler_Backtrace();
    }
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
    this.start = TimeStamp::now();
    Profiler_Begin(c_str!("[Root]"));
    Signal_AddHandlerAll(Profiler_SignalHandler);
}

#[no_mangle]
pub unsafe extern "C" fn Profiler_Disable() {
    if this.stackIndex != 0 {
        panic!("Profiler_Disable: Cannot stop profiler from within a profiled section");
    }
    Profiler_End();

    let total = this.start.get_elapsed();
    let mut i = 0;
    while i < this.scopeList.len() {
        let scope: &mut Scope = &mut *this.scopeList[i];
        (*scope).var /= (*scope).count - 1.0f64;
        (*scope).var = f64::sqrt((*scope).var);
        i += 1;
    }

    this.scopeList.sort_by(|pa: &*mut Scope, pb: &*mut Scope| {
        let (a, b) = (&**pa, &**pb);

        if b.total < a.total {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });

    info!("-- PHX PROFILER -------------------------------------");
    info!("-- Measured timespan: {total}ms");
    info!("");
    info!(" Scope |  Cumul |    Scope |    Min |      Max |   Mean |   Var | Var/Mean | Name");
    info!("-------|--------|----------|--------|----------|--------|-------|----------|---------------------------");

    let mut cumulative = 0.0;
    let mut i = 0;
    while i < this.scopeList.len() {
        let scope = &mut *this.scopeList[i];
        let scopeTotal = (*scope).total as f64;

        cumulative += scopeTotal;

        if scopeTotal / total > 0.01 || (*scope).max > 0.01 {
            info!(
                "{:5.1}% | {:5.0}% | {:6.0}ms | {:6.2} | {:8.2} | {:6.2} | {:5.2} | {:7.0}% | {:?}",
                100.0 * (scopeTotal / total),
                100.0 * (cumulative / total),
                1000.0 * scopeTotal,
                1000.0 * (*scope).min,
                1000.0 * (*scope).max,
                1000.0 * (*scope).mean,
                1000.0 * (*scope).var,
                100.0 * ((*scope).var / (*scope).mean),
                CStr::from_ptr((*scope).name),
            );
        }

        i += 1;
    }

    info!("-------------------------------------------------------------------------------------------------------");

    for scope in this.scopeList.iter() {
        Scope_Free(*scope);
    }
    HashMap_Free(this.map);
    profiling = false;
    Signal_RemoveHandlerAll(Profiler_SignalHandler);
}

#[no_mangle]
pub unsafe extern "C" fn Profiler_Begin(name: *const libc::c_char) {
    if !profiling {
        return;
    }
    if this.stackIndex + 1 >= 128 {
        Profiler_Backtrace();
        warn!("Profiler_Begin: Maximum stack depth exceeded");
        return;
    }
    let now = TimeStamp::now();
    if this.stackIndex >= 0 {
        let prev: *mut Scope = this.stack[this.stackIndex as usize];
        (*prev).frame += (*prev).last.get_difference(&now);
        (*prev).last = now;
    }
    this.stackIndex += 1;
    let curr: *mut Scope = Profiler_GetScope(name);
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
        panic!("Profiler_End: Attempting to pop an empty stack");
    }
    let now = TimeStamp::now();
    let prev: *mut Scope = this.stack[this.stackIndex as usize];
    (*prev).frame += (*prev).last.get_difference(&now);
    this.stackIndex -= 1;
    if this.stackIndex >= 0 {
        let curr: *mut Scope = this.stack[this.stackIndex as usize];
        (*curr).last = now;
    }
}

#[no_mangle]
pub extern "C" fn Profiler_SetValue(_name: *const libc::c_char, _value: i32) {}

#[no_mangle]
pub unsafe extern "C" fn Profiler_LoopMarker() {
    if !profiling {
        return;
    }
    let mut i: i32 = 0;
    while i < this.scopeList.len() as i32 {
        let scope: &mut Scope = &mut *this.scopeList[i as usize];
        if (*scope).frame as f64 > 0.0f64 {
            (*scope).total = (*scope).total + (*scope).frame;
            let frame: f64 = (*scope).frame as f64;
            (*scope).min = f64::min((*scope).min, frame);
            (*scope).max = f64::max((*scope).max, frame);
            (*scope).count += 1.0f64;
            let d1: f64 = frame - (*scope).mean;
            (*scope).mean += d1 / (*scope).count;
            let d2: f64 = frame - (*scope).mean;
            (*scope).var += d1 * d2;
            (*scope).frame = 0.0;
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn Profiler_Backtrace() {
    if !profiling {
        return;
    }

    info!("PHX Profiler Backtrace:");

    let mut i: i32 = 0;
    while i <= this.stackIndex {
        let index: i32 = this.stackIndex - i;

        info!(
            "  [{index}] {:?}",
            CStr::from_ptr((*this.stack[index as usize]).name)
        );

        i += 1;
    }
}
