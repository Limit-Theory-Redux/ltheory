use std::{fs::File, io::Write};

use glam::{Vec2, Vec4};

use super::{DockingType, HmGui, HmGuiContainer, HmGuiImage, HmGuiRect, HmGuiText, Rf, IDENT};

#[derive(Clone, PartialEq)]
pub enum WidgetItem {
    Container(HmGuiContainer),
    Text(HmGuiText),
    Rect(HmGuiRect),
    Image(HmGuiImage),
}

impl WidgetItem {
    fn name(&self) -> &str {
        match self {
            WidgetItem::Container(_) => "Container",
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

    pub hash: u64,
    pub item: WidgetItem,

    /// Left/top widget position including margin and border
    pub pos: Vec2,
    /// Widget final size after layout calculating including margin and border
    pub size: Vec2,
    /// Left/top widget position excluding margin and border
    pub inner_pos: Vec2,
    /// Widget final size after layout calculating excluding margin and border
    pub inner_size: Vec2,

    // Layout
    pub fixed_width: Option<f32>,
    pub fixed_height: Option<f32>,
    pub docking: DockingType,
    pub margin_upper: Vec2,
    pub margin_lower: Vec2,
    pub bg_color: Vec4,
    pub border_width: f32,
    pub border_color: Vec4,

    /// Widget min size after compute_size() including margin and border
    pub min_size: Vec2,
    /// Widget min size after compute_size() excluding margin and border
    pub inner_min_size: Vec2,

    pub align: Vec2,
    pub stretch: Vec2,
}

impl HmGuiWidget {
    pub fn get_container_item(&self) -> Option<&HmGuiContainer> {
        if let WidgetItem::Container(item) = &self.item {
            Some(item)
        } else {
            None
        }
    }

    /// Calculate outer min size that includes margin and border.
    /// Do not add margins if min size and border width are both 0.
    fn calculate_min_size(&self) -> Vec2 {
        let mut inner_min_size = self.inner_min_size;

        if !self.docking.is_dock_left() && !self.docking.is_dock_right() {
            if let Some(fixed_width) = self.fixed_width {
                inner_min_size.x = fixed_width;
            }
        }
        if !self.docking.is_dock_top() && !self.docking.is_dock_bottom() {
            if let Some(fixed_height) = self.fixed_height {
                inner_min_size.y = fixed_height;
            }
        }

        let x = if inner_min_size.x > 0.0 || self.border_width > 0.0 {
            inner_min_size.x + self.border_width * 2.0 + self.margin_upper.x + self.margin_lower.x
        } else {
            0.0
        };
        let y = if inner_min_size.y > 0.0 || self.border_width > 0.0 {
            inner_min_size.y + self.border_width * 2.0 + self.margin_upper.y + self.margin_lower.y
        } else {
            0.0
        };

        Vec2 { x, y }
    }

    /// Calculate inner pos and size from outer by subtracting margins and border.
    /// Do not subtract of outer width/height is 0.
    pub fn calculate_inner_pos_size(&mut self) {
        if self.size.x > 0.0 {
            self.inner_pos.x = self.pos.x + self.border_width + self.margin_upper.x;
            self.inner_size.x =
                self.size.x - (self.border_width * 2.0 + self.margin_upper.x + self.margin_lower.x);
        } else {
            self.inner_pos.x = self.pos.x;
            self.inner_size.x = 0.0;
        }

        if self.size.y > 0.0 {
            self.inner_pos.y = self.pos.y + self.border_width + self.margin_upper.y;
            self.inner_size.y =
                self.size.y - (self.border_width * 2.0 + self.margin_upper.y + self.margin_lower.y);
        } else {
            self.inner_pos.y = self.pos.y;
            self.inner_size.y = 0.0;
        }
    }

    pub fn compute_size(&mut self, hmgui: &mut HmGui) {
        println!("Widget::compute_size({}):", self.item.name());

        match &self.item {
            WidgetItem::Container(container) => {
                self.inner_min_size = container.compute_size(hmgui);

                self.min_size = self.calculate_min_size();

                println!(
                    "Widget::compute_size(Container/{:?}): inner_min={:?}, outer_min={:?}",
                    container.layout, self.inner_min_size, self.min_size
                );

                if container.store_size {
                    let data = hmgui.get_data(self.hash);

                    data.min_size = self.min_size;
                }
            }
            _ => {}
        }
    }

    pub fn layout(&self, hmgui: &mut HmGui) {
        println!("Widget::layout({}): begin", self.item.name());

        // Do not process zero square widget
        if self.min_size.x > 0.0 && self.min_size.y > 0.0 {
            match &self.item {
                WidgetItem::Container(container) => {
                    container.layout(
                        hmgui,
                        self.inner_pos,
                        self.inner_size,
                        self.inner_size - self.inner_min_size,
                    );

                    println!("  - inner={:?}, outer={:?}", self.inner_size, self.size);

                    if container.store_size {
                        let data = hmgui.get_data(self.hash);

                        data.size = self.size;
                    }
                }
                _ => {}
            }
        }
    }

    pub fn draw(&self, hmgui: &mut HmGui) {
        let size = self.size - (self.border_width * 2.0 + self.margin_upper + self.margin_lower);

        if size.x > 0.0 && size.y > 0.0 {
            let pos = self.pos + self.border_width + self.margin_upper;

            match &self.item {
                WidgetItem::Container(container) => {
                    let hmgui_focus = hmgui.mouse_focus_hash();

                    container.draw(hmgui, pos, size, hmgui_focus == self.hash);
                }
                WidgetItem::Text(text) => {
                    let min_size_y = self.min_size.y
                        - (self.border_width * 2.0 + self.margin_upper.y + self.margin_lower.y);
                    text.draw(&mut hmgui.renderer, Vec2::new(pos.x, pos.y + min_size_y));
                }
                WidgetItem::Rect(rect) => {
                    rect.draw(&mut hmgui.renderer, pos, size);
                }
                WidgetItem::Image(image) => {
                    image.draw(&mut hmgui.renderer, pos, size);
                }
            }
        }
    }

    // For testing.
    #[allow(dead_code)]
    #[rustfmt::skip]
    pub(crate) fn dump(&self, ident: usize, file: &mut Option<File>) {
        if let Some(file) = file {
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
        }

        let ident_str = format!("{}", IDENT.repeat(ident));

        println!("{ident_str}{}:", self.item.name());
        println!("{ident_str}{IDENT}- pos:            {:?}", self.pos);
        println!("{ident_str}{IDENT}- size:           {:?}", self.size);
        println!("{ident_str}{IDENT}- inner_pos:      {:?}", self.inner_pos);
        println!("{ident_str}{IDENT}- inner_size:     {:?}", self.inner_size);
        println!("{ident_str}{IDENT}- fixed_width:    {:?}", self.fixed_width);
        println!("{ident_str}{IDENT}- fixed_height:   {:?}", self.fixed_height);
        println!("{ident_str}{IDENT}- docking:        {:?}", self.docking);
        println!("{ident_str}{IDENT}- margin_upper:   {:?}", self.margin_upper);
        println!("{ident_str}{IDENT}- margin_lower:   {:?}", self.margin_lower);
        println!("{ident_str}{IDENT}- bg_color:       {:?}", self.bg_color);
        println!("{ident_str}{IDENT}- border_width:   {}",   self.border_width);
        println!("{ident_str}{IDENT}- border_color:   {:?}", self.border_color);
        println!("{ident_str}{IDENT}- min_size:       {:?}", self.min_size);
        println!("{ident_str}{IDENT}- inner_min_size: {:?}", self.inner_min_size);
        println!("{ident_str}{IDENT}- hash:           0x{:X?}", self.hash);
        println!("{ident_str}{IDENT}# item: {}", self.item.name());

        match &self.item {
            WidgetItem::Container(item) => item.dump(ident + 1, file),
            WidgetItem::Text(item) => item.dump(ident + 1),
            WidgetItem::Rect(item) => item.dump(ident + 1),
            WidgetItem::Image(item) => item.dump(ident + 1),
        }
    }
}
