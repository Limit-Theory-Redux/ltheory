#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RendererState {
    Started,
    AlreadyRunning,
    Failed,
}
