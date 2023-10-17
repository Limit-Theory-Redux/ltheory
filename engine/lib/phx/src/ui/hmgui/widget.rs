use glam::Vec2;

use super::group::HmGuiGroup;
use super::image::HmGuiImage;
use super::rect::HmGuiRect;
use super::text::HmGuiText;
use super::{HmGuiGroupId, HmGuiWidgetId};

#[derive(Clone)]
pub enum WidgetItem {
    Group(HmGuiGroupId),
    Text(HmGuiText),
    Rect(HmGuiRect),
    Image(HmGuiImage),
}

#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub enum LayoutType {
    #[default]
    None,
    Stack,
    Vertical,
    Horizontal,
}

#[derive(Clone)]
pub struct HmGuiWidget {
    pub parent_id: Option<HmGuiGroupId>,
    pub next_id: Option<HmGuiWidgetId>,
    pub prev_id: Option<HmGuiWidgetId>,

    pub hash: u64,
    pub item: WidgetItem,
    pub pos: Vec2,
    pub size: Vec2,
    pub minSize: Vec2,
    pub align: Vec2,
    pub stretch: Vec2,
}

impl HmGuiWidget {
    pub fn layout(&mut self, pos: Vec2, sx: f32, sy: f32) {
        self.pos = pos;
        self.size = self.minSize;
        self.size.x += self.stretch.x * (sx - self.minSize.x);
        self.size.y += self.stretch.y * (sy - self.minSize.y);
        self.pos.x += self.align.x * (sx - self.size.x);
        self.pos.y += self.align.y * (sy - self.size.y);
    }
}
