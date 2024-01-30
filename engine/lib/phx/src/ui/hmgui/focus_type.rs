// TODO: do we really need this mouse/scroll 'focus' separation?

#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FocusType {
    Mouse,
    Scroll,
}
