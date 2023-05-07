macro_rules! unwrap_or_return {
    ($value: ident) => {
        match $value {
            Some(v) => v,
            _ => return,
        }
    };
    ($value: ident, $return: expr) => {
        match $value {
            Some(v) => v,
            _ => return $return,
        }
    };
}

macro_rules! c_str {
    ($s:expr) => {{
        concat!($s, "\0").as_ptr() as *const i8
    }};
}
pub(crate) use c_str;

macro_rules! Fatal {
    ($fmt:expr) => (
        println!(fmt);
        libc::abort()
    );
    ($fmt:expr, $($args:expr),* $(,)?) => (
        println!($fmt, $($args),*);
        libc::abort()
    );
}
pub(crate) use Fatal;

macro_rules! CFatal {
    ($fmt:expr) => (
        unsafe { common_impl::Fatal(c_str!($fmt)) }
    );
    ($fmt:expr, $($args:expr),* $(,)?) => (
        unsafe { common_impl::Fatal(c_str!($fmt), $($args),*) }
    );
}
pub(crate) use CFatal;

macro_rules! Warn {
    ($fmt:expr) => (
        println!($fmt)
    );
    ($fmt:expr, $($args:expr),* $(,)?) => (
        println!($fmt, $($args),*)
    );
}
pub(crate) use Warn;

macro_rules! CWarn {
    ($fmt:expr) => (
        unsafe { common_impl::Warn(c_str!($fmt)) }
    );
    ($fmt:expr, $($args:expr),* $(,)?) => (
        unsafe { common_impl::Warn(c_str!($fmt), $($args),*) }
    );
}
pub(crate) use CWarn;

macro_rules! CPrintf {
    ($fmt:expr) => (
        unsafe { common_impl::Printf(c_str!($fmt)) }
    );
    ($fmt:expr, $($args:expr),* $(,)?) => (
        unsafe { common_impl::Printf(c_str!($fmt), $($args),*) }
    );
}
pub(crate) use CPrintf;

pub mod common_impl {
    /*
    #ifdef WINDOWS
      #include <windows.h>

      static void Fatal_Output (cstr message) {
        /* TODO : We want this in Visual Studio where stdout/err don't go to the
         *       Console window, but not in VS Code where it does. */
        //OutputDebugStringA(message);
        fprintf(stderr, "%s\n", message);

        #ifndef BUILD_DISTRIBUTABLE
          int result = MessageBox(0,
            StrAdd(message, "\n\nBreak into debugger? (Cancel to continue execution)"),
            "PHX Engine: Fatal Error", MB_YESNOCANCEL | MB_ICONWARNING | MB_DEFBUTTON2);
          switch(result) {
            case IDYES:
              Profiler_Backtrace();
              Lua_Backtrace();
              DEBUG_BREAK;
              exit(1);
              break;

            case IDNO:
              abort();
              break;

            case IDCANCEL:
              break;
          }
        #else
          abort();
        #endif
      }
    #else
      static void Fatal_Output (cstr message) {
        puts(message);
        abort();
      }
    #endif
    */
    unsafe fn FatalOutput(message: String) -> ! {
        println!("{}", message);
        libc::abort()
    }

    #[no_mangle]
    pub unsafe extern "C" fn Fatal(format: *const libc::c_char, mut args: ...) -> ! {
        let mut s = String::new();
        let _ = printf_compat::format(
            format,
            args.as_va_list(),
            printf_compat::output::fmt_write(&mut s),
        );
        FatalOutput(s)
    }

    #[no_mangle]
    pub unsafe extern "C" fn Warn(format: *const libc::c_char, mut args: ...) {
        let mut s = String::new();
        let _ = printf_compat::format(
            format,
            args.as_va_list(),
            printf_compat::output::fmt_write(&mut s),
        );
        println!("{}", s);
    }

    #[no_mangle]
    pub unsafe extern "C" fn Printf(format: *const libc::c_char, mut args: ...) {
        let mut s = String::new();
        let _ = printf_compat::format(
            format,
            args.as_va_list(),
            printf_compat::output::fmt_write(&mut s),
        );
        print!("{}", s);
    }
}
