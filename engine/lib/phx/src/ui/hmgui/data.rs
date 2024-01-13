use glam::Vec2;

/// Persistent data storage used for the widget scrolling.
#[derive(Clone, Default)]
pub struct HmGuiData {
    /// Scrolling offset.
    pub offset: Vec2,
    /// Min size of the widget after compute_size().
    pub min_size: Vec2,
    /// Actual size of the widget after layout().
    pub size: Vec2,
}
