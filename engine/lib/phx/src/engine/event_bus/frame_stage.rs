use strum_macros::EnumIter;

use super::Event;

/// Frame stages in order they are processed.
/// Standard game loop: gather input → simulate physics → render.
/// Events can be registered for each stage that will be dispatched in order they were sent.
#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter)]
pub enum FrameStage {
    /// Before input handling
    PreInput,
    /// Input handling
    Input,
    /// After input handling
    PostInput,
    /// Before physics update
    PreSim,
    /// Physics update
    Sim,
    /// After physics update
    PostSim,
    /// Before frame render
    PreRender,
    /// Frame render
    Render,
    /// After frame render
    PostRender,
}

impl FrameStage {
    pub const fn first() -> Self {
        Self::PreInput
    }

    pub const fn last() -> Self {
        Self::PostRender
    }

    pub const fn len() -> usize {
        Self::last().index() + 1
    }

    pub fn next(&self) -> Option<Self> {
        match self {
            Self::PreInput => Some(Self::Input),
            Self::Input => Some(Self::PostInput),
            Self::PostInput => Some(Self::PreSim),
            Self::PreSim => Some(Self::Sim),
            Self::Sim => Some(Self::PostSim),
            Self::PostSim => Some(Self::PreRender),
            Self::PreRender => Some(Self::Render),
            Self::Render => Some(Self::PostRender),
            Self::PostRender => None,
        }
    }

    pub const fn index(&self) -> usize {
        *self as usize
    }

    pub fn as_event_type(&self) -> Event {
        match self {
            Self::PreInput => Event::PreInput,
            Self::Input => Event::Input,
            Self::PostInput => Event::PostInput,
            Self::PreSim => Event::PreSim,
            Self::Sim => Event::Sim,
            Self::PostSim => Event::PostSim,
            Self::PreRender => Event::PreRender,
            Self::Render => Event::Render,
            Self::PostRender => Event::PostRender,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::event_bus::FrameStage;

    #[test]
    fn test_frame_stage_first() {
        assert_eq!(FrameStage::first(), FrameStage::PreInput);
    }

    #[test]
    fn test_frame_stage_last() {
        assert_eq!(FrameStage::last(), FrameStage::PostRender);
    }

    #[test]
    fn test_frame_stage_order() {
        assert_eq!(FrameStage::PreInput.index() + 1, FrameStage::Input.index());
        assert_eq!(FrameStage::Input.index() + 1, FrameStage::PostInput.index());
        assert_eq!(
            FrameStage::PostInput.index() + 1,
            FrameStage::PreSim.index()
        );
        assert_eq!(FrameStage::PreSim.index() + 1, FrameStage::Sim.index());
        assert_eq!(FrameStage::Sim.index() + 1, FrameStage::PostSim.index());
        assert_eq!(
            FrameStage::PostSim.index() + 1,
            FrameStage::PreRender.index()
        );
        assert_eq!(
            FrameStage::PreRender.index() + 1,
            FrameStage::Render.index()
        );
        assert_eq!(
            FrameStage::Render.index() + 1,
            FrameStage::PostRender.index()
        );
    }

    #[test]
    fn test_frame_stage_next() {
        assert_eq!(FrameStage::PreInput.next(), Some(FrameStage::Input));
        assert_eq!(FrameStage::Input.next(), Some(FrameStage::PostInput));
        assert_eq!(FrameStage::PostInput.next(), Some(FrameStage::PreSim));
        assert_eq!(FrameStage::PreSim.next(), Some(FrameStage::Sim));
        assert_eq!(FrameStage::Sim.next(), Some(FrameStage::PostSim));
        assert_eq!(FrameStage::PostSim.next(), Some(FrameStage::PreRender));
        assert_eq!(FrameStage::PreRender.next(), Some(FrameStage::Render));
        assert_eq!(FrameStage::Render.next(), Some(FrameStage::PostRender));
        assert_eq!(FrameStage::PostRender.next(), None);
    }
}
