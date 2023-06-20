use tracing::warn;

use crate::common::*;
use crate::internal::*;
use crate::*;

use std::collections::HashMap;

pub type Signal = i32;
pub type SignalHandler = extern "C" fn(Signal) -> ();

pub const Signal_Int: Signal = libc::SIGINT;
pub const Signal_Ill: Signal = libc::SIGILL;
pub const Signal_Fpe: Signal = libc::SIGFPE;
pub const Signal_Segv: Signal = libc::SIGSEGV;
pub const Signal_Term: Signal = libc::SIGTERM;
pub const Signal_Abrt: Signal = libc::SIGABRT;

static mut ignoreDefault: bool = false;
static mut handlerDefault: Option<HashMap<Signal, SignalHandler>> = None;
static mut handlerTable: Option<HashMap<Signal, Vec<SignalHandler>>> = None;

fn HandlerDefault(signal: Signal) -> SignalHandler {
    unsafe { *handlerDefault.as_ref().unwrap().get(&signal).unwrap() }
}

fn HandlerTable<'a>(signal: Signal) -> &'a mut Vec<SignalHandler> {
    unsafe { handlerTable.as_mut().unwrap().get_mut(&signal).unwrap() }
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
        Signal(Signal_Int, HandlerDefault(Signal_Int));
        Signal(Signal_Ill, HandlerDefault(Signal_Ill));
        Signal(Signal_Fpe, HandlerDefault(Signal_Fpe));
        Signal(Signal_Segv, HandlerDefault(Signal_Segv));
        Signal(Signal_Term, HandlerDefault(Signal_Term));
        Signal(Signal_Abrt, HandlerDefault(Signal_Abrt));

        /* Call custom handlers. */
        for handler in HandlerTable(sig).iter() {
            handler(sig);
        }
        if ignoreDefault {
            ignoreDefault = false;
            return;
        }

        /* Re-raise the signal to let the default handler run. */
        libc::raise(sig);
    }
}

#[no_mangle]
pub unsafe extern "C" fn Signal_Init() {
    handlerDefault = Some(HashMap::from([
        (Signal_Int, Signal(Signal_Int, Signal_Handler)),
        (Signal_Ill, Signal(Signal_Ill, Signal_Handler)),
        (Signal_Fpe, Signal(Signal_Fpe, Signal_Handler)),
        (Signal_Segv, Signal(Signal_Segv, Signal_Handler)),
        (Signal_Term, Signal(Signal_Term, Signal_Handler)),
        (Signal_Abrt, Signal(Signal_Abrt, Signal_Handler)),
    ]));
    handlerTable = Some(HashMap::from([
        (Signal_Int, Vec::new()),
        (Signal_Ill, Vec::new()),
        (Signal_Fpe, Vec::new()),
        (Signal_Segv, Vec::new()),
        (Signal_Term, Vec::new()),
        (Signal_Abrt, Vec::new()),
    ]));
}

#[no_mangle]
pub unsafe extern "C" fn Signal_Free() {
    Signal(Signal_Int, HandlerDefault(Signal_Int));
    Signal(Signal_Ill, HandlerDefault(Signal_Ill));
    Signal(Signal_Fpe, HandlerDefault(Signal_Fpe));
    Signal(Signal_Segv, HandlerDefault(Signal_Segv));
    Signal(Signal_Term, HandlerDefault(Signal_Term));
    Signal(Signal_Abrt, HandlerDefault(Signal_Abrt));
}

#[no_mangle]
pub unsafe extern "C" fn Signal_AddHandler(sig: Signal, handler: SignalHandler) {
    HandlerTable(sig).push(handler);
}

#[no_mangle]
pub unsafe extern "C" fn Signal_AddHandlerAll(handler: SignalHandler) {
    Signal_AddHandler(Signal_Int, handler);
    Signal_AddHandler(Signal_Ill, handler);
    Signal_AddHandler(Signal_Fpe, handler);
    Signal_AddHandler(Signal_Segv, handler);
    Signal_AddHandler(Signal_Term, handler);
    Signal_AddHandler(Signal_Abrt, handler);
}

#[no_mangle]
pub unsafe extern "C" fn Signal_RemoveHandler(sig: Signal, handler: SignalHandler) {
    let handlers = HandlerTable(sig);
    if let Some(pos) = handlers.iter().position(|f| *f == handler) {
        handlers.remove(pos);
    } else {
        Fatal!(
            "Signal_RemoveHandler: No such handler installed for signal {}",
            signal_to_string(sig)
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn Signal_RemoveHandlerAll(handler: SignalHandler) {
    Signal_RemoveHandler(Signal_Int, handler);
    Signal_RemoveHandler(Signal_Ill, handler);
    Signal_RemoveHandler(Signal_Fpe, handler);
    Signal_RemoveHandler(Signal_Segv, handler);
    Signal_RemoveHandler(Signal_Term, handler);
    Signal_RemoveHandler(Signal_Abrt, handler);
}

#[no_mangle]
pub extern "C" fn Signal_ToString(this: Signal) -> *const libc::c_char {
    static_string!(signal_to_string(this))
}

pub fn signal_to_string(this: Signal) -> String {
    match this {
        Signal_Int => "Interrupt",
        Signal_Ill => "Illegal Instruction",
        Signal_Fpe => "FP Exception",
        Signal_Segv => "Memory Access Violation",
        Signal_Term => "Terminate",
        Signal_Abrt => "Abort",
        _ => "<unknown signal>",
    }
    .into()
}

#[no_mangle]
pub unsafe extern "C" fn Signal_IgnoreDefault() {
    ignoreDefault = true;
}
