use crate::common::*;
use crate::input::*;
use crate::math::*;
use crate::render::*;
use crate::system::{Hash_FNV64_Incremental, Hash_FNV64_Init, Profiler_Begin, Profiler_End};

use super::*;

pub struct HmGui {
    pub(super) renderer: UIRenderer,

    /// Current active group
    group: Option<Rf<HmGuiWidget>>,
    /// Top level group object. Used for recalculating sizes, layouts and drawing of the whole gui
    root: Option<Rf<HmGuiWidget>>,
    /// Either last created/initialized widget (group, image, text, rect) or the last widget of the ended group
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
            group: None,
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
            parent: self.group.clone(),
            hash: 0,
            item,
            pos: Default::default(),
            size: Default::default(),
            min_size: Default::default(),
            align: Default::default(),
            stretch: Default::default(),
        };
        let widget_rf = Rf::new(widget);
        let mut widget = widget_rf.as_mut();

        if let Some(parent_rf) = widget.parent.clone() {
            let mut parent = parent_rf.as_mut();
            let parent_hash = parent.hash;
            let WidgetItem::Group(parent_group) = &mut parent.item else {
                unreachable!();
            };

            parent_group.children_hash = (parent_group.children_hash).wrapping_add(1);

            widget.hash = unsafe {
                Hash_FNV64_Incremental(
                    parent_hash,
                    &mut parent_group.children_hash as *mut u32 as *const _,
                    std::mem::size_of::<u32>() as i32,
                )
            };

            parent_group.children.push(widget_rf.clone());
        } else {
            widget.hash = Hash_FNV64_Init();
        }

        self.last = Some(widget_rf.clone());

        widget_rf.clone()
    }

    fn begin_group(&mut self, layout: LayoutType) {
        let spacing = self.styles.last().expect("Style was not set").spacing;

        let group = HmGuiGroup {
            layout,
            spacing,
            max_size: Vec2::new(1e30, 1e30),
            ..Default::default()
        };

        let widget_rf = self.init_widget(WidgetItem::Group(group));
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

        self.group = Some(widget_rf.clone());
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
        let WidgetItem::Group(group) = &widget.item else {
            return;
        };

        if group.clip && self.is_clipped(widget.pos, widget.size, self.focus_pos) {
            return;
        }

        for widget_rf in group.children.iter().rev() {
            self.check_focus(widget_rf.clone());
        }

        for i in 0..self.focus.len() {
            if self.focus[i] == 0
                && group.focusable[i] as i32 != 0
                && widget.pos.x <= self.focus_pos.x
                && widget.pos.y <= self.focus_pos.y
                && self.focus_pos.x <= widget.pos.x + widget.size.x
                && self.focus_pos.y <= widget.pos.y + widget.size.y
            {
                self.focus[i] = widget.hash;
            }
        }
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl HmGui {
    pub fn begin_gui(&mut self, sx: f32, sy: f32, input: &Input) {
        self.group = None;
        self.root = None;
        self.last = None;
        self.activate = input.mouse().is_pressed(MouseControl::Left);

        self.begin_group(LayoutType::None);

        if let Some(widget_rf) = &self.group {
            let mut widget = widget_rf.as_mut();
            let WidgetItem::Group(group) = &mut widget.item else {
                unreachable!()
            };

            group.clip = true;

            widget.pos = Vec2::ZERO;
            widget.size = Vec2::new(sx, sy);
        } else {
            unreachable!();
        }

        self.root = self.group.clone();
    }

    // TODO: do not calculate layout for the widgets that go out of the screen. If possible.
    pub fn end_gui(&mut self, input: &Input) {
        unsafe { Profiler_Begin(c_str!("HmGui_End")) };

        self.end_group();

        if let Some(root_rf) = self.root.clone() {
            let group_rf = {
                let mut root = root_rf.as_mut();

                root.compute_size(self);
                root.layout(self);

                self.focus.fill(0);

                let mouse = input.mouse();

                self.focus_pos = mouse.position();

                root_rf.clone()
            };

            self.check_focus(group_rf);
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

    pub fn begin_group_x(&mut self) {
        self.begin_group(LayoutType::Horizontal);
    }

    pub fn begin_group_y(&mut self) {
        self.begin_group(LayoutType::Vertical);
    }

    pub fn begin_group_stack(&mut self) {
        self.begin_group(LayoutType::Stack);
    }

    pub fn end_group(&mut self) {
        if let Some(widget_rf) = self.group.clone() {
            let widget = widget_rf.as_ref();

            self.last = Some(widget_rf.clone());
            self.group = widget.parent.clone();
        } else {
            unreachable!();
        }
    }

    pub fn begin_scroll(&mut self, max_size: f32) {
        if let Some(widget_rf) = self.group.clone() {
            let mut widget = widget_rf.as_mut();
            let widget_hash = widget.hash;
            let WidgetItem::Group(group) = &mut widget.item else {
                unreachable!()
            };

            self.begin_group_x();
            self.set_stretch(1.0, 1.0);
            group.clip = true;
            self.set_spacing(2.0);

            self.begin_group_y();
            self.set_padding(6.0, 6.0);
            self.set_stretch(1.0, 1.0);

            group.expand = false;
            group.store_size = true;
            group.max_size.y = max_size;

            let data = self.get_data(widget_hash);

            group.offset.y = -data.offset.y;
        } else {
            unreachable!();
        }
    }

    pub fn end_scroll(&mut self, input: &Input) {
        if let Some(widget_rf) = self.group.clone() {
            let widget = widget_rf.as_ref();
            let has_focus = self.group_has_focus(FocusType::Scroll);

            let data = self.get_data(widget.hash);

            if has_focus {
                let scroll_y = input.mouse().value(MouseControl::ScrollY);

                data.offset.y -= 10.0 * scroll_y as f32;
            }

            let max_scroll = f32::max(0.0, data.min_size.y - data.size.y);
            data.offset.y = f32::clamp(data.offset.y, 0.0, max_scroll);

            self.end_group();

            self.begin_group_y();
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

                self.rect(4.0, handle_pos, 0.0, 0.0, 0.0, 0.0);
                self.rect(
                    4.0,
                    handle_size,
                    color_frame.x,
                    color_frame.y,
                    color_frame.z,
                    color_frame.w,
                );
            } else {
                self.rect(4.0, 16.0, 0.0, 0.0, 0.0, 0.0);
            }

            self.end_group();
            self.end_group();
        } else {
            unreachable!();
        }
    }

    pub fn begin_window(&mut self, _title: &str, input: &Input) {
        if let Some(widget_rf) = self.group.clone() {
            self.begin_group_stack();
            self.set_stretch(0.0, 0.0);

            let mouse = input.mouse();
            let has_focus = self.group_has_focus(FocusType::Mouse);

            {
                let mut widget = widget_rf.as_mut();
                let data = self.get_data(widget.hash);

                if has_focus && mouse.is_down(MouseControl::Left) {
                    data.offset.x += mouse.value(MouseControl::DeltaX);
                    data.offset.y += mouse.value(MouseControl::DeltaY);
                }

                widget.pos.x += data.offset.x;
                widget.pos.y += data.offset.y;

                let WidgetItem::Group(group) = &mut widget.item else {
                    unreachable!()
                };
                group.focus_style = FocusStyle::None;
                group.frame_opacity = 0.95;
                group.clip = true;
            }

            self.begin_group_y();
            self.set_padding(8.0, 8.0);
            self.set_stretch(1.0, 1.0);
            // self.text_colored(title, 1.0f, 1.0f, 1.0f, 0.3f);
            // self.set_align(0.5f, 0.0f);
        } else {
            unreachable!();
        }
    }

    pub fn end_window(&mut self) {
        self.end_group();
        self.end_group();
    }

    pub fn button(&mut self, label: &str) -> bool {
        if let Some(widget_rf) = self.group.clone() {
            self.begin_group_stack();

            {
                let mut widget = widget_rf.as_mut();
                let WidgetItem::Group(group) = &mut widget.item else {
                    unreachable!()
                };

                group.focus_style = FocusStyle::Fill;
                group.frame_opacity = 0.5;
            }

            let focus: bool = self.group_has_focus(FocusType::Mouse);

            self.set_padding(8.0, 8.0);
            self.text(label);
            self.set_align(0.5, 0.5);

            self.end_group();

            focus && self.activate
        } else {
            unreachable!();
        }
    }

    pub fn checkbox(&mut self, label: &str, mut value: bool) -> bool {
        if let Some(widget_rf) = self.group.clone() {
            self.begin_group_x();

            {
                let mut widget = widget_rf.as_mut();
                let WidgetItem::Group(group) = &mut widget.item else {
                    unreachable!()
                };

                group.focus_style = FocusStyle::Underline;
            }

            if self.group_has_focus(FocusType::Mouse) as i32 != 0 && self.activate as i32 != 0 {
                value = !value;
            }

            self.set_padding(4.0, 4.0);
            self.set_spacing(8.0);
            self.set_stretch(1.0, 0.0);

            self.text(label);
            self.set_align(0.0, 0.5);
            self.set_stretch(1.0, 0.0);

            self.begin_group_stack();

            let (color_frame, color_primary) = {
                let style = self.styles.last().expect("Style was not set");
                (style.color_frame, style.color_primary)
            };

            self.rect(
                16.0,
                16.0,
                color_frame.x,
                color_frame.y,
                color_frame.z,
                color_frame.w,
            );

            if value {
                self.rect(
                    10.0,
                    10.0,
                    color_primary.x,
                    color_primary.y,
                    color_primary.z,
                    color_primary.w,
                );
                self.set_align(0.5, 0.5);
            }

            self.end_group();
            self.set_stretch(0.0, 0.0);
            self.end_group();

            value
        } else {
            unreachable!();
        }
    }

    pub fn slider(&mut self, _lower: f32, _upper: f32, _value: f32) -> f32 {
        self.begin_group_stack();
        self.rect(0.0, 2.0, 0.5, 0.5, 0.5, 1.0);
        self.set_align(0.5, 0.5);
        self.set_stretch(1.0, 0.0);
        self.end_group();

        0.0
    }

    pub fn image(&mut self, image: &mut Tex2D) {
        let image_item = HmGuiImage { image };

        let widget_rf = self.init_widget(WidgetItem::Image(image_item));
        let mut widget = widget_rf.as_mut();

        widget.stretch = Vec2::ONE;
    }

    pub fn rect(&mut self, sx: f32, sy: f32, r: f32, g: f32, b: f32, a: f32) {
        let rect_item = HmGuiRect {
            color: Vec4::new(r, g, b, a),
        };

        let widget_rf = self.init_widget(WidgetItem::Rect(rect_item));
        let mut widget = widget_rf.as_mut();

        widget.min_size = Vec2::new(sx, sy);
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

            widget.min_size = Vec2::new(size.x as f32, size.y as f32);
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

            widget.min_size = Vec2::new(size.x as f32, size.y as f32);
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

            widget.min_size = Vec2::new(size.x as f32, size.y as f32);
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

    pub fn set_padding(&self, px: f32, py: f32) {
        if let Some(widget_rf) = self.group.clone() {
            let mut widget = widget_rf.as_mut();
            let WidgetItem::Group(group) = &mut widget.item else {
                unreachable!()
            };

            group.padding_lower = Vec2::new(px, py);
            group.padding_upper = Vec2::new(px, py);
        } else {
            unreachable!();
        }
    }

    pub fn set_padding_ex(&self, left: f32, top: f32, right: f32, bottom: f32) {
        if let Some(widget_rf) = self.group.clone() {
            let mut widget = widget_rf.as_mut();
            let WidgetItem::Group(group) = &mut widget.item else {
                unreachable!()
            };

            group.padding_lower = Vec2::new(left, top);
            group.padding_upper = Vec2::new(right, bottom);
        } else {
            unreachable!();
        }
    }

    pub fn set_padding_left(&self, padding: f32) {
        if let Some(widget_rf) = self.group.clone() {
            let mut widget = widget_rf.as_mut();
            let WidgetItem::Group(group) = &mut widget.item else {
                unreachable!()
            };

            group.padding_lower.x = padding;
        } else {
            unreachable!();
        }
    }

    pub fn set_padding_top(&self, padding: f32) {
        if let Some(widget_rf) = self.group.clone() {
            let mut widget = widget_rf.as_mut();
            let WidgetItem::Group(group) = &mut widget.item else {
                unreachable!()
            };

            group.padding_lower.y = padding;
        } else {
            unreachable!();
        }
    }

    pub fn set_padding_right(&self, padding: f32) {
        if let Some(widget_rf) = self.group.clone() {
            let mut widget = widget_rf.as_mut();
            let WidgetItem::Group(group) = &mut widget.item else {
                unreachable!()
            };

            group.padding_upper.x = padding;
        } else {
            unreachable!();
        }
    }

    pub fn set_padding_bottom(&self, padding: f32) {
        if let Some(widget_rf) = self.group.clone() {
            let mut widget = widget_rf.as_mut();
            let WidgetItem::Group(group) = &mut widget.item else {
                unreachable!()
            };

            group.padding_upper.y = padding;
        } else {
            unreachable!();
        }
    }

    pub fn set_spacing(&self, spacing: f32) {
        if let Some(widget_rf) = self.group.clone() {
            let mut widget = widget_rf.as_mut();
            let WidgetItem::Group(group) = &mut widget.item else {
                unreachable!()
            };

            group.spacing = spacing;
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

    pub fn group_has_focus(&self, ty: FocusType) -> bool {
        if let Some(widget_rf) = self.group.clone() {
            let mut widget = widget_rf.as_mut();
            let WidgetItem::Group(group) = &mut widget.item else {
                unreachable!()
            };

            group.focusable[ty as usize] = true;

            self.focus[ty as usize] == widget.hash
        } else {
            unreachable!();
        }
    }

    pub fn push_style(&mut self) {
        let style = self
            .styles
            .last()
            .cloned()
            .expect("Styles stack is empty".into());

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

    pub fn dump_widgets(&self, file_name: &str) {
        let file_path = PathBuf::new().join(file_name);

        if file_path.exists() {
            return;
        }

        let mut file = File::create(file_path).expect(&format!("Cannot create {file_name}"));

        println!("Widgets:");

        if let Some(group_opt) = &self.root {
            let group = group_opt.as_ref();

            group.dump(1, &mut file);
        } else {
            println!("{IDENT}No widgets");
        }
    }
}
