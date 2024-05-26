use glam::Vec2;

/// Persistent data storage used for the widget scrolling.
#[derive(Clone)]
pub struct HmGuiData {
    /// Scrolling offset.
    pub offset: Vec2,

    /// Min size of the widget after compute_size().
    pub min_size: Vec2,

    /// Actual size of the widget after layout().
    pub size: Vec2,

    /// Actual position of the widget after layout().
    pub pos: Vec2,
}

impl Default for HmGuiData {
    fn default() -> Self {
        Self {
            offset: Default::default(),
            min_size: Default::default(),
            size: Default::default(),
            pos: Default::default(),
        }
    }
}
