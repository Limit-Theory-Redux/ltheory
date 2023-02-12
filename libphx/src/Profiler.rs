use ::libc;
extern "C" {
    pub type HashMap;
    pub type __sFILEX;
    fn memcpy(
        _: *mut libc::c_void,
        _: *const libc::c_void,
        _: libc::c_ulong,
    ) -> *mut libc::c_void;
    fn Fatal(_: cstr, _: ...);
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
    fn realloc(_: *mut libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
    fn qsort(
        __base: *mut libc::c_void,
        __nel: size_t,
        __width: size_t,
        __compar: Option::<
            unsafe extern "C" fn(*const libc::c_void, *const libc::c_void) -> libc::c_int,
        >,
    );
    fn HashMap_Create(keySize: uint32, capacity: uint32) -> *mut HashMap;
    fn HashMap_Free(_: *mut HashMap);
    fn HashMap_GetRaw(_: *mut HashMap, keyHash: uint64) -> *mut libc::c_void;
    fn HashMap_SetRaw(_: *mut HashMap, keyHash: uint64, value: *mut libc::c_void);
    fn sqrt(_: libc::c_double) -> libc::c_double;
    fn Signal_AddHandlerAll(_: SignalHandler);
    fn Signal_RemoveHandlerAll(_: SignalHandler);
    static mut __stdoutp: *mut FILE;
    fn fflush(_: *mut FILE) -> libc::c_int;
    fn printf(_: *const libc::c_char, _: ...) -> libc::c_int;
    fn puts(_: *const libc::c_char) -> libc::c_int;
    fn TimeStamp_Get() -> TimeStamp;
    fn TimeStamp_GetElapsed(start: TimeStamp) -> libc::c_double;
    fn TimeStamp_ToDouble(_: TimeStamp) -> libc::c_double;
}
pub type int32_t = libc::c_int;
pub type uint32_t = libc::c_uint;
pub type uint64_t = libc::c_ulonglong;
pub type __int64_t = libc::c_longlong;
pub type __darwin_size_t = libc::c_ulong;
pub type __darwin_off_t = __int64_t;
pub type size_t = __darwin_size_t;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type uint32 = uint32_t;
pub type uint64 = uint64_t;
pub type TimeStamp = uint64;
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
    pub count: libc::c_double,
    pub mean: libc::c_double,
    pub var: libc::c_double,
    pub min: libc::c_double,
    pub max: libc::c_double,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Profiler {
    pub map: *mut HashMap,
    pub stackIndex: libc::c_int,
    pub stack: [*mut Scope; 128],
    pub scopeList_size: int32,
    pub scopeList_capacity: int32,
    pub scopeList_data: *mut *mut Scope,
    pub start: TimeStamp,
}
#[inline]
unsafe extern "C" fn MemAlloc(mut size: size_t) -> *mut libc::c_void {
    return malloc(size);
}
#[inline]
unsafe extern "C" fn MemFree(mut ptr: *const libc::c_void) {
    free(ptr as *mut libc::c_void);
}
#[inline]
unsafe extern "C" fn MemRealloc(
    mut ptr: *mut libc::c_void,
    mut newSize: size_t,
) -> *mut libc::c_void {
    return realloc(ptr, newSize);
}
#[inline]
unsafe extern "C" fn Max(
    mut a: libc::c_double,
    mut b: libc::c_double,
) -> libc::c_double {
    return if a > b { a } else { b };
}
#[inline]
unsafe extern "C" fn Min(
    mut a: libc::c_double,
    mut b: libc::c_double,
) -> libc::c_double {
    return if a < b { a } else { b };
}
#[inline]
unsafe extern "C" fn Sqrt(mut t: libc::c_double) -> libc::c_double {
    return sqrt(t);
}
#[no_mangle]
pub static mut Signal_Int: Signal = 0;
#[no_mangle]
pub static mut Signal_Ill: Signal = 0;
#[no_mangle]
pub static mut Signal_Fpe: Signal = 0;
#[no_mangle]
pub static mut Signal_Segv: Signal = 0;
#[no_mangle]
pub static mut Signal_Term: Signal = 0;
#[no_mangle]
pub static mut Signal_Abrt: Signal = 0;
#[inline]
unsafe extern "C" fn StrDup(mut s: cstr) -> cstr {
    if s.is_null() {
        return 0 as cstr;
    }
    let mut len: size_t = (StrLen(s)).wrapping_add(1 as libc::c_int as libc::c_ulong);
    let mut buf: *mut libc::c_char = StrAlloc(len);
    memcpy(buf as *mut libc::c_void, s as *const libc::c_void, len);
    return buf as cstr;
}
#[inline]
unsafe extern "C" fn StrLen(mut s: cstr) -> size_t {
    if s.is_null() {
        return 0 as libc::c_int as size_t;
    }
    let mut begin: cstr = s;
    while *s != 0 {
        s = s.offset(1);
    }
    return s.offset_from(begin) as libc::c_long as size_t;
}
#[inline]
unsafe extern "C" fn StrAlloc(mut len: size_t) -> *mut libc::c_char {
    return malloc(len) as *mut libc::c_char;
}
#[inline]
unsafe extern "C" fn StrFree(mut s: cstr) {
    free(s as *mut libc::c_void);
}
static mut self_0: Profiler = Profiler {
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
        ::core::mem::size_of::<Scope>() as libc::c_ulong,
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
    if (self_0.scopeList_capacity == self_0.scopeList_size) as libc::c_int
        as libc::c_long != 0
    {
        self_0
            .scopeList_capacity = if self_0.scopeList_capacity != 0 {
            self_0.scopeList_capacity * 2 as libc::c_int
        } else {
            1 as libc::c_int
        };
        let mut elemSize: size_t = ::core::mem::size_of::<*mut Scope>() as libc::c_ulong;
        let mut pData: *mut *mut libc::c_void = &mut self_0.scopeList_data
            as *mut *mut *mut Scope as *mut *mut libc::c_void;
        *pData = MemRealloc(
            self_0.scopeList_data as *mut libc::c_void,
            (self_0.scopeList_capacity as libc::c_ulong).wrapping_mul(elemSize),
        );
    }
    let fresh0 = self_0.scopeList_size;
    self_0.scopeList_size = self_0.scopeList_size + 1;
    let ref mut fresh1 = *(self_0.scopeList_data).offset(fresh0 as isize);
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
    let mut scope: *mut Scope = HashMap_GetRaw(self_0.map, name as size_t as uint64)
        as *mut Scope;
    if !scope.is_null() {
        return scope;
    }
    scope = Scope_Create(name);
    HashMap_SetRaw(self_0.map, name as size_t as uint64, scope as *mut libc::c_void);
    return scope;
}
unsafe extern "C" fn Profiler_SignalHandler(mut s: Signal) {
    Profiler_Backtrace();
}
#[no_mangle]
pub unsafe extern "C" fn Profiler_Enable() {
    profiling = 1 as libc::c_int != 0;
    self_0
        .map = HashMap_Create(
        ::core::mem::size_of::<*mut libc::c_void>() as libc::c_ulong as uint32,
        (2 as libc::c_int * 1024 as libc::c_int) as uint32,
    );
    self_0.scopeList_capacity = 0 as libc::c_int;
    self_0.scopeList_size = 0 as libc::c_int;
    self_0.scopeList_data = 0 as *mut *mut Scope;
    if (self_0.scopeList_capacity < 1024 as libc::c_int) as libc::c_int as libc::c_long
        != 0
    {
        self_0.scopeList_capacity = 1024 as libc::c_int;
        let mut elemSize: size_t = ::core::mem::size_of::<*mut Scope>() as libc::c_ulong;
        let mut pData: *mut *mut libc::c_void = &mut self_0.scopeList_data
            as *mut *mut *mut Scope as *mut *mut libc::c_void;
        *pData = MemRealloc(
            self_0.scopeList_data as *mut libc::c_void,
            (self_0.scopeList_capacity as libc::c_ulong).wrapping_mul(elemSize),
        );
    }
    self_0.stackIndex = -(1 as libc::c_int);
    self_0.start = TimeStamp_Get();
    Profiler_Begin(b"[Root]\0" as *const u8 as *const libc::c_char);
    Signal_AddHandlerAll(
        Some(Profiler_SignalHandler as unsafe extern "C" fn(Signal) -> ()),
    );
}
#[no_mangle]
pub unsafe extern "C" fn Profiler_Disable() {
    if self_0.stackIndex != 0 as libc::c_int {
        Fatal(
            b"Profiler_Disable: Cannot stop profiler from within a profiled section\0"
                as *const u8 as *const libc::c_char,
        );
    }
    Profiler_End();
    let mut total: libc::c_double = TimeStamp_GetElapsed(self_0.start);
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < self_0.scopeList_size {
        let mut scope: *mut Scope = *(self_0.scopeList_data).offset(i as isize);
        (*scope).var /= (*scope).count - 1.0f64;
        (*scope).var = Sqrt((*scope).var);
        i += 1;
    }
    qsort(
        self_0.scopeList_data as *mut libc::c_void,
        self_0.scopeList_size as size_t,
        ::core::mem::size_of::<*mut Scope>() as libc::c_ulong,
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
    let mut cumulative: libc::c_double = 0 as libc::c_int as libc::c_double;
    let mut i_0: libc::c_int = 0 as libc::c_int;
    while i_0 < self_0.scopeList_size {
        let mut scope_0: *mut Scope = *(self_0.scopeList_data).offset(i_0 as isize);
        let mut scopeTotal: libc::c_double = TimeStamp_ToDouble((*scope_0).total);
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
    fflush(__stdoutp);
    let mut i_1: libc::c_int = 0 as libc::c_int;
    while i_1 < self_0.scopeList_size {
        Scope_Free(*(self_0.scopeList_data).offset(i_1 as isize));
        i_1 += 1;
    }
    MemFree(self_0.scopeList_data as *const libc::c_void);
    HashMap_Free(self_0.map);
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
    if self_0.stackIndex + 1 as libc::c_int >= 128 as libc::c_int {
        Profiler_Backtrace();
        Fatal(
            b"Profiler_Begin: Maximum stack depth exceeded\0" as *const u8
                as *const libc::c_char,
        );
    }
    let mut now: TimeStamp = TimeStamp_Get();
    if self_0.stackIndex >= 0 as libc::c_int {
        let mut prev: *mut Scope = self_0.stack[self_0.stackIndex as usize];
        (*prev)
            .frame = ((*prev).frame as libc::c_ulonglong)
            .wrapping_add(now.wrapping_sub((*prev).last)) as TimeStamp as TimeStamp;
        (*prev).last = now;
    }
    self_0.stackIndex += 1;
    let mut curr: *mut Scope = Profiler_GetScope(name);
    self_0.stack[self_0.stackIndex as usize] = curr;
    (*curr).last = now;
}
#[no_mangle]
pub unsafe extern "C" fn Profiler_End() {
    if !profiling {
        return;
    }
    if self_0.stackIndex < 0 as libc::c_int {
        Profiler_Backtrace();
        Fatal(
            b"Profiler_End: Attempting to pop an empty stack\0" as *const u8
                as *const libc::c_char,
        );
    }
    let mut now: TimeStamp = TimeStamp_Get();
    let mut prev: *mut Scope = self_0.stack[self_0.stackIndex as usize];
    (*prev)
        .frame = ((*prev).frame as libc::c_ulonglong)
        .wrapping_add(now.wrapping_sub((*prev).last)) as TimeStamp as TimeStamp;
    self_0.stackIndex -= 1;
    if self_0.stackIndex >= 0 as libc::c_int {
        let mut curr: *mut Scope = self_0.stack[self_0.stackIndex as usize];
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
    while i < self_0.scopeList_size {
        let mut scope: *mut Scope = *(self_0.scopeList_data).offset(i as isize);
        if (*scope).frame as libc::c_double > 0.0f64 {
            (*scope)
                .total = ((*scope).total as libc::c_ulonglong)
                .wrapping_add((*scope).frame) as TimeStamp as TimeStamp;
            let mut frame: libc::c_double = TimeStamp_ToDouble((*scope).frame);
            (*scope).min = Min((*scope).min, frame);
            (*scope).max = Max((*scope).max, frame);
            (*scope).count += 1.0f64;
            let mut d1: libc::c_double = frame - (*scope).mean;
            (*scope).mean += d1 / (*scope).count;
            let mut d2: libc::c_double = frame - (*scope).mean;
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
    while i <= self_0.stackIndex {
        let mut index: libc::c_int = self_0.stackIndex - i;
        printf(
            b"  [%i] %s\n\0" as *const u8 as *const libc::c_char,
            index,
            (*self_0.stack[index as usize]).name,
        );
        i += 1;
    }
    fflush(__stdoutp);
}
