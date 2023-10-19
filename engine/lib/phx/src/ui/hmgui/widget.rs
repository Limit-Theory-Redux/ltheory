use glam::Vec2;

use super::{HmGui, HmGuiGroup, HmGuiImage, HmGuiRect, HmGuiText, Rf};

#[derive(Clone, PartialEq)]
pub enum WidgetItem {
    Group(HmGuiGroup),
    Text(HmGuiText),
    Rect(HmGuiRect),
    Image(HmGuiImage),
}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub enum LayoutType {
    #[default]
    None,
    Stack,
    Vertical,
    Horizontal,
}

#[derive(Clone, PartialEq)]
pub struct HmGuiWidget {
    pub parent: Option<Rf<HmGuiWidget>>,
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
    pub fn compute_size(&mut self, hmgui: &mut HmGui) {
        match &self.item {
            WidgetItem::Group(group) => {
                self.minSize = Vec2::ZERO;

                group.compute_size(hmgui, &mut self.minSize);

                if group.storeSize {
                    let data = hmgui.get_data(self.hash);

                    data.minSize = self.minSize;
                }
            }
            _ => {}
        }
    }

    pub fn layout(&self, hmgui: &mut HmGui) {
        match &self.item {
            WidgetItem::Group(group) => {
                group.layout(hmgui, self.pos, self.size, self.size - self.minSize);

                if group.storeSize {
                    let data = hmgui.get_data(self.hash);

                    data.size = self.size;
                }
            }
            _ => {}
        }
    }

    pub fn draw(&self, hmgui: &mut HmGui) {
        match &self.item {
            WidgetItem::Group(group) => {
                let hmgui_focus = hmgui.mouse_focus_hash();

                group.draw(hmgui, self.pos, self.size, hmgui_focus == self.hash);
            }
            WidgetItem::Text(item) => {
                item.draw(self.pos.x, self.pos.y + self.minSize.y);
            }
            WidgetItem::Rect(item) => {
                item.draw(self.pos, self.size);
            }
            WidgetItem::Image(item) => {
                item.draw(self.pos, self.size);
            }
        }
    }

    pub fn layout_item(&mut self, pos: Vec2, sx: f32, sy: f32) {
        self.pos = pos;
        self.size = self.minSize;
        self.size.x += self.stretch.x * (sx - self.minSize.x);
        self.size.y += self.stretch.y * (sy - self.minSize.y);
        self.pos.x += self.align.x * (sx - self.size.x);
        self.pos.y += self.align.y * (sy - self.size.y);
    }
}
