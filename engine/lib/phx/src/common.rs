macro_rules! c_str {
    ($s:expr) => {{
        concat!($s, "\0").as_ptr() as *const i8
    }};
}
pub(crate) use c_str;

macro_rules! Fatal {
    ($fmt:expr) => (
        panic!($fmt)
    );
    ($fmt:expr, $($args:expr),* $(,)?) => (
        panic!($fmt, $($args),*)
    );
}
pub(crate) use Fatal;
