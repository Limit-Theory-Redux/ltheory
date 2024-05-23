use glam::Vec2;

use crate::{rf::Rf, system::Hash_FNV64_Init};

use super::{HmGuiWidget, WidgetItem};

pub const UNDEFINED_LAYER_INDEX: usize = 0xFFFFFFFF;

pub struct HmGuiLayer {
    // Previous layer index. [`UNDEFINED_LAYER_INDEX`] if this is the first layer.
    pub prev_index: usize,
    /// Top level container object with Stack layout. Used for recalculating sizes, layouts and drawing of the whole gui
    pub root: Rf<HmGuiWidget>,
    /// Current active container
    pub container: Rf<HmGuiWidget>,
    /// Either last created/initialized widget (container, image, text, rect) or the last widget of the ended container
    pub last: Rf<HmGuiWidget>,
}

impl HmGuiLayer {
    pub fn new(prev_index: usize, sx: f32, sy: f32) -> Self {
        let mut widget = HmGuiWidget::new(None, WidgetItem::Container(Default::default()));

        widget.hash = Hash_FNV64_Init();
        widget.inner_size = Vec2::new(sx, sy);
        widget.size = widget.inner_size;

        let root = Rf::new(widget);
        let container = root.clone();
        let last = root.clone();

        Self {
            prev_index,
            root,
            container,
            last,
        }
    }
}
