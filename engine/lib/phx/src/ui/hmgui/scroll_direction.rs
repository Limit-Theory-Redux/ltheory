/// Define a scroll area allowed scrolling directions.
#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum ScrollDirection {
    /// Horizontal and vertical scrolling.
    #[default]
    All,
    /// Only horizontal scrolling.
    Horizontal,
    /// Only vertical scrolling.
    Vertical,
}
