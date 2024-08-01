use strum_macros::EnumIter;

#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter)]
pub enum FrameStage {
    // Before physics update
    PreSim,
    // Physics update
    Sim,
    // After physics update
    PostSim,
    // Before frame render
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

impl FrameStage {
    pub fn first() -> Self {
        Self::PreSim
    }

    pub fn last() -> Self {
        Self::PostInput
    }

    pub fn next(&self) -> Option<Self> {
        match self {
            Self::PreSim => Some(Self::Sim),
            Self::Sim => Some(Self::PostSim),
            Self::PostSim => Some(Self::PreRender),
            Self::PreRender => Some(Self::Render),
            Self::Render => Some(Self::PostRender),
            Self::PostRender => Some(Self::PreInput),
            Self::PreInput => Some(Self::Input),
            Self::Input => Some(Self::PostInput),
            Self::PostInput => None,
        }
    }
}
