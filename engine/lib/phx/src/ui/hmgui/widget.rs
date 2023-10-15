use glam::Vec2;

use super::group::HmGuiGroup;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum WidgetType {
    Group,
    Text,
    Rect,
    Image,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum LayoutType {
    None,
    Stack,
    Vertical,
    Horizontal,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGuiWidget {
    pub parent: *mut HmGuiGroup,
    pub next: *mut HmGuiWidget,
    pub prev: *mut HmGuiWidget,
    pub hash: u64,
    pub ty: WidgetType,
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
