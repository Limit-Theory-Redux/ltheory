use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

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
static HANDLER_DEFAULT: LazyLock<HashMap<Signal, SignalHandler>> = LazyLock::new(|| {
    HashMap::from([
        (SIGNAL_INT, signal(SIGNAL_INT, signal_handler)),
        (SIGNAL_ILL, signal(SIGNAL_ILL, signal_handler)),
        (SIGNAL_FPE, signal(SIGNAL_FPE, signal_handler)),
        (SIGNAL_SEGV, signal(SIGNAL_SEGV, signal_handler)),
        (SIGNAL_TERM, signal(SIGNAL_TERM, signal_handler)),
        (SIGNAL_ABRT, signal(SIGNAL_ABRT, signal_handler)),
    ])
});
static HANDLER_TABLE: LazyLock<Mutex<HashMap<Signal, Vec<SignalHandler>>>> = LazyLock::new(|| {
    let m = HashMap::from([
        (SIGNAL_INT, Vec::new()),
        (SIGNAL_ILL, Vec::new()),
        (SIGNAL_FPE, Vec::new()),
        (SIGNAL_SEGV, Vec::new()),
        (SIGNAL_TERM, Vec::new()),
        (SIGNAL_ABRT, Vec::new()),
    ]);
    Mutex::new(m)
});

fn handler_default(signal: Signal) -> SignalHandler {
    *HANDLER_DEFAULT.get(&signal).unwrap()
}

fn signal(signal: Signal, handler: SignalHandler) -> SignalHandler {
    #[allow(unsafe_code)]
    unsafe {
        let ptr = libc::signal(signal, handler as *mut libc::c_void as libc::sighandler_t);
        std::mem::transmute::<libc::sighandler_t, SignalHandler>(ptr)
    }
}

extern "C" fn signal_handler(sig: Signal) {
    warn!("Signal_Handler: Caught {:?}", signal_to_string(sig));

    signal_restore_default();

    /* Call custom handlers. */
    {
        let handler_table = HANDLER_TABLE.lock().unwrap();
        let handlers = handler_table.get(&sig).unwrap();
        for handler in handlers {
            handler(sig);
        }
    }

    #[allow(unsafe_code)] // TODO: refactor
    unsafe {
        if IGNORE_DEFAULT {
            IGNORE_DEFAULT = false;
            return;
        }

        /* Re-raise the signal to let the default handler run. */
        libc::raise(sig);
    }
}

pub fn signal_restore_default() {
    signal(SIGNAL_INT, handler_default(SIGNAL_INT));
    signal(SIGNAL_ILL, handler_default(SIGNAL_ILL));
    signal(SIGNAL_FPE, handler_default(SIGNAL_FPE));
    signal(SIGNAL_SEGV, handler_default(SIGNAL_SEGV));
    signal(SIGNAL_TERM, handler_default(SIGNAL_TERM));
    signal(SIGNAL_ABRT, handler_default(SIGNAL_ABRT));
}

pub fn signal_add_handler(sig: Signal, handler: SignalHandler) {
    let mut handler_table = HANDLER_TABLE.lock().unwrap();
    let handlers = handler_table.get_mut(&sig).unwrap();
    handlers.push(handler);
}

pub fn signal_add_handler_all(handler: SignalHandler) {
    signal_add_handler(SIGNAL_INT, handler);
    signal_add_handler(SIGNAL_ILL, handler);
    signal_add_handler(SIGNAL_FPE, handler);
    signal_add_handler(SIGNAL_SEGV, handler);
    signal_add_handler(SIGNAL_TERM, handler);
    signal_add_handler(SIGNAL_ABRT, handler);
}

pub fn signal_remove_handler(sig: Signal, handler: SignalHandler) {
    let mut handler_table = HANDLER_TABLE.lock().unwrap();
    let handlers = handler_table.get_mut(&sig).unwrap();
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

pub fn signal_remove_handler_all(handler: SignalHandler) {
    signal_remove_handler(SIGNAL_INT, handler);
    signal_remove_handler(SIGNAL_ILL, handler);
    signal_remove_handler(SIGNAL_FPE, handler);
    signal_remove_handler(SIGNAL_SEGV, handler);
    signal_remove_handler(SIGNAL_TERM, handler);
    signal_remove_handler(SIGNAL_ABRT, handler);
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

#[allow(unsafe_code)]
pub unsafe extern "C" fn signal_ignore_default() {
    IGNORE_DEFAULT = true;
}
