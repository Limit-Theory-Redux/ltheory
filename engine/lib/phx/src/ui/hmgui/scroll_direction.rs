#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum ScrollDirection {
    #[default]
    All,
    Horizontal,
    Vertical,
}
