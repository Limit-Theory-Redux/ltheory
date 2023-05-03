use crate::phx::internal::Memory::*;
use crate::phx::Common::*;
use crate::phx::Math::Vec3;

extern "C" {
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
pub static Signal_Ill: Signal = 4;

#[no_mangle]
pub static Signal_Fpe: Signal = 8;

#[no_mangle]
pub static Signal_Segv: Signal = 11;

#[no_mangle]
pub static Signal_Term: Signal = 15;

#[no_mangle]
pub static Signal_Abrt: Signal = 6;

#[no_mangle]
pub static Signal_Int: Signal = 2;

static mut ignoreDefault: bool = false;

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

unsafe extern "C" fn Signal_Handler(sig: Signal) {
    CWarn!("Signal_Handler: Caught %s", Signal_ToString(sig));
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
        ignoreDefault = false;
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
pub unsafe extern "C" fn Signal_AddHandler(sig: Signal, fn_0: SignalHandler) {
    let e = MemNew!(HandlerElem);
    (*e).next = handlerTable[sig as usize];
    (*e).fn_0 = fn_0;
    // handlerTable[sig as usize] = e;
}

#[no_mangle]
pub unsafe extern "C" fn Signal_AddHandlerAll(fn_0: SignalHandler) {
    Signal_AddHandler(Signal_Int, fn_0);
    Signal_AddHandler(Signal_Ill, fn_0);
    Signal_AddHandler(Signal_Fpe, fn_0);
    Signal_AddHandler(Signal_Segv, fn_0);
    Signal_AddHandler(Signal_Term, fn_0);
    Signal_AddHandler(Signal_Abrt, fn_0);
}

#[no_mangle]
pub unsafe extern "C" fn Signal_RemoveHandler(sig: Signal, fn_0: SignalHandler) {
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
    CFatal!("Signal_RemoveHandler: No such handler installed");
}

#[no_mangle]
pub unsafe extern "C" fn Signal_RemoveHandlerAll(fn_0: SignalHandler) {
    Signal_RemoveHandler(Signal_Int, fn_0);
    Signal_RemoveHandler(Signal_Ill, fn_0);
    Signal_RemoveHandler(Signal_Fpe, fn_0);
    Signal_RemoveHandler(Signal_Segv, fn_0);
    Signal_RemoveHandler(Signal_Term, fn_0);
    Signal_RemoveHandler(Signal_Abrt, fn_0);
}

#[no_mangle]
pub extern "C" fn Signal_ToString(this: Signal) -> *const libc::c_char {
    match this {
        2 => return c_str!("Interrupt"),
        4 => return c_str!("Illegal Instruction"),
        8 => return c_str!("FP Exception"),
        11 => return c_str!("Memory Access Violation"),
        15 => return c_str!("Terminate"),
        6 => return c_str!("Abort"),
        _ => {}
    }
    c_str!("<unknown signal>")
}

#[no_mangle]
pub unsafe extern "C" fn Signal_IgnoreDefault() {
    ignoreDefault = true;
}
