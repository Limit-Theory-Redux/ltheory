#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FocusStyle {
    None,
    Fill,
    Outline,
    Underline,
}

#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FocusType {
    Mouse,
    Scroll,
}
