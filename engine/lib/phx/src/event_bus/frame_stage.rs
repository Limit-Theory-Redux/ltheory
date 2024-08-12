use strum_macros::EnumIter;

use super::Event;

/// Frame stages in order they are processed.
/// Events can be registered for each stage that will be dispatched in order they ere sent.
#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter)]
pub enum FrameStage {
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
    /// Before input handling
    PreInput,
    /// Input handling
    Input,
    /// After input handling
    PostInput,
}

impl FrameStage {
    pub const fn first() -> Self {
        Self::PreSim
    }

    pub const fn last() -> Self {
        Self::PostInput
    }

    pub const fn len() -> usize {
        Self::last().index() + 1
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

    pub const fn index(&self) -> usize {
        *self as usize
    }

    pub fn as_event_type(&self) -> Event {
        match self {
            Self::PreSim => Event::PreSim,
            Self::Sim => Event::Sim,
            Self::PostSim => Event::PostSim,
            Self::PreRender => Event::PreRender,
            Self::Render => Event::Render,
            Self::PostRender => Event::PostRender,
            Self::PreInput => Event::PreInput,
            Self::Input => Event::Input,
            Self::PostInput => Event::PostInput,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::event_bus::FrameStage;

    #[test]
    fn test_frame_stage_first() {
        assert_eq!(FrameStage::first(), FrameStage::PreSim);
    }

    #[test]
    fn test_frame_stage_last() {
        assert_eq!(FrameStage::last(), FrameStage::PostInput);
    }

    #[test]
    fn test_frame_stage_order() {
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
        assert_eq!(
            FrameStage::PostRender.index() + 1,
            FrameStage::PreInput.index()
        );
        assert_eq!(FrameStage::PreInput.index() + 1, FrameStage::Input.index());
        assert_eq!(FrameStage::Input.index() + 1, FrameStage::PostInput.index());
    }

    #[test]
    fn test_frame_stage_next() {
        assert_eq!(FrameStage::PreSim.next(), Some(FrameStage::Sim));
        assert_eq!(FrameStage::Sim.next(), Some(FrameStage::PostSim));
        assert_eq!(FrameStage::PostSim.next(), Some(FrameStage::PreRender));
        assert_eq!(FrameStage::PreRender.next(), Some(FrameStage::Render));
        assert_eq!(FrameStage::Render.next(), Some(FrameStage::PostRender));
        assert_eq!(FrameStage::PostRender.next(), Some(FrameStage::PreInput));
        assert_eq!(FrameStage::PreInput.next(), Some(FrameStage::Input));
        assert_eq!(FrameStage::Input.next(), Some(FrameStage::PostInput));
        assert_eq!(FrameStage::PostInput.next(), None);
    }
}
