#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub enum FocusStyle {
    #[default]
    None,
    Fill,
    #[allow(dead_code)]
    Outline,
    Underline,
}

#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FocusType {
    Mouse,
    Scroll,
}
