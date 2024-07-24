use strum_macros::EnumIter;

#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter)]
pub enum FrameStage {
    // Before physics update
    PreSim,
    // Physics update
    Sim,
    // After physics update
    PostSim,
    // Before frame render
    #[default]
    PreRender,
    // Frame render
    Render,
    // After frame render
    PostRender,
    // Before input handling
    PreInput,
    // Input handling
    Input,
    // After input handling
    PostInput,
}