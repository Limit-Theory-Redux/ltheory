use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

use crate::common::*;
use crate::input::*;
use crate::math::*;
use crate::render::*;
use crate::system::{Hash_FNV64_Incremental, Hash_FNV64_Init, Profiler_Begin, Profiler_End};

use super::*;

pub struct HmGui {
    pub(super) renderer: UIRenderer,

    /// Current active container
    container: Option<Rf<HmGuiWidget>>,
    /// Top level container object. Used for recalculating sizes, layouts and drawing of the whole gui
    root: Option<Rf<HmGuiWidget>>,
    /// Either last created/initialized widget (container, image, text, rect) or the last widget of the ended container
    last: Option<Rf<HmGuiWidget>>,

    styles: Vec<HmGuiStyle>,
    data: HashMap<u64, HmGuiData>,
    focus: [u64; 2],
    focus_pos: Vec2,
    activate: bool,
}

impl HmGui {
    pub fn new(default_font: Font) -> Self {
        let style = HmGuiStyle {
            font: default_font.into(),
            spacing: 6.0,
            color_primary: Vec4::new(0.1, 0.5, 1.0, 1.0),
            color_frame: Vec4::new(0.1, 0.1, 0.1, 0.5),
            color_text: Vec4::ONE,
        };

        Self {
            renderer: Default::default(),
            container: None,
            root: None,
            last: None,
            styles: vec![style],
            data: HashMap::with_capacity(128),
            focus: [0; 2],
            focus_pos: Vec2::ZERO,
            activate: false,
        }
    }

    pub fn mouse_focus_hash(&self) -> u64 {
        self.focus[FocusType::Mouse as usize]
    }

    fn init_widget(&mut self, item: WidgetItem) -> Rf<HmGuiWidget> {
        let widget = HmGuiWidget {
            parent: self.container.clone(),

            hash: 0,
            item,
            pos: Default::default(),
            size: Default::default(),
            inner_pos: Default::default(),
            inner_size: Default::default(),

            default_width: Default::default(),
            default_height: Default::default(),
            docking: Default::default(),
            margin_upper: Default::default(),
            margin_lower: Default::default(),
            bg_color: Default::default(),
            border_width: Default::default(),
            border_color: Default::default(),

            min_size: Default::default(),
            inner_min_size: Default::default(),

            align: Default::default(),
            stretch: Default::default(),
        };
        let widget_rf = Rf::new(widget);
        let mut widget = widget_rf.as_mut();

        if let Some(parent_rf) = widget.parent.clone() {
            let mut parent = parent_rf.as_mut();
            let parent_hash = parent.hash;
            let WidgetItem::Container(parent_container) = &mut parent.item else {
                unreachable!();
            };

            parent_container.children_hash = (parent_container.children_hash).wrapping_add(1);

            widget.hash = unsafe {
                Hash_FNV64_Incremental(
                    parent_hash,
                    &mut parent_container.children_hash as *mut u32 as *const _,
                    std::mem::size_of::<u32>() as i32,
                )
            };

            parent_container.children.push(widget_rf.clone());
        } else {
            widget.hash = Hash_FNV64_Init();
        }

        self.last = Some(widget_rf.clone());

        widget_rf.clone()
    }

    fn begin_container(&mut self, layout: LayoutType) {
        let spacing = self.styles.last().expect("Style was not set").spacing;

        let container = HmGuiContainer {
            layout,
            spacing,
            max_size: Vec2::new(1e30, 1e30),
            ..Default::default()
        };

        let widget_rf = self.init_widget(WidgetItem::Container(container));
        let mut widget = widget_rf.as_mut();

        match layout {
            LayoutType::None => {}
            LayoutType::Stack => {
                widget.stretch = Vec2::ONE;
            }
            LayoutType::Vertical => {
                widget.stretch = Vec2::X;
            }
            LayoutType::Horizontal => {
                widget.stretch = Vec2::Y;
            }
        };

        self.container = Some(widget_rf.clone());
    }

    pub fn get_data(&mut self, widget_hash: u64) -> &mut HmGuiData {
        self.data.entry(widget_hash).or_insert(HmGuiData {
            offset: Vec2::ZERO,
            min_size: Vec2::ZERO,
            size: Vec2::ZERO,
        })
    }

    #[inline]
    fn is_clipped(&self, pos: Vec2, size: Vec2, p: Vec2) -> bool {
        p.x < pos.x || p.y < pos.y || pos.x + size.x < p.x || pos.y + size.y < p.y
    }

    fn check_focus(&mut self, widget_rf: Rf<HmGuiWidget>) {
        let widget = widget_rf.as_ref();
        let WidgetItem::Container(container) = &widget.item else {
            return;
        };

        if container.clip && self.is_clipped(widget.pos, widget.size, self.focus_pos) {
            return;
        }

        for widget_rf in container.children.iter().rev() {
            self.check_focus(widget_rf.clone());
        }

        for i in 0..self.focus.len() {
            if self.focus[i] == 0
                && container.focusable[i] as i32 != 0
                && widget.pos.x <= self.focus_pos.x
                && widget.pos.y <= self.focus_pos.y
                && self.focus_pos.x <= widget.pos.x + widget.size.x
                && self.focus_pos.y <= widget.pos.y + widget.size.y
            {
                self.focus[i] = widget.hash;
            }
        }
    }

    pub fn root(&self) -> Option<Rf<HmGuiWidget>> {
        self.root.clone()
    }

    pub fn dump_widgets(&self, file_name: Option<&str>) {
        let mut file: Option<File> = file_name
            .filter(|file_name| PathBuf::new().join(file_name).exists())
            .map(|file_name| {
                let file_path = PathBuf::new().join(file_name);

                File::create(file_path).expect(&format!("Cannot create {file_name}"))
            });

        println!("Widgets:");

        if let Some(container_opt) = &self.root {
            let container = container_opt.as_ref();

            container.dump(1, &mut file);
        } else {
            println!("{IDENT}No widgets");
        }
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl HmGui {
    pub fn begin_gui(&mut self, sx: f32, sy: f32, input: &Input) {
        self.container = None;
        self.root = None;
        self.last = None;
        self.activate = input.mouse().is_pressed(MouseControl::Left);

        self.begin_container(LayoutType::None);

        if let Some(widget_rf) = &self.container {
            let mut widget = widget_rf.as_mut();
            let WidgetItem::Container(container) = &mut widget.item else {
                unreachable!()
            };

            container.clip = true;

            widget.inner_pos = Vec2::ZERO;
            widget.pos = widget.inner_pos;
            widget.inner_size = Vec2::new(sx, sy);
            widget.size = widget.inner_size;
        } else {
            unreachable!();
        }

        self.root = self.container.clone();
    }

    // TODO: do not calculate layout for the widgets that go out of the screen. If possible.
    pub fn end_gui(&mut self, input: &Input) {
        unsafe { Profiler_Begin(c_str!("HmGui_End")) };

        self.end_container();

        if let Some(root_rf) = self.root.clone() {
            let container_rf = {
                let mut root = root_rf.as_mut();

                root.compute_size(self);
                root.layout(self);

                self.focus.fill(0);

                let mouse = input.mouse();

                self.focus_pos = mouse.position();

                root_rf.clone()
            };

            self.check_focus(container_rf);
        } else {
            unreachable!();
        }

        unsafe { Profiler_End() };
    }

    // TODO: optimize - clip by window screen - do not draw anything that goes out of the screen
    pub fn draw(&mut self) {
        if let Some(root_rf) = self.root.clone() {
            unsafe {
                Profiler_Begin(c_str!("HmGui_Draw"));

                RenderState_PushBlendMode(1);
            }

            self.renderer.begin();

            root_rf.as_ref().draw(self);

            self.renderer.end();

            unsafe { RenderState_PopBlendMode() };

            self.renderer.draw();

            unsafe { Profiler_End() };
        } else {
            unreachable!();
        }
    }

    pub fn begin_horizontal_container(&mut self) {
        self.begin_container(LayoutType::Horizontal);
    }

    pub fn begin_vertical_container(&mut self) {
        self.begin_container(LayoutType::Vertical);
    }

    pub fn begin_stack_container(&mut self) {
        self.begin_container(LayoutType::Stack);
    }

    pub fn end_container(&mut self) {
        if let Some(widget_rf) = self.container.clone() {
            let widget = widget_rf.as_ref();

            self.last = Some(widget_rf.clone());
            self.container = widget.parent.clone();
        } else {
            unreachable!();
        }
    }

    pub fn begin_scroll(&mut self, max_size: f32) {
        if let Some(widget_rf) = self.container.clone() {
            let mut widget = widget_rf.as_mut();
            let widget_hash = widget.hash;
            let WidgetItem::Container(container) = &mut widget.item else {
                unreachable!()
            };

            self.begin_horizontal_container();
            self.set_stretch(1.0, 1.0);
            container.clip = true;
            self.set_spacing(2.0);

            self.begin_vertical_container();
            self.set_padding(6.0, 6.0);
            self.set_stretch(1.0, 1.0);

            container.store_size = true;
            container.max_size.y = max_size;

            let data = self.get_data(widget_hash);

            container.offset.y = -data.offset.y;
        } else {
            unreachable!();
        }
    }

    pub fn end_scroll(&mut self, input: &Input) {
        if let Some(widget_rf) = self.container.clone() {
            let widget = widget_rf.as_ref();
            let has_focus = self.container_has_focus(FocusType::Scroll);

            let data = self.get_data(widget.hash);

            if has_focus {
                let scroll_y = input.mouse().value(MouseControl::ScrollY);

                data.offset.y -= 10.0 * scroll_y as f32;
            }

            let max_scroll = f32::max(0.0, data.min_size.y - data.size.y);
            data.offset.y = f32::clamp(data.offset.y, 0.0, max_scroll);

            self.end_container();

            self.begin_vertical_container();
            self.set_stretch(0.0, 1.0);
            self.set_spacing(0.0);

            if max_scroll > 0.0 {
                let data = self.get_data(widget.hash);
                let handle_size = data.size.y * (data.size.y / data.min_size.y);
                let handle_pos = Lerp(
                    0.0f64,
                    (data.size.y - handle_size) as f64,
                    (data.offset.y / max_scroll) as f64,
                ) as f32;
                let color_frame = self.styles.last().expect("Style was not set").color_frame;

                self.rect(0.0, 0.0, 0.0, 0.0);
                self.set_fixed_size(4.0, handle_pos);
                self.rect(color_frame.x, color_frame.y, color_frame.z, color_frame.w);
                self.set_fixed_size(4.0, handle_size);
            } else {
                self.rect(0.0, 0.0, 0.0, 0.0);
                self.set_fixed_size(4.0, 16.0);
            }

            self.end_container();
            self.end_container();
        } else {
            unreachable!();
        }
    }

    pub fn begin_window(&mut self, _title: &str, input: &Input) {
        if let Some(widget_rf) = self.container.clone() {
            self.begin_stack_container();
            self.set_stretch(0.0, 0.0);

            let mouse = input.mouse();
            let has_focus = self.container_has_focus(FocusType::Mouse);

            {
                let mut widget = widget_rf.as_mut();
                let data = self.get_data(widget.hash);

                if has_focus && mouse.is_down(MouseControl::Left) {
                    data.offset.x += mouse.value(MouseControl::DeltaX);
                    data.offset.y += mouse.value(MouseControl::DeltaY);
                }

                widget.pos.x += data.offset.x;
                widget.pos.y += data.offset.y;

                let WidgetItem::Container(container) = &mut widget.item else {
                    unreachable!()
                };
                container.focus_style = FocusStyle::None;
                container.frame_opacity = 0.95;
                container.clip = true;
            }

            self.begin_vertical_container();
            self.set_padding(8.0, 8.0);
            self.set_stretch(1.0, 1.0);
            // self.text_colored(title, 1.0f, 1.0f, 1.0f, 0.3f);
            // self.set_align(0.5f, 0.0f);
        } else {
            unreachable!();
        }
    }

    pub fn end_window(&mut self) {
        self.end_container();
        self.end_container();
    }

    pub fn button(&mut self, label: &str) -> bool {
        if let Some(widget_rf) = self.container.clone() {
            self.begin_stack_container();

            {
                let mut widget = widget_rf.as_mut();
                let WidgetItem::Container(container) = &mut widget.item else {
                    unreachable!()
                };

                container.focus_style = FocusStyle::Fill;
                container.frame_opacity = 0.5;
            }

            let focus: bool = self.container_has_focus(FocusType::Mouse);

            self.set_padding(8.0, 8.0);
            self.text(label);
            self.set_align(0.5, 0.5);

            self.end_container();

            focus && self.activate
        } else {
            unreachable!();
        }
    }

    pub fn checkbox(&mut self, label: &str, mut value: bool) -> bool {
        if let Some(widget_rf) = self.container.clone() {
            self.begin_horizontal_container();

            {
                let mut widget = widget_rf.as_mut();
                let WidgetItem::Container(container) = &mut widget.item else {
                    unreachable!()
                };

                container.focus_style = FocusStyle::Underline;
            }

            if self.container_has_focus(FocusType::Mouse) as i32 != 0 && self.activate as i32 != 0 {
                value = !value;
            }

            self.set_padding(4.0, 4.0);
            self.set_spacing(8.0);
            self.set_stretch(1.0, 0.0);

            self.text(label);
            self.set_align(0.0, 0.5);
            self.set_stretch(1.0, 0.0);

            self.begin_stack_container();

            let (color_frame, color_primary) = {
                let style = self.styles.last().expect("Style was not set");
                (style.color_frame, style.color_primary)
            };

            self.rect(color_frame.x, color_frame.y, color_frame.z, color_frame.w);
            self.set_fixed_size(16.0, 16.0);

            if value {
                self.rect(
                    color_primary.x,
                    color_primary.y,
                    color_primary.z,
                    color_primary.w,
                );
                self.set_fixed_size(10.0, 10.0);
                self.set_align(0.5, 0.5);
            }

            self.end_container();
            self.set_stretch(0.0, 0.0);
            self.end_container();

            value
        } else {
            unreachable!();
        }
    }

    pub fn slider(&mut self, _lower: f32, _upper: f32, _value: f32) -> f32 {
        self.begin_stack_container();
        self.rect(0.5, 0.5, 0.5, 1.0);
        self.set_fixed_size(0.0, 2.0);
        self.set_align(0.5, 0.5);
        self.set_stretch(1.0, 0.0);
        self.end_container();

        0.0
    }

    pub fn image(&mut self, image: &mut Tex2D) {
        let image_item = HmGuiImage { image };

        let widget_rf = self.init_widget(WidgetItem::Image(image_item));
        let mut widget = widget_rf.as_mut();

        widget.stretch = Vec2::ONE;
    }

    pub fn rect(&mut self, r: f32, g: f32, b: f32, a: f32) {
        let rect_item = HmGuiRect {
            color: Vec4::new(r, g, b, a),
        };

        self.init_widget(WidgetItem::Rect(rect_item));
    }

    pub fn text(&mut self, text: &str) {
        let style = self.styles.last().expect("Style was not set");

        // NOTE: cannot call text_ex() here because of mutable/immutable borrow conflict
        let item = HmGuiText {
            font: style.font.clone().into(),
            text: text.into(),
            color: style.color_text,
        };
        let size = item.font.get_size2(text);

        // NOTE: This scope is needed to prevent widget be mut borrowed twice here and in set_align below
        {
            let widget_rf = self.init_widget(WidgetItem::Text(item));
            let mut widget = widget_rf.as_mut();

            widget.inner_min_size = Vec2::new(size.x as f32, size.y as f32);
        }

        self.set_align(0.0, 1.0);
    }

    pub fn text_colored(&mut self, text: &str, r: f32, g: f32, b: f32, a: f32) {
        let style = self.styles.last().expect("Style was not set");

        // NOTE: cannot call text_ex() here because of mutable/immutable borrow conflict
        let item = HmGuiText {
            font: style.font.clone().into(),
            text: text.into(),
            color: Vec4::new(r, g, b, a),
        };
        let size = item.font.get_size2(text);

        // NOTE: This scope is needed to prevent widget be mut borrowed twice here and in set_align below
        {
            let widget_rf = self.init_widget(WidgetItem::Text(item));
            let mut widget = widget_rf.as_mut();

            widget.inner_min_size = Vec2::new(size.x as f32, size.y as f32);
        }

        self.set_align(0.0, 1.0);
    }

    pub fn text_ex(&mut self, font: &Font, text: &str, r: f32, g: f32, b: f32, a: f32) {
        let item = HmGuiText {
            font: font.clone().into(),
            text: text.into(),
            color: Vec4::new(r, g, b, a),
        };
        let size = item.font.get_size2(text);

        // NOTE: This scope is needed to prevent widget to be mut borrowed twice here and in set_align below
        {
            let widget_rf = self.init_widget(WidgetItem::Text(item));
            let mut widget = widget_rf.as_mut();

            widget.inner_min_size = Vec2::new(size.x as f32, size.y as f32);
        }

        self.set_align(0.0, 1.0);
    }

    pub fn set_align(&self, ax: f32, ay: f32) {
        if let Some(widget_rf) = &self.last {
            let mut widget = widget_rf.as_mut();

            widget.align = Vec2::new(ax, ay);
        } else {
            unreachable!();
        }
    }

    pub fn set_fixed_width(&self, width: f32) {
        if let Some(widget_rf) = &self.last {
            let mut widget = widget_rf.as_mut();

            widget.default_width = Some(Length::Fixed(width));
        } else {
            unreachable!();
        }
    }

    pub fn set_fixed_height(&self, height: f32) {
        if let Some(widget_rf) = &self.last {
            let mut widget = widget_rf.as_mut();

            widget.default_height = Some(Length::Fixed(height));
        } else {
            unreachable!();
        }
    }

    pub fn set_fixed_size(&self, width: f32, height: f32) {
        if let Some(widget_rf) = &self.last {
            let mut widget = widget_rf.as_mut();

            widget.default_width = Some(Length::Fixed(width));
            widget.default_height = Some(Length::Fixed(height));
        } else {
            unreachable!();
        }
    }

    pub fn set_percent_width(&self, width: f32) {
        if let Some(widget_rf) = &self.last {
            let mut widget = widget_rf.as_mut();

            widget.default_width = Some(Length::Percent(width));
        } else {
            unreachable!();
        }
    }

    pub fn set_percent_height(&self, height: f32) {
        if let Some(widget_rf) = &self.last {
            let mut widget = widget_rf.as_mut();

            widget.default_height = Some(Length::Percent(height));
        } else {
            unreachable!();
        }
    }

    pub fn set_percent_size(&self, width: f32, height: f32) {
        if let Some(widget_rf) = &self.last {
            let mut widget = widget_rf.as_mut();

            widget.default_width = Some(Length::Percent(width));
            widget.default_height = Some(Length::Percent(height));
        } else {
            unreachable!();
        }
    }

    pub fn set_margin(&self, px: f32, py: f32) {
        if let Some(widget_rf) = self.last.clone() {
            let mut widget = widget_rf.as_mut();

            widget.margin_lower = Vec2::new(px, py);
            widget.margin_upper = Vec2::new(px, py);
        } else {
            unreachable!();
        }
    }

    pub fn set_margin_ex(&self, left: f32, top: f32, right: f32, bottom: f32) {
        if let Some(widget_rf) = self.last.clone() {
            let mut widget = widget_rf.as_mut();

            widget.margin_lower = Vec2::new(left, top);
            widget.margin_upper = Vec2::new(right, bottom);
        } else {
            unreachable!();
        }
    }

    pub fn set_margin_left(&self, margin: f32) {
        if let Some(widget_rf) = self.last.clone() {
            let mut widget = widget_rf.as_mut();

            widget.margin_lower.x = margin;
        } else {
            unreachable!();
        }
    }

    pub fn set_margin_top(&self, margin: f32) {
        if let Some(widget_rf) = self.last.clone() {
            let mut widget = widget_rf.as_mut();

            widget.margin_lower.y = margin;
        } else {
            unreachable!();
        }
    }

    pub fn set_margin_right(&self, margin: f32) {
        if let Some(widget_rf) = self.last.clone() {
            let mut widget = widget_rf.as_mut();

            widget.margin_upper.x = margin;
        } else {
            unreachable!();
        }
    }

    pub fn set_margin_bottom(&self, margin: f32) {
        if let Some(widget_rf) = self.last.clone() {
            let mut widget = widget_rf.as_mut();

            widget.margin_upper.y = margin;
        } else {
            unreachable!();
        }
    }

    pub fn set_border_width(&self, width: f32) {
        if let Some(widget_rf) = self.last.clone() {
            let mut widget = widget_rf.as_mut();

            widget.border_width = width;
        } else {
            unreachable!();
        }
    }

    pub fn set_padding(&self, px: f32, py: f32) {
        if let Some(widget_rf) = self.container.clone() {
            let mut widget = widget_rf.as_mut();
            let WidgetItem::Container(container) = &mut widget.item else {
                unreachable!()
            };

            container.padding_lower = Vec2::new(px, py);
            container.padding_upper = Vec2::new(px, py);
        } else {
            unreachable!();
        }
    }

    pub fn set_padding_ex(&self, left: f32, top: f32, right: f32, bottom: f32) {
        if let Some(widget_rf) = self.container.clone() {
            let mut widget = widget_rf.as_mut();
            let WidgetItem::Container(container) = &mut widget.item else {
                unreachable!()
            };

            container.padding_lower = Vec2::new(left, top);
            container.padding_upper = Vec2::new(right, bottom);
        } else {
            unreachable!();
        }
    }

    pub fn set_padding_left(&self, padding: f32) {
        if let Some(widget_rf) = self.container.clone() {
            let mut widget = widget_rf.as_mut();
            let WidgetItem::Container(container) = &mut widget.item else {
                unreachable!()
            };

            container.padding_lower.x = padding;
        } else {
            unreachable!();
        }
    }

    pub fn set_padding_top(&self, padding: f32) {
        if let Some(widget_rf) = self.container.clone() {
            let mut widget = widget_rf.as_mut();
            let WidgetItem::Container(container) = &mut widget.item else {
                unreachable!()
            };

            container.padding_lower.y = padding;
        } else {
            unreachable!();
        }
    }

    pub fn set_padding_right(&self, padding: f32) {
        if let Some(widget_rf) = self.container.clone() {
            let mut widget = widget_rf.as_mut();
            let WidgetItem::Container(container) = &mut widget.item else {
                unreachable!()
            };

            container.padding_upper.x = padding;
        } else {
            unreachable!();
        }
    }

    pub fn set_padding_bottom(&self, padding: f32) {
        if let Some(widget_rf) = self.container.clone() {
            let mut widget = widget_rf.as_mut();
            let WidgetItem::Container(container) = &mut widget.item else {
                unreachable!()
            };

            container.padding_upper.y = padding;
        } else {
            unreachable!();
        }
    }

    pub fn set_spacing(&self, spacing: f32) {
        if let Some(widget_rf) = self.container.clone() {
            let mut widget = widget_rf.as_mut();
            let WidgetItem::Container(container) = &mut widget.item else {
                unreachable!()
            };

            println!("Gui::set_spacing({spacing}): layout={:?}", container.layout);

            container.spacing = spacing;
        } else {
            unreachable!();
        }
    }

    pub fn set_stretch(&self, x: f32, y: f32) {
        if let Some(widget_rf) = &self.last {
            let mut widget = widget_rf.as_mut();

            widget.stretch = Vec2::new(x, y);
        } else {
            unreachable!();
        }
    }

    pub fn set_docking(&self, docking: u8) {
        if let Some(widget_rf) = &self.last {
            let mut widget = widget_rf.as_mut();

            widget.docking = docking.into();
        } else {
            unreachable!();
        }
    }

    pub fn container_has_focus(&self, ty: FocusType) -> bool {
        if let Some(widget_rf) = self.container.clone() {
            let mut widget = widget_rf.as_mut();
            let WidgetItem::Container(container) = &mut widget.item else {
                unreachable!()
            };

            container.focusable[ty as usize] = true;

            self.focus[ty as usize] == widget.hash
        } else {
            unreachable!();
        }
    }

    pub fn set_children_docking(&self, docking: u8) {
        if let Some(widget_rf) = self.container.clone() {
            let mut widget = widget_rf.as_mut();
            let WidgetItem::Container(container) = &mut widget.item else {
                unreachable!()
            };

            container.children_docking = docking.into();
        } else {
            unreachable!();
        }
    }

    pub fn push_style(&mut self) {
        let style = self.styles.last().cloned().unwrap_or_default();

        self.styles.push(style);
    }

    pub fn push_font(&mut self, font: &Font) {
        self.push_style();

        let style = self.styles.last_mut().expect("Style was not set");

        style.font = font.clone().into();
    }

    pub fn push_text_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.push_style();

        let style = self.styles.last_mut().expect("Style was not set");

        style.color_text = Vec4::new(r, g, b, a);
    }

    pub fn pop_style(&mut self, depth: i32) {
        assert!(self.styles.len() >= depth as usize);

        self.styles.truncate(self.styles.len() - depth as usize);
    }
}
