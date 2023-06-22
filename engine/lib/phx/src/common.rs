macro_rules! c_str {
    ($s:expr) => {{
        concat!($s, "\0").as_ptr() as *const i8
    }};
}
pub(crate) use c_str;

macro_rules! Fatal {
    ($fmt:expr) => (
        { tracing::error!($fmt); std::process::exit(1); }
    );
    ($fmt:expr, $($args:expr),* $(,)?) => (
        { tracing::error!($fmt, $($args),*); std::process::exit(1); }
    );
}
pub(crate) use Fatal;
