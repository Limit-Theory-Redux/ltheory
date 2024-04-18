use glam::Vec2;

use crate::render::Color;

use super::{
    AlignHorizontal, AlignVertical, FocusType, HmGui, HmGuiContainer, HmGuiImage, HmGuiProperties,
    HmGuiRect, HmGuiText, RenderStyle, IDENT,
};

use crate::rf::Rf;

#[derive(Clone, PartialEq)]
pub enum WidgetItem {
    Container(HmGuiContainer),
    Text(HmGuiText),
    Rect(HmGuiRect),
    Image(HmGuiImage),
}

impl WidgetItem {
    pub fn name(&self) -> String {
        match self {
            WidgetItem::Container(item) => format!("Container/{:?}", item.layout),
            WidgetItem::Text(text) => format!("Text/{}", text.text),
            WidgetItem::Rect(_) => "Rect".into(),
            WidgetItem::Image(_) => "Image".into(),
        }
    }

    pub fn is_container(&self) -> bool {
        matches!(self, Self::Container(_))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Length {
    Fixed(f32),
    Percent(f32),
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
    pub default_width: Option<Length>,
    pub default_height: Option<Length>,
    pub horizontal_alignment: AlignHorizontal,
    pub vertical_alignment: AlignVertical,
    pub margin_upper: Vec2,
    pub margin_lower: Vec2,
    pub bg_color: Option<Color>,
    pub border_width: f32,
    pub border_color: Color,

    /// Widget min size after compute_size() including margin and border
    pub min_size: Vec2,
    /// Widget min size after compute_size() excluding margin and border
    pub inner_min_size: Vec2,

    pub render_style: RenderStyle,
    pub mouse_over: [bool; FocusType::SIZE],
}

impl HmGuiWidget {
    pub fn new(parent: Option<Rf<HmGuiWidget>>, item: WidgetItem) -> Self {
        Self {
            parent,

            hash: 0,
            item,

            pos: Default::default(),
            size: Default::default(),
            inner_pos: Default::default(),
            inner_size: Default::default(),

            default_width: Default::default(),
            default_height: Default::default(),
            horizontal_alignment: Default::default(),
            vertical_alignment: Default::default(),
            margin_upper: Default::default(),
            margin_lower: Default::default(),
            bg_color: Default::default(),
            border_width: Default::default(),
            border_color: Default::default(),

            min_size: Default::default(),
            inner_min_size: Vec2::new(20.0, 20.0),

            render_style: Default::default(),
            mouse_over: Default::default(),
        }
    }

    pub fn get_container_item(&self) -> &HmGuiContainer {
        let WidgetItem::Container(item) = &self.item else {
            panic!("Expected container but was: {}", self.item.name())
        };

        item
    }

    pub fn get_container_item_mut(&mut self) -> &mut HmGuiContainer {
        let item_name = self.item.name().to_string();
        let WidgetItem::Container(item) = &mut self.item else {
            panic!("Expected container but was: {}", item_name)
        };

        item
    }

    pub fn contains_point(&self, point: &Vec2) -> bool {
        self.pos.x <= point.x
            && self.pos.y <= point.y
            && point.x <= self.pos.x + self.size.x
            && point.y <= self.pos.y + self.size.y
    }

    /// Calculate outer min size that includes margin and border.
    fn calculate_min_size(&self) -> Vec2 {
        let mut inner_min_width = self.inner_min_size.x;
        if let Some(default_width) = self.default_width {
            if let Length::Fixed(fixed_width) = default_width {
                inner_min_width = fixed_width;
            }
        }

        let mut inner_min_height = self.inner_min_size.y;
        if let Some(default_height) = self.default_height {
            if let Length::Fixed(fixed_height) = default_height {
                inner_min_height = fixed_height;
            }
        }

        let x =
            inner_min_width + self.border_width * 2.0 + self.margin_upper.x + self.margin_lower.x;
        let y =
            inner_min_height + self.border_width * 2.0 + self.margin_upper.y + self.margin_lower.y;

        Vec2 { x, y }
    }

    /// Calculate inner pos and size from outer ones by subtracting margins and border.
    /// Do not subtract if outer width and/or height is 0.
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
        match &self.item {
            WidgetItem::Container(container) => {
                self.inner_min_size = container.compute_size(hmgui);

                self.min_size = self.calculate_min_size();

                if container.scroll_dir.is_some() {
                    let data = hmgui.get_data(self.hash);

                    data.min_size = self.min_size;
                }
            }
            _ => {
                self.min_size = self.calculate_min_size();
            }
        }
    }

    pub fn layout(&mut self, hmgui: &mut HmGui) {
        // TODO: do not process widgets with min size, margin and border all 0
        match &self.item {
            WidgetItem::Container(container) => {
                self.inner_size = container.layout(
                    hmgui,
                    self.horizontal_alignment == AlignHorizontal::Stretch,
                    self.vertical_alignment == AlignVertical::Stretch,
                    self.inner_pos,
                    self.inner_size,
                    self.inner_size - self.inner_min_size,
                );
                self.size = self.inner_size
                    + self.border_width * 2.0
                    + self.margin_upper
                    + self.margin_lower;

                if container.scroll_dir.is_some() {
                    let data = hmgui.get_data(self.hash);

                    data.size = self.size;
                }
            }
            _ => {}
        }
    }

    pub fn draw(&self, hmgui: &mut HmGui) {
        let size = self.inner_size;

        if size.x > 0.0 && size.y > 0.0 {
            let pos = self.inner_pos;

            if self.border_width > 0.0 {
                hmgui.renderer.rect(
                    pos - self.border_width,
                    size + self.border_width * 2.0,
                    self.border_color,
                    Some(self.border_width),
                );
            }

            if let Some(bg_color) = self.bg_color {
                hmgui.renderer.rect(pos, size, bg_color, None);
            }

            match &self.item {
                WidgetItem::Container(container) => {
                    container.draw(hmgui, pos, size);
                }
                WidgetItem::Text(text) => {
                    let x = pos.x + (size.x - self.inner_min_size.x) / 2.0; // center text
                    let y = pos.y + self.min_size.y
                        - (self.border_width * 2.0 + self.margin_upper.y + self.margin_lower.y);

                    text.draw(&mut hmgui.renderer, Vec2::new(x, y));
                }
                WidgetItem::Rect(rect) => {
                    rect.draw(&mut hmgui.renderer, pos, size);
                }
                WidgetItem::Image(image) => {
                    image.draw(&mut hmgui.renderer, pos, size);
                }
            }

            self.process_mouse_over(hmgui, pos, size);
        }
    }

    fn process_mouse_over(&self, hmgui: &mut HmGui, pos: Vec2, size: Vec2) {
        if self.mouse_over[FocusType::Mouse as usize] {
            let is_mouse_over = hmgui.mouse_over_widget_hash() == self.hash;

            match self.render_style {
                RenderStyle::None => {
                    let color = hmgui.get_property_color(HmGuiProperties::BackgroundColorId.id());
                    let inner_alpha = hmgui.get_property_f32(HmGuiProperties::OpacityId.id());

                    hmgui
                        .renderer
                        .panel(pos, size, color.clone(), 8.0, inner_alpha);
                }
                RenderStyle::Fill => {
                    if is_mouse_over {
                        let color =
                            hmgui.get_property_color(HmGuiProperties::HighlightColorId.id());

                        hmgui.renderer.panel(pos, size, color.clone(), 0.0, 1.0);
                    } else {
                        let color =
                            hmgui.get_property_color(HmGuiProperties::BackgroundColorId.id());
                        let inner_alpha = hmgui.get_property_f32(HmGuiProperties::OpacityId.id());

                        hmgui
                            .renderer
                            .panel(pos, size, color.clone(), 0.0, inner_alpha);
                    }
                }
                RenderStyle::Outline => {
                    // TODO: not used. Remove?
                    if is_mouse_over {
                        let color = Color::new(0.1, 0.5, 1.0, 1.0);

                        hmgui.renderer.rect(pos, size, color, Some(1.0));
                    }
                }
                RenderStyle::Underline => {
                    let color = if is_mouse_over {
                        hmgui.get_property_color(HmGuiProperties::BackgroundColorId.id())
                    } else {
                        hmgui.get_property_color(HmGuiProperties::HighlightColorId.id())
                    };

                    hmgui.renderer.rect(pos, size, color.clone(), None);
                }
            }
        }
    }

    // For testing.
    #[allow(dead_code)]
    #[rustfmt::skip]
    pub(crate) fn dump(&self, ident: usize) {
        let ident_str = format!("{}", IDENT.repeat(ident));

        println!("{ident_str}{}:", self.item.name());
        println!("{ident_str}{IDENT}- pos:            {:?}", self.pos);
        println!("{ident_str}{IDENT}- size:           {:?}", self.size);
        println!("{ident_str}{IDENT}- inner_pos:      {:?}", self.inner_pos);
        println!("{ident_str}{IDENT}- inner_size:     {:?}", self.inner_size);
        println!("{ident_str}{IDENT}- default_width:  {:?}", self.default_width);
        println!("{ident_str}{IDENT}- default_height: {:?}", self.default_height);
        println!("{ident_str}{IDENT}- horiz_align:    {:?}", self.vertical_alignment);
        println!("{ident_str}{IDENT}- vert_align:     {:?}", self.horizontal_alignment);
        println!("{ident_str}{IDENT}- margin_upper:   {:?}", self.margin_upper);
        println!("{ident_str}{IDENT}- margin_lower:   {:?}", self.margin_lower);
        println!("{ident_str}{IDENT}- bg_color:       {:?}", self.bg_color);
        println!("{ident_str}{IDENT}- border_width:   {}",   self.border_width);
        println!("{ident_str}{IDENT}- border_color:   {:?}", self.border_color);
        println!("{ident_str}{IDENT}- min_size:       {:?}", self.min_size);
        println!("{ident_str}{IDENT}- inner_min_size: {:?}", self.inner_min_size);
        println!("{ident_str}{IDENT}- hash:           0x{:X?}", self.hash);
        println!("{ident_str}{IDENT}- render_style:   {:?}", self.render_style);
        println!("{ident_str}{IDENT}- mouse_over:     {:?}", self.mouse_over);
        println!("{ident_str}{IDENT}# item: {}", self.item.name());

        match &self.item {
            WidgetItem::Container(item) => item.dump(ident + 1),
            WidgetItem::Text(item) => item.dump(ident + 1),
            WidgetItem::Rect(item) => item.dump(ident + 1),
            WidgetItem::Image(item) => item.dump(ident + 1),
        }
    }
}
