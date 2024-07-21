use glam::Vec2;

use super::TextView;

/// Persistent data storage used for the widget scrolling.
pub struct HmGuiData {
    /// Scrolling offset.
    pub offset: Vec2,

    /// Min size of the widget after compute_size().
    pub min_size: Vec2,

    /// Actual size of the widget after layout().
    pub size: Vec2,

    /// Actual position of the widget after layout().
    pub pos: Vec2,

    /// Text view data. Only for the text view widgets.
    pub text_view: Option<TextView>,
}

impl Default for HmGuiData {
    fn default() -> Self {
        Self {
            offset: Default::default(),
            min_size: Default::default(),
            size: Default::default(),
            pos: Default::default(),
            text_view: Default::default(),
        }
    }
}
