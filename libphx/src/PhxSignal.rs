use crate::internal::Memory::*;
use glam::Vec3;
use libc;

extern "C" {
    fn Fatal(_: *const libc::c_char, _: ...);
    fn Warn(_: *const libc::c_char, _: ...);
    fn signal(
        _: i32,
        _: Option<unsafe extern "C" fn(i32) -> ()>,
    ) -> Option<unsafe extern "C" fn(i32) -> ()>;
}

pub type Signal = i32;
pub type SignalHandler = Option<unsafe extern "C" fn(Signal) -> ()>;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HandlerElem {
    pub next: *mut HandlerElem,
    pub fn_0: SignalHandler,
}

#[no_mangle]
pub static Signal_Ill: Signal = 4 as i32;

#[no_mangle]
pub static Signal_Fpe: Signal = 8 as i32;

#[no_mangle]
pub static Signal_Segv: Signal = 11 as i32;

#[no_mangle]
pub static Signal_Term: Signal = 15 as i32;

#[no_mangle]
pub static Signal_Abrt: Signal = 6 as i32;

#[no_mangle]
pub static Signal_Int: Signal = 2 as i32;

static mut ignoreDefault: bool = 0 as i32 != 0;
static mut handlerDefault: [SignalHandler; 32] = [
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
];
static mut handlerTable: [*mut HandlerElem; 32] = [
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
];
unsafe extern "C" fn Signal_Handler(mut sig: Signal) {
    Warn(
        b"Signal_Handler: Caught %s\0" as *const u8 as *const libc::c_char,
        Signal_ToString(sig),
    );
    signal(Signal_Int, handlerDefault[Signal_Int as usize]);
    signal(Signal_Ill, handlerDefault[Signal_Ill as usize]);
    signal(Signal_Fpe, handlerDefault[Signal_Fpe as usize]);
    signal(Signal_Segv, handlerDefault[Signal_Segv as usize]);
    signal(Signal_Term, handlerDefault[Signal_Term as usize]);
    signal(Signal_Abrt, handlerDefault[Signal_Abrt as usize]);
    let mut e: *mut HandlerElem = handlerTable[sig as usize];
    while !e.is_null() {
        ((*e).fn_0).expect("non-null function pointer")(sig);
        e = (*e).next;
    }
    if ignoreDefault {
        ignoreDefault = 0 as i32 != 0;
        return;
    }
    libc::raise(sig);
}

#[no_mangle]
pub unsafe extern "C" fn Signal_Init() {
    handlerDefault[Signal_Int as usize] = signal(
        Signal_Int,
        Some(Signal_Handler as unsafe extern "C" fn(Signal) -> ()),
    );
    handlerDefault[Signal_Ill as usize] = signal(
        Signal_Ill,
        Some(Signal_Handler as unsafe extern "C" fn(Signal) -> ()),
    );
    handlerDefault[Signal_Fpe as usize] = signal(
        Signal_Fpe,
        Some(Signal_Handler as unsafe extern "C" fn(Signal) -> ()),
    );
    handlerDefault[Signal_Segv as usize] = signal(
        Signal_Segv,
        Some(Signal_Handler as unsafe extern "C" fn(Signal) -> ()),
    );
    handlerDefault[Signal_Term as usize] = signal(
        Signal_Term,
        Some(Signal_Handler as unsafe extern "C" fn(Signal) -> ()),
    );
    handlerDefault[Signal_Abrt as usize] = signal(
        Signal_Abrt,
        Some(Signal_Handler as unsafe extern "C" fn(Signal) -> ()),
    );
}

#[no_mangle]
pub unsafe extern "C" fn Signal_Free() {
    signal(Signal_Int, handlerDefault[Signal_Int as usize]);
    signal(Signal_Ill, handlerDefault[Signal_Ill as usize]);
    signal(Signal_Fpe, handlerDefault[Signal_Fpe as usize]);
    signal(Signal_Segv, handlerDefault[Signal_Segv as usize]);
    signal(Signal_Term, handlerDefault[Signal_Term as usize]);
    signal(Signal_Abrt, handlerDefault[Signal_Abrt as usize]);
}

#[no_mangle]
pub unsafe extern "C" fn Signal_AddHandler(mut sig: Signal, mut fn_0: SignalHandler) {
    let mut e: *mut HandlerElem =
        MemAlloc(::core::mem::size_of::<HandlerElem>() as usize) as *mut HandlerElem;
    (*e).next = handlerTable[sig as usize];
    (*e).fn_0 = fn_0;
    // handlerTable[sig as usize] = e;
}

#[no_mangle]
pub unsafe extern "C" fn Signal_AddHandlerAll(mut fn_0: SignalHandler) {
    Signal_AddHandler(Signal_Int, fn_0);
    Signal_AddHandler(Signal_Ill, fn_0);
    Signal_AddHandler(Signal_Fpe, fn_0);
    Signal_AddHandler(Signal_Segv, fn_0);
    Signal_AddHandler(Signal_Term, fn_0);
    Signal_AddHandler(Signal_Abrt, fn_0);
}

#[no_mangle]
pub unsafe extern "C" fn Signal_RemoveHandler(mut sig: Signal, mut fn_0: SignalHandler) {
    let mut prev: *mut *mut HandlerElem =
        &mut *handlerTable.as_mut_ptr().offset(sig as isize) as *mut *mut HandlerElem;
    let mut curr: *mut HandlerElem = handlerTable[sig as usize];
    while !curr.is_null() {
        if (*curr).fn_0 == fn_0 {
            *prev = (*curr).next;
            return;
        }
        prev = &mut (*curr).next;
        curr = (*curr).next;
    }
    Fatal(b"Signal_RemoveHandler: No such handler installed\0" as *const u8 as *const libc::c_char);
}

#[no_mangle]
pub unsafe extern "C" fn Signal_RemoveHandlerAll(mut fn_0: SignalHandler) {
    Signal_RemoveHandler(Signal_Int, fn_0);
    Signal_RemoveHandler(Signal_Ill, fn_0);
    Signal_RemoveHandler(Signal_Fpe, fn_0);
    Signal_RemoveHandler(Signal_Segv, fn_0);
    Signal_RemoveHandler(Signal_Term, fn_0);
    Signal_RemoveHandler(Signal_Abrt, fn_0);
}

#[no_mangle]
pub unsafe extern "C" fn Signal_ToString(mut this: Signal) -> *const libc::c_char {
    match this {
        2 => return b"Interrupt\0" as *const u8 as *const libc::c_char,
        4 => return b"Illegal Instruction\0" as *const u8 as *const libc::c_char,
        8 => return b"FP Exception\0" as *const u8 as *const libc::c_char,
        11 => return b"Memory Access Violation\0" as *const u8 as *const libc::c_char,
        15 => return b"Terminate\0" as *const u8 as *const libc::c_char,
        6 => return b"Abort\0" as *const u8 as *const libc::c_char,
        _ => {}
    }
    return b"<unknown signal>\0" as *const u8 as *const libc::c_char;
}

#[no_mangle]
pub unsafe extern "C" fn Signal_IgnoreDefault() {
    ignoreDefault = 1 as i32 != 0;
}
