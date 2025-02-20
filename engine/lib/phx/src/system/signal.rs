use std::collections::HashMap;
use std::ffi::CStr;

use internal::static_string;

use crate::logging::warn;

pub type Signal = i32;
pub type SignalHandler = extern "C" fn(Signal) -> ();

pub const SIGNAL_INT: Signal = libc::SIGINT;
pub const SIGNAL_ILL: Signal = libc::SIGILL;
pub const SIGNAL_FPE: Signal = libc::SIGFPE;
pub const SIGNAL_SEGV: Signal = libc::SIGSEGV;
pub const SIGNAL_TERM: Signal = libc::SIGTERM;
pub const SIGNAL_ABRT: Signal = libc::SIGABRT;

static mut IGNORE_DEFAULT: bool = false;
static mut HANDLER_DEFAULT: Option<HashMap<Signal, SignalHandler>> = None;
static mut HANDLER_TABLE: Option<HashMap<Signal, Vec<SignalHandler>>> = None;

fn HandlerDefault(signal: Signal) -> SignalHandler {
    unsafe { *HANDLER_DEFAULT.as_ref().unwrap().get(&signal).unwrap() }
}

fn HandlerTable<'a>(signal: Signal) -> &'a mut Vec<SignalHandler> {
    unsafe { HANDLER_TABLE.as_mut().unwrap().get_mut(&signal).unwrap() }
}

fn Signal(signal: Signal, handler: SignalHandler) -> SignalHandler {
    unsafe {
        let ptr = libc::signal(signal, handler as *mut libc::c_void as libc::sighandler_t);
        std::mem::transmute::<libc::sighandler_t, SignalHandler>(ptr)
    }
}

extern "C" fn Signal_Handler(sig: Signal) {
    unsafe {
        warn!(
            "Signal_Handler: Caught {:?}",
            CStr::from_ptr(Signal_ToString(sig))
        );

        /* Re-install default handlers. */
        Signal(SIGNAL_INT, HandlerDefault(SIGNAL_INT));
        Signal(SIGNAL_ILL, HandlerDefault(SIGNAL_ILL));
        Signal(SIGNAL_FPE, HandlerDefault(SIGNAL_FPE));
        Signal(SIGNAL_SEGV, HandlerDefault(SIGNAL_SEGV));
        Signal(SIGNAL_TERM, HandlerDefault(SIGNAL_TERM));
        Signal(SIGNAL_ABRT, HandlerDefault(SIGNAL_ABRT));

        /* Call custom handlers. */
        for handler in HandlerTable(sig).iter() {
            handler(sig);
        }
        if IGNORE_DEFAULT {
            IGNORE_DEFAULT = false;
            return;
        }

        /* Re-raise the signal to let the default handler run. */
        libc::raise(sig);
    }
}

#[no_mangle]
pub unsafe extern "C" fn Signal_Init() {
    HANDLER_DEFAULT = Some(HashMap::from([
        (SIGNAL_INT, Signal(SIGNAL_INT, Signal_Handler)),
        (SIGNAL_ILL, Signal(SIGNAL_ILL, Signal_Handler)),
        (SIGNAL_FPE, Signal(SIGNAL_FPE, Signal_Handler)),
        (SIGNAL_SEGV, Signal(SIGNAL_SEGV, Signal_Handler)),
        (SIGNAL_TERM, Signal(SIGNAL_TERM, Signal_Handler)),
        (SIGNAL_ABRT, Signal(SIGNAL_ABRT, Signal_Handler)),
    ]));
    HANDLER_TABLE = Some(HashMap::from([
        (SIGNAL_INT, Vec::new()),
        (SIGNAL_ILL, Vec::new()),
        (SIGNAL_FPE, Vec::new()),
        (SIGNAL_SEGV, Vec::new()),
        (SIGNAL_TERM, Vec::new()),
        (SIGNAL_ABRT, Vec::new()),
    ]));
}

#[no_mangle]
pub unsafe extern "C" fn Signal_Free() {
    Signal(SIGNAL_INT, HandlerDefault(SIGNAL_INT));
    Signal(SIGNAL_ILL, HandlerDefault(SIGNAL_ILL));
    Signal(SIGNAL_FPE, HandlerDefault(SIGNAL_FPE));
    Signal(SIGNAL_SEGV, HandlerDefault(SIGNAL_SEGV));
    Signal(SIGNAL_TERM, HandlerDefault(SIGNAL_TERM));
    Signal(SIGNAL_ABRT, HandlerDefault(SIGNAL_ABRT));
}

#[no_mangle]
pub unsafe extern "C" fn Signal_AddHandler(sig: Signal, handler: SignalHandler) {
    HandlerTable(sig).push(handler);
}

#[no_mangle]
pub unsafe extern "C" fn Signal_AddHandlerAll(handler: SignalHandler) {
    Signal_AddHandler(SIGNAL_INT, handler);
    Signal_AddHandler(SIGNAL_ILL, handler);
    Signal_AddHandler(SIGNAL_FPE, handler);
    Signal_AddHandler(SIGNAL_SEGV, handler);
    Signal_AddHandler(SIGNAL_TERM, handler);
    Signal_AddHandler(SIGNAL_ABRT, handler);
}

#[no_mangle]
pub unsafe extern "C" fn Signal_RemoveHandler(sig: Signal, handler: SignalHandler) {
    let handlers = HandlerTable(sig);
    if let Some(pos) = handlers
        .iter()
        .position(|f| std::ptr::fn_addr_eq(*f, handler))
    {
        handlers.remove(pos);
    } else {
        panic!(
            "Signal_RemoveHandler: No such handler installed for signal {}",
            signal_to_string(sig)
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn Signal_RemoveHandlerAll(handler: SignalHandler) {
    Signal_RemoveHandler(SIGNAL_INT, handler);
    Signal_RemoveHandler(SIGNAL_ILL, handler);
    Signal_RemoveHandler(SIGNAL_FPE, handler);
    Signal_RemoveHandler(SIGNAL_SEGV, handler);
    Signal_RemoveHandler(SIGNAL_TERM, handler);
    Signal_RemoveHandler(SIGNAL_ABRT, handler);
}

#[no_mangle]
pub extern "C" fn Signal_ToString(this: Signal) -> *const libc::c_char {
    static_string!(signal_to_string(this))
}

pub fn signal_to_string(this: Signal) -> String {
    match this {
        SIGNAL_INT => "Interrupt",
        SIGNAL_ILL => "Illegal Instruction",
        SIGNAL_FPE => "FP Exception",
        SIGNAL_SEGV => "Memory Access Violation",
        SIGNAL_TERM => "Terminate",
        SIGNAL_ABRT => "Abort",
        _ => "<unknown signal>",
    }
    .into()
}

#[no_mangle]
pub unsafe extern "C" fn Signal_IgnoreDefault() {
    IGNORE_DEFAULT = true;
}
