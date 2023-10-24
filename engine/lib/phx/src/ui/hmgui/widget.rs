use std::{fs::File, io::Write};

use glam::Vec2;

use super::{HmGui, HmGuiGroup, HmGuiImage, HmGuiRect, HmGuiText, Rf, IDENT};

#[derive(Clone, PartialEq)]
pub enum WidgetItem {
    Group(HmGuiGroup),
    Text(HmGuiText),
    Rect(HmGuiRect),
    Image(HmGuiImage),
}

impl WidgetItem {
    fn name(&self) -> &str {
        match self {
            WidgetItem::Group(_) => "Group",
            WidgetItem::Text(_) => "Text",
            WidgetItem::Rect(_) => "Rect",
            WidgetItem::Image(_) => "Image",
        }
    }
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
    pub min_size: Vec2,
    pub align: Vec2,
    pub stretch: Vec2,
}

impl HmGuiWidget {
    pub fn compute_size(&mut self, hmgui: &mut HmGui) {
        match &self.item {
            WidgetItem::Group(group) => {
                self.min_size = Vec2::ZERO;

                group.compute_size(hmgui, &mut self.min_size);

                if group.store_size {
                    let data = hmgui.get_data(self.hash);

                    data.min_size = self.min_size;
                }
            }
            _ => {}
        }
    }

    pub fn layout(&self, hmgui: &mut HmGui) {
        match &self.item {
            WidgetItem::Group(group) => {
                group.layout(hmgui, self.pos, self.size, self.size - self.min_size);

                if group.store_size {
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
            WidgetItem::Text(text) => {
                text.draw(
                    &hmgui.renderer,
                    Vec2::new(self.pos.x, self.pos.y + self.min_size.y),
                );
            }
            WidgetItem::Rect(rect) => {
                rect.draw(&hmgui.renderer, self.pos, self.size);
            }
            WidgetItem::Image(image) => {
                image.draw(&hmgui.renderer, self.pos, self.size);
            }
        }
    }

    pub fn layout_item(&mut self, pos: Vec2, sx: f32, sy: f32) {
        self.pos = pos;
        self.size = self.min_size;
        self.size.x += self.stretch.x * (sx - self.min_size.x);
        self.size.y += self.stretch.y * (sy - self.min_size.y);
        self.pos.x += self.align.x * (sx - self.size.x);
        self.pos.y += self.align.y * (sy - self.size.y);
    }

    // For testing.
    #[allow(dead_code)]
    pub(crate) fn dump(&self, ident: usize, file: &mut File) {
        writeln!(
            file,
            "{} {} {} {} {}",
            self.item.name(),
            self.pos.x,
            self.pos.y,
            self.size.x,
            self.size.y
        )
        .expect("Cannot write line");

        let ident_str = format!("{}", IDENT.repeat(ident));

        println!("{ident_str}{}:", self.item.name());
        println!("{ident_str}{IDENT}- pos:      {:?}", self.pos);
        println!("{ident_str}{IDENT}- size:     {:?}", self.size);
        println!("{ident_str}{IDENT}- min_size: {:?}", self.min_size);
        println!("{ident_str}{IDENT}- align:    {:?}", self.align);
        println!("{ident_str}{IDENT}- stretch:  {:?}", self.stretch);
        println!("{ident_str}{IDENT}- hash:     0x{:X?}", self.hash);
        println!("{ident_str}{IDENT}# item: {}", self.item.name());

        match &self.item {
            WidgetItem::Group(item) => item.dump(ident + 1, file),
            WidgetItem::Text(item) => item.dump(ident + 1),
            WidgetItem::Rect(item) => item.dump(ident + 1),
            WidgetItem::Image(item) => item.dump(ident + 1),
        }
    }
}
