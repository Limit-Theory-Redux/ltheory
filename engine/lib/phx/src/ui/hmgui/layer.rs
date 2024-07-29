use glam::Vec2;

use super::{HmGuiWidget, WidgetItem};
use crate::rf::Rf;
use crate::system::Hash_FNV64_Init;

pub const UNDEFINED_LAYER_INDEX: usize = 0xFFFFFFFF;

/// Specify how layer will be positioned on the screen.
#[derive(Clone, Copy)]
pub enum HmGuiLayerLocation {
    /// New layer will occupy predefined position and size on the screen.
    Fixed,
    /// New layer position will be below the last element (defined by u64 hash) of the current layer.
    /// Size of the layer will be up to the screen borders.
    Below(u64),
}

/// Layer represents a separate hierarchy of elements with a separate layouting.
pub struct HmGuiLayer {
    /// Where layer positioned on a screen.
    pub location: HmGuiLayerLocation,
    // Previous layer index. [`UNDEFINED_LAYER_INDEX`] if this is the first layer.
    pub prev_index: usize,
    /// Top level container object with Stack layout. Used for recalculating sizes, layouts and drawing of the whole gui
    pub root: Rf<HmGuiWidget>,
    /// Current active container
    pub container: Rf<HmGuiWidget>,
    /// Either last created/initialized widget (container, image, text, rect) or the last widget of the ended container
    pub last: Rf<HmGuiWidget>,
}

// TODO: root widget of the layer has a fixed size. Check if it would be possible to make it flexible (via alignment)
// so there is no need to add internal container to have properly sized visualization
impl HmGuiLayer {
    pub fn new_fixed(prev_index: usize, pos: Vec2, size: Vec2) -> Self {
        let mut widget = HmGuiWidget::new(None, WidgetItem::Container(Default::default()));

        widget.hash = Hash_FNV64_Init();

        widget.pos = pos;
        widget.size = size;
        widget.inner_pos = pos;
        widget.inner_size = size;

        let root = Rf::new(widget);
        let container = root.clone();
        let last = root.clone();

        Self {
            location: HmGuiLayerLocation::Fixed,
            prev_index,
            root,
            container,
            last,
        }
    }

    pub fn new_below(prev_index: usize, hash: u64) -> Self {
        let mut widget = HmGuiWidget::new(None, WidgetItem::Container(Default::default()));

        widget.hash = Hash_FNV64_Init();

        let root = Rf::new(widget);
        let container = root.clone();
        let last = root.clone();

        Self {
            location: HmGuiLayerLocation::Below(hash),
            prev_index,
            root,
            container,
            last,
        }
    }

    pub fn set_location(&mut self, pos: Vec2, size: Vec2) {
        let mut widget = self.root.as_mut();

        widget.pos = pos;
        widget.size = size;
    }
}
