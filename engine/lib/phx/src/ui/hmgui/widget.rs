use glam::Vec2;

use super::{HmGuiGroup, HmGuiImage, HmGuiRect, HmGuiText, Rf};

#[derive(Clone, Default, PartialEq)]
pub enum WidgetItem {
    #[default]
    Undefined,
    Group(Rf<HmGuiGroup>),
    Text(HmGuiText),
    Rect(HmGuiRect),
    Image(HmGuiImage),
}

#[derive(Copy, Clone, Default, PartialEq)]
pub enum LayoutType {
    #[default]
    None,
    Stack,
    Vertical,
    Horizontal,
}

#[derive(Clone, Default, PartialEq)]
pub struct HmGuiWidget {
    pub parent: Option<Rf<HmGuiGroup>>,
    pub next: Option<Rf<HmGuiWidget>>,
    pub prev: Option<Rf<HmGuiWidget>>,

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
