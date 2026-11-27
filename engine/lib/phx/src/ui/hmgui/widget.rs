use std::borrow::BorrowMut;

use glam::Vec2;

use super::{
    Alignment, FocusType, HmGui, HmGuiContainer, HmGuiImage, HmGuiShape, HmGuiText, IDENT, TEXT_CTX,
};
use crate::input::Input;
use crate::render::{Color, Renderer};
use crate::rf::Rf;
use crate::ui::hmgui::HmGuiImageLayout;

#[derive(Clone)]
pub enum WidgetItem {
    Container(HmGuiContainer),
    Text(HmGuiText),
    Rect,
    Image(HmGuiImage),
    TextView(Option<HmGuiImage>),
    Shape(HmGuiShape),
}

impl WidgetItem {
    pub fn name(&self) -> String {
        match self {
            WidgetItem::Container(item) => format!("Container/{:?}", item.layout),
            WidgetItem::Text(text) => format!("Text/{}", text.text),
            WidgetItem::Rect => "Rect".into(),
            WidgetItem::Image(_) => "Image".into(),
            WidgetItem::TextView(_) => "TextView".into(),
            WidgetItem::Shape(_) => "Shape".into(),
        }
    }

    pub fn is_container(&self) -> bool {
        matches!(self, Self::Container(_))
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum Length {
    #[default]
    Auto,
    Fixed(f32),
    Percent(f32),
}

impl Length {
    pub fn is_auto(&self) -> bool {
        *self == Self::Auto
    }

    pub fn is_fixed(&self) -> bool {
        matches!(self, Self::Fixed(_))
    }
}

#[derive(Clone)]
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
    pub default_size: [Length; 2],
    /// Min size asked for with `set_min_width/height/size`, excluding margin and
    /// border. A container or text is at least this big, and bigger if its
    /// content needs it.
    pub min_size_request: Vec2,
    /// Max size (`set_max_width/height/size`), excluding margin and border:
    /// the widget is never bigger, also when stretched; a text is cut to it.
    /// Infinite when not set.
    pub max_size: Vec2,
    /// Position relative to the parent container's top left corner inside its
    /// border (`set_position`). A positioned widget is out of its parent's
    /// layout: it doesn't move its siblings or size its parent.
    pub position: Option<Vec2>,
    pub alignment: [Alignment; 2],
    /// Right/bottom margin (the side with the larger coordinate)
    pub margin_upper: Vec2,
    /// Left/top margin (the side with the smaller coordinate)
    pub margin_lower: Vec2,
    pub border_width: f32,

    /// The scale asked for with `set_scale` (1 = none): this widget and
    /// everything in it is drawn that much bigger.
    pub own_scale: f32,
    /// The UI scale the widget is laid out with: its own times its parents'
    /// (and its layer's), from `apply_scale`. Its lengths from Lua are design
    /// pixels times this.
    pub scale: f32,

    // Style
    pub border_color: Color,
    pub background_color: Color,
    pub opacity: f32,

    /// Widget min size after compute_size() including margin and border
    pub min_size: Vec2,
    /// Widget min size after compute_size() excluding margin and border
    pub inner_min_size: Vec2,

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

            default_size: Default::default(),
            min_size_request: Vec2::ZERO,
            max_size: Vec2::INFINITY,
            position: None,
            alignment: Default::default(),
            margin_upper: Default::default(),
            margin_lower: Default::default(),
            border_width: Default::default(),
            own_scale: 1.0,
            scale: 1.0,

            border_color: Color::TRANSPARENT,
            background_color: Color::TRANSPARENT,
            opacity: 1.0,

            min_size: Default::default(),
            inner_min_size: Vec2::new(20.0, 20.0),

            mouse_over: Default::default(),
        }
    }

    pub fn set_border_color(&mut self, color: &Color) {
        self.border_color = *color;
    }

    pub fn set_background_color(&mut self, color: &Color) {
        self.background_color = *color;
    }

    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity;
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
            panic!("Expected container but was: {item_name}")
        };

        item
    }

    pub fn contains_point(&self, point: Vec2) -> bool {
        contains_point(self.pos, self.size, point)
    }

    /// Calculate outer min size that includes margin and border.
    fn calculate_min_size_dim<const DIM: usize>(&self) -> f32 {
        let inner_min_size = if let Length::Fixed(size) = self.default_size[DIM] {
            size
        } else {
            self.inner_min_size[DIM]
        };

        inner_min_size.min(self.max_size[DIM]) + self.outer_extra_dim::<DIM>()
    }

    /// Border and margins in a dimension: what the outer size adds to the inner one.
    pub(super) fn outer_extra_dim<const DIM: usize>(&self) -> f32 {
        self.border_width * 2.0 + self.margin_upper[DIM] + self.margin_lower[DIM]
    }

    /// Keep the size the parent gave within the max size.
    fn apply_max_size(&mut self) {
        let max_x = self.max_size.x + self.outer_extra_dim::<0>();
        let max_y = self.max_size.y + self.outer_extra_dim::<1>();

        self.size.x = self.size.x.min(max_x);
        self.size.y = self.size.y.min(max_y);
    }

    /// Calculate outer min size that includes margin and border.
    fn calculate_min_size(&self) -> Vec2 {
        Vec2 {
            x: self.calculate_min_size_dim::<0>(),
            y: self.calculate_min_size_dim::<1>(),
        }
    }

    /// Calculate inner pos and size from outer ones by subtracting margins and border.
    /// Do not subtract if outer width and/or height is 0.
    fn calculate_inner_pos_size_dim<const DIM: usize>(&mut self) {
        if self.size[DIM] > 0.0 {
            self.inner_pos[DIM] = self.pos[DIM] + self.border_width + self.margin_lower[DIM];
            self.inner_size[DIM] = self.size[DIM]
                - (self.border_width * 2.0 + self.margin_upper[DIM] + self.margin_lower[DIM]);
        } else {
            self.inner_pos[DIM] = self.pos[DIM];
            self.inner_size[DIM] = 0.0;
        }
    }

    /// Calculate inner pos and size from outer ones by subtracting margins and border.
    /// Do not subtract if outer width and/or height is 0.
    fn calculate_inner_pos_size(&mut self) {
        self.calculate_inner_pos_size_dim::<0>();
        self.calculate_inner_pos_size_dim::<1>();
    }

    /// Turn the lengths this widget and its children got from Lua (design
    /// pixels) into screen pixels: times the widget's scale, its own
    /// (`set_scale`) times `parent_scale`. Fixed, min and max sizes, position,
    /// margins, border, padding, spacing and fonts are scaled here, shapes when
    /// they are drawn; percentages need nothing. A border stays at least a
    /// pixel wide and lands on whole pixels, and a text gets its font at the
    /// scaled size (whole pixels per em). Called once per frame, top-down,
    /// before `compute_size`.
    pub fn apply_scale(&mut self, hmgui: &mut HmGui, r: &mut Renderer, parent_scale: f32) {
        let scale = parent_scale * self.own_scale;
        self.scale = scale;

        if scale != 1.0 {
            for length in &mut self.default_size {
                if let Length::Fixed(size) = length {
                    *size *= scale;
                }
            }
            self.min_size_request *= scale;
            self.inner_min_size *= scale;
            self.max_size *= scale;
            if let Some(position) = &mut self.position {
                *position *= scale;
            }
            self.margin_upper *= scale;
            self.margin_lower *= scale;
            if self.border_width > 0.0 {
                self.border_width = (self.border_width * scale).round().max(1.0);
            }

            match &mut self.item {
                WidgetItem::Container(container) => {
                    container.padding_lower *= scale;
                    container.padding_upper *= scale;
                    container.spacing *= scale;
                }
                WidgetItem::Text(text) => {
                    text.font = hmgui.scaled_font(r, &text.font, scale);
                }
                WidgetItem::Shape(shape) => {
                    shape.scale = scale;
                }
                _ => {}
            }
        }

        if let WidgetItem::Container(container) = &self.item {
            for widget_rf in &container.children {
                widget_rf.as_mut().apply_scale(hmgui, r, scale);
            }
        }
    }

    pub fn compute_size(&mut self, hmgui: &mut HmGui, r: &mut Renderer) {
        match &mut self.item {
            WidgetItem::Container(container) => {
                self.inner_min_size = container.compute_size(hmgui, r).max(self.min_size_request);

                self.min_size = self.calculate_min_size();

                let data = hmgui.data_mut(self.hash);
                data.min_size = self.min_size;
            }
            WidgetItem::Text(text_item) => {
                // a wrapped text breaks at its max or fixed width
                let width = match self.default_size[0] {
                    Length::Fixed(width) => width.min(self.max_size.x),
                    _ => self.max_size.x,
                };
                let wrap_width = width.is_finite().then_some(width);

                self.inner_min_size = text_item.measure(r, wrap_width).max(self.min_size_request);

                self.min_size = self.calculate_min_size();
            }
            WidgetItem::TextView(image) => {
                if image.is_some() {
                    self.inner_min_size = image.as_ref().unwrap().image.get_size().as_vec2();
                } else {
                    let scale_factor = hmgui.scale_factor() as f32 * self.scale;
                    let data = hmgui.data_mut(self.hash);
                    let text_view = data.text_view.as_mut().expect("Cannot get a text view");

                    let mut text_ctx = TEXT_CTX.lock().expect("Cannot use text context");

                    let size = text_view
                        .data_mut()
                        .calculate_rect(text_ctx.borrow_mut(), scale_factor);

                    if let Some(size) = size {
                        self.inner_min_size = size.max(self.min_size_request);
                    }
                }

                self.min_size = self.calculate_min_size();
            }
            _ => {
                self.min_size = self.calculate_min_size();
            }
        }
    }

    pub fn layout(&mut self, hmgui: &mut HmGui, r: &mut Renderer, input: &Input) {
        self.apply_max_size();
        self.calculate_inner_pos_size();

        let focused = hmgui.in_focus(self);

        // TODO: do not process widgets with min size, margin and border all 0
        match &mut self.item {
            WidgetItem::Container(container) => {
                let is_root = self.parent.is_none();
                // A stretched, fixed, percent or capped (max) size is decided
                // outside the container: it doesn't grow around bigger children
                // (they overflow).
                let constrained = |dim: usize| {
                    !is_root
                        && (self.alignment[dim] == Alignment::Stretch
                            || !self.default_size[dim].is_auto()
                            || self.max_size[dim].is_finite())
                };
                let (constrained_x, constrained_y) = (constrained(0), constrained(1));

                self.inner_size = container.layout(
                    hmgui,
                    r,
                    input,
                    constrained_x,
                    constrained_y,
                    self.inner_pos,
                    self.inner_size,
                );

                self.size = self.inner_size
                    + self.border_width * 2.0
                    + self.margin_upper
                    + self.margin_lower;

                let data = hmgui.data_mut(self.hash);
                data.size = self.size;
                data.pos = self.pos;
            }
            WidgetItem::TextView(image) => {
                let mut clipboard = hmgui.clipboard_get_text();
                let scale_factor = hmgui.scale_factor() as f32 * self.scale;
                let data = hmgui.data_mut(self.hash);
                let text_view = data.text_view.as_mut().expect("Text view data was not set");

                // TODO: TextContext could be part of HmGui without Lazy<Mutex<>> wrapper
                // but here it would conflict with mutable borrow of hmgui.get_data() above.
                // Check if this can be solved.
                let mut text_ctx = TEXT_CTX.lock().expect("Cannot use text context");

                *image = text_view
                    .update(
                        r,
                        text_ctx.borrow_mut(),
                        self.inner_size.x,
                        scale_factor,
                        self.inner_pos,
                        input,
                        focused,
                        &mut clipboard,
                    )
                    .map(|tex_ref| HmGuiImage::new(tex_ref.clone(), HmGuiImageLayout::TopLeft));

                if !clipboard.is_empty() {
                    hmgui.clipboard_set_text(clipboard);
                }
            }
            WidgetItem::Text(text) => {
                // narrower than its text: cut it and end it with an ellipsis
                text.fit(r, self.inner_size.x);
            }
            _ => {}
        }
    }

    pub fn draw(&self, hmgui: &mut HmGui) {
        let size = self.inner_size;

        if size.x > 0.0 && size.y > 0.0 {
            let pos = self.inner_pos;

            // background, border, then content: in that order on screen
            self.draw_background(hmgui, pos, size);

            if self.border_width > 0.0 || self.border_color.is_opaque() {
                hmgui.renderer.rect(
                    pos - self.border_width,
                    size + self.border_width * 2.0,
                    self.border_color,
                    Some(self.border_width),
                );
            }

            match &self.item {
                WidgetItem::Container(container) => {
                    container.draw(hmgui, pos, size);
                }
                WidgetItem::Text(text) => {
                    let bottom = pos.y + self.min_size.y
                        - (self.border_width * 2.0 + self.margin_upper.y + self.margin_lower.y);

                    text.draw(&mut hmgui.renderer, pos.x, size.x, bottom);
                }
                WidgetItem::Rect => {}
                WidgetItem::Shape(shape) => {
                    shape.draw(&hmgui.looks, &mut hmgui.renderer, pos, size);
                }
                WidgetItem::Image(image) => {
                    image.draw(hmgui, pos, size);
                }
                WidgetItem::TextView(image) => {
                    let image = image
                        .as_ref()
                        .expect("Expected TextView to have a valid image.");

                    image.draw(hmgui, pos, size);

                    if hmgui.in_focus(self) {
                        // draw cursor (only where the player can type)
                        let data = hmgui.data_mut(self.hash);
                        let text_view =
                            data.text_view.as_mut().expect("Text view data was not set");
                        let editable = text_view.is_editable();
                        let cursor_rect = text_view.data().cursor_rect();
                        let cursor_size = cursor_rect.size();

                        if editable && cursor_size.x > 0.0 && cursor_size.y > 0.0 {
                            let cursor_pos = self.inner_pos + cursor_rect.pos();
                            let color = cursor_rect.color();

                            hmgui.renderer.rect(cursor_pos, cursor_size, color, None);
                        }
                    }
                }
            }
        }
    }

    fn draw_background(&self, hmgui: &mut HmGui, pos: Vec2, size: Vec2) {
        if self.background_color.is_opaque() || self.opacity > 0.0 {
            hmgui
                .renderer
                .panel(pos, size, self.background_color, 0.0, self.opacity);
        }
    }

    // For testing.
    #[allow(dead_code)]
    #[rustfmt::skip]
    pub(crate) fn dump(&self, title: &str, ident: usize) {
        let ident_str = IDENT.repeat(ident).to_string();

        println!("{ident_str}=== {title} ===");
        println!("{ident_str}{}:", self.item.name());
        println!("{ident_str}{IDENT}- pos:              {:?}", self.pos);
        println!("{ident_str}{IDENT}- size:             {:?}", self.size);
        println!("{ident_str}{IDENT}- inner_pos:        {:?}", self.inner_pos);
        println!("{ident_str}{IDENT}- inner_size:       {:?}", self.inner_size);
        println!("{ident_str}{IDENT}- default_size:     {:?}", self.default_size);
        println!("{ident_str}{IDENT}- alignment:        {:?}", self.alignment);
        println!("{ident_str}{IDENT}- margin_upper:     {:?}", self.margin_upper);
        println!("{ident_str}{IDENT}- margin_lower:     {:?}", self.margin_lower);
        println!("{ident_str}{IDENT}- border_width:     {}",   self.border_width);
        println!("{ident_str}{IDENT}- scale:            {} (own {})", self.scale, self.own_scale);
        println!("{ident_str}{IDENT}- background_color: {:?}", self.background_color);
        println!("{ident_str}{IDENT}- opacity:          {}", self.opacity);
        println!("{ident_str}{IDENT}- min_size:         {:?}", self.min_size);
        println!("{ident_str}{IDENT}- inner_min_size:   {:?}", self.inner_min_size);
        println!("{ident_str}{IDENT}- hash:             0x{:X?}", self.hash);
        println!("{ident_str}{IDENT}- mouse_over:       {:?}", self.mouse_over);
        println!("{ident_str}{IDENT}# item: {}", self.item.name());

        match &self.item {
            WidgetItem::Container(item) => item.dump(ident + 1),
            WidgetItem::Text(item) => item.dump(ident + 1),
            WidgetItem::Rect => {
                let ident_str = crate::ui::hmgui::IDENT.repeat(ident+1).to_string();

                println!("{ident_str}- rect");
            },
            WidgetItem::Image(item) => item.dump(ident + 1),
            WidgetItem::TextView(item) => match item {
                Some(item) => item.dump(ident + 1),
                None => println!("{ident_str}- image: None")
            },
            WidgetItem::Shape(item) => item.dump(ident + 1),
        }
    }
}

#[inline]
fn contains_point(pos: Vec2, size: Vec2, point: Vec2) -> bool {
    pos.x <= point.x && pos.y <= point.y && point.x <= pos.x + size.x && point.y <= pos.y + size.y
}
