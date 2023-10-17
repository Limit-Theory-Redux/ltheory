mod data;
mod focus;
mod group;
mod image;
mod rect;
mod style;
mod text;
mod widget;

use std::collections::HashMap;
use std::ffi::CString;

use internal::*;

use self::data::*;
use self::focus::*;
use self::group::*;
use self::image::*;
use self::rect::*;
use self::style::*;
use self::text::*;
use self::widget::*;

use super::*;
use crate::common::*;
use crate::input::*;
use crate::math::*;
use crate::render::*;
use crate::system::{Hash_FNV64_Incremental, Hash_FNV64_Init, Profiler_Begin, Profiler_End};
use crate::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HmGuiGroupId(usize);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HmGuiWidgetId(usize);

#[derive(Clone)]
pub struct HmGui {
    groups: Vec<HmGuiGroup>,
    widgets: Vec<HmGuiWidget>,

    /// Current active group
    pub group_id: Option<HmGuiGroupId>,
    /// Top level group object. Used for recalculating sizes, layouts and drawing of the whole gui
    pub root_id: Option<HmGuiGroupId>,
    /// Either last created/initialized widget (group, image, text, rect) or the last widget of the ended group
    pub last_id: Option<HmGuiWidgetId>,

    pub styles: Vec<HmGuiStyle>,
    pub data: HashMap<u64, HmGuiData>,
    pub focus: [u64; 2],
    pub focusPos: Vec2,
    pub activate: bool,
}

impl HmGui {
    pub fn new() -> Self {
        let style = HmGuiStyle {
            font: unsafe { Font_Load(c_str!("Rajdhani"), 14) },
            spacing: 6.0f32,
            colorPrimary: Vec4::new(0.1f32, 0.5f32, 1.0f32, 1.0f32),
            colorFrame: Vec4::new(0.1f32, 0.1f32, 0.1f32, 0.5f32),
            colorText: Vec4::ONE,
        };

        Self {
            groups: vec![],
            widgets: vec![],
            group_id: None,
            root_id: None,
            last_id: None,
            styles: vec![style],
            data: HashMap::with_capacity(128),
            focus: [0; 2],
            focusPos: Vec2::ZERO,
            activate: false,
        }
    }

    fn add_group(&mut self, group: HmGuiGroup) -> (HmGuiGroupId, &mut HmGuiGroup) {
        let id = self.groups.len();

        self.groups.push(group);

        (HmGuiGroupId(id), &mut self.groups[id])
    }

    fn get_group(&self, id: HmGuiGroupId) -> &HmGuiGroup {
        &self.groups[id.0]
    }

    fn get_group_mut(&mut self, id: HmGuiGroupId) -> &mut HmGuiGroup {
        &mut self.groups[id.0]
    }

    fn add_widget(&mut self, widget: HmGuiWidget) -> (HmGuiWidgetId, &mut HmGuiWidget) {
        let id = self.widgets.len();

        self.widgets.push(widget);

        (HmGuiWidgetId(id), &mut self.widgets[id])
    }

    fn get_widget(&self, id: HmGuiWidgetId) -> &HmGuiWidget {
        &self.widgets[id.0]
    }

    fn get_widget_mut(&mut self, id: HmGuiWidgetId) -> &mut HmGuiWidget {
        &mut self.widgets[id.0]
    }

    fn init_widget(&mut self, item: WidgetItem) -> &mut HmGuiWidget {
        let widget = HmGuiWidget {
            parent_id: self.group_id,
            next_id: None,
            prev_id: self
                .group_id
                .map(|group_id| self.get_group(group_id).tail_id)
                .flatten(),
            hash: 0,
            item,
            pos: Default::default(),
            size: Default::default(),
            minSize: Default::default(),
            align: Default::default(),
            stretch: Default::default(),
        };
        let (widget_id, widget) = self.add_widget(widget);

        // Set item's widget id to this one
        match &mut widget.item {
            WidgetItem::Group(id) => {
                let group = self.get_group_mut(*id);
                group.widget_id = widget_id;
            }
            WidgetItem::Text(item) => item.widget_id = widget_id,
            WidgetItem::Rect(item) => item.widget_id = widget_id,
            WidgetItem::Image(item) => item.widget_id = widget_id,
        }

        if let Some(parent_id) = widget.parent_id {
            let parent_group = self.get_group_mut(parent_id);

            parent_group.children = (parent_group.children).wrapping_add(1);

            widget.hash = unsafe {
                Hash_FNV64_Incremental(
                    self.get_widget(parent_group.widget_id).hash,
                    &mut parent_group.children as *mut u32 as *const _,
                    std::mem::size_of::<u32>() as i32,
                )
            };

            if let Some(next_id) = widget.next_id {
                self.get_widget_mut(next_id).prev_id = Some(widget_id);
            } else {
                parent_group.tail_id = Some(widget_id);
            }

            if let Some(prev_id) = widget.prev_id {
                self.get_widget_mut(prev_id).next_id = Some(widget_id);
            } else {
                parent_group.head_id = Some(widget_id);
            }
        } else {
            widget.hash = Hash_FNV64_Init();
        }

        self.last_id = Some(widget_id);

        widget
    }

    fn begin_group(&mut self, layout: LayoutType) {
        let spacing = self.styles.last().expect("Style was not set").spacing;

        let group = HmGuiGroup {
            layout,
            maxSize: Vec2::new(1e30f32, 1e30f32),
            ..Default::default()
        };

        let (group_id, group) = self.add_group(group);

        let widget = self.init_widget(WidgetItem::Group(group_id));

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

        self.group_id = Some(group_id);
    }

    pub fn get_data(&mut self, g: &HmGuiGroup) -> &HmGuiData {
        let widget = self.get_widget(g.widget_id);

        self.data.entry(widget.hash).or_insert(HmGuiData {
            offset: Vec2::ZERO,
            minSize: Vec2::ZERO,
            size: Vec2::ZERO,
        })
    }

    #[inline]
    fn is_clipped(&self, group: &HmGuiGroup, p: Vec2) -> bool {
        let widget = self.get_widget(group.widget_id);

        p.x < widget.pos.x
            || p.y < widget.pos.y
            || widget.pos.x + widget.size.x < p.x
            || widget.pos.y + widget.size.y < p.y
    }

    fn check_focus(&mut self, group_id: HmGuiGroupId) {
        let group = self.get_group(group_id);

        if group.clip && self.is_clipped(group, self.focusPos) {
            return;
        }

        let mut widget_id = group.tail_id;
        while let Some(id) = widget_id {
            let widget = self.get_widget(id);

            if let WidgetItem::Group(group_id) = widget.item {
                self.check_focus(group_id);
            }

            widget_id = widget.prev_id;
        }

        let widget = self.get_widget(group.widget_id);
        for i in 0..self.focus.len() {
            if self.focus[i] == 0
                && group.focusable[i] as i32 != 0
                && widget.pos.x <= self.focusPos.x
                && widget.pos.y <= self.focusPos.y
                && self.focusPos.x <= widget.pos.x + widget.size.x
                && self.focusPos.y <= widget.pos.y + widget.size.y
            {
                self.focus[i] = widget.hash;
            }
        }
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl HmGui {
    pub fn begin_gui(&mut self, sx: f32, sy: f32, input: &Input) {
        if self.root_id.is_some() {
            self.groups.clear();
            self.widgets.clear();
            self.root_id = None;
        }
        self.last_id = None;
        self.activate = input.mouse().is_pressed(MouseControl::Left);

        self.begin_group(LayoutType::None);

        if let Some(group_id) = self.group_id {
            let group = self.get_group_mut(group_id);
            let widget = self.get_widget_mut(group.widget_id);

            group.clip = true;
            widget.pos = Vec2::ZERO;
            widget.size = Vec2::new(sx, sy);
        } else {
            unreachable!();
        }

        self.root_id = self.group_id;
    }

    pub fn end_gui(&mut self, input: &Input) {
        unsafe { Profiler_Begin(c_str!("HmGui_End")) };

        self.end_group();

        if let Some(root_id) = self.root_id {
            let root = self.get_group_mut(root_id);

            root.compute_size(self);
            root.layout(self);

            self.focus.fill(0);

            let mouse = input.mouse();

            self.focusPos = mouse.position();

            self.check_focus(root_id);
        } else {
            unreachable!();
        }

        unsafe { Profiler_End() };
    }

    pub fn draw(&mut self) {
        if let Some(root_id) = self.root_id {
            unsafe {
                Profiler_Begin(c_str!("HmGui_Draw"));

                let root = self.get_group_mut(root_id);

                RenderState_PushBlendMode(1);
                UIRenderer_Begin();

                root.draw(self, self.focus[FocusType::Mouse as usize]);

                UIRenderer_End();
                RenderState_PopBlendMode();

                UIRenderer_Draw();

                Profiler_End();
            }
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
        if let Some(group_id) = self.group_id {
            let group = self.get_group_mut(group_id);
            let widget = self.get_widget_mut(group.widget_id);

            self.last_id = Some(group.widget_id);
            self.group_id = widget.parent_id;
        } else {
            unreachable!();
        }
    }

    pub fn begin_scroll(&mut self, maxSize: f32) {
        if let Some(group_id) = self.group_id {
            let group = self.get_group_mut(group_id);

            self.begin_group_x();
            self.set_stretch(1.0f32, 1.0f32);
            group.clip = true;
            self.set_spacing(2.0f32);

            self.begin_group_y();
            self.set_padding(6.0f32, 6.0f32);
            self.set_stretch(1.0f32, 1.0f32);

            group.expand = false;
            group.storeSize = true;
            group.maxSize.y = maxSize;

            let data = self.get_data(group);
            group.offset.y = -data.offset.y;
        } else {
            unreachable!();
        }
    }

    pub fn end_scroll(&mut self, input: &Input) {
        if let Some(group_id) = self.group_id {
            let group = self.get_group_mut(group_id);
            let data = self.get_data(group);

            if self.group_has_focus(FocusType::Scroll) {
                let scroll_y = input.mouse().value(MouseControl::ScrollY);

                data.offset.y -= 10.0f32 * scroll_y as f32;
            }

            let maxScroll: f32 = f32::max(0.0f32, data.minSize.y - data.size.y);
            data.offset.y = f32::clamp(data.offset.y, 0.0f32, maxScroll);

            self.end_group();

            self.begin_group_y();
            self.set_stretch(0.0f32, 1.0f32);
            self.set_spacing(0.0f32);

            if maxScroll > 0.0f32 {
                let handleSize: f32 = data.size.y * (data.size.y / data.minSize.y);
                let handlePos: f32 = Lerp(
                    0.0f64,
                    (data.size.y - handleSize) as f64,
                    (data.offset.y / maxScroll) as f64,
                ) as f32;
                let colorFrame = self.styles.last().expect("Style was not set").colorFrame;

                self.rect(4.0f32, handlePos, 0.0f32, 0.0f32, 0.0f32, 0.0f32);
                self.rect(
                    4.0f32,
                    handleSize,
                    colorFrame.x,
                    colorFrame.y,
                    colorFrame.z,
                    colorFrame.w,
                );
            } else {
                self.rect(4.0f32, 16.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32);
            }

            self.end_group();
            self.end_group();
        } else {
            unreachable!();
        }
    }

    pub fn begin_window(&mut self, _title: &str, input: &Input) {
        if let Some(group_id) = self.group_id {
            let group = self.get_group_mut(group_id);
            let widget = self.get_widget_mut(group.widget_id);

            self.begin_group_stack();
            self.set_stretch(0.0f32, 0.0f32);

            group.focusStyle = FocusStyle::None;
            group.frameOpacity = 0.95f32;

            let data = self.get_data(group);
            let mouse = input.mouse();

            if self.group_has_focus(FocusType::Mouse) && mouse.is_down(MouseControl::Left) {
                (*data).offset.x += mouse.value(MouseControl::DeltaX);
                (*data).offset.y += mouse.value(MouseControl::DeltaY);
            }

            widget.pos.x += (*data).offset.x;
            widget.pos.y += (*data).offset.y;

            self.begin_group_y();
            group.clip = true;
            self.set_padding(8.0f32, 8.0f32);
            self.set_stretch(1.0f32, 1.0f32);
            // HmGui_TextColored(title, 1.0f, 1.0f, 1.0f, 0.3f);
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
        if let Some(group_id) = self.group_id {
            let group = self.get_group_mut(group_id);

            self.begin_group_stack();

            group.focusStyle = FocusStyle::Fill;
            group.frameOpacity = 0.5f32;

            let focus: bool = self.group_has_focus(FocusType::Mouse);

            self.set_padding(8.0f32, 8.0f32);
            self.text(label);
            self.set_align(0.5f32, 0.5f32);

            self.end_group();

            focus && self.activate
        } else {
            unreachable!();
        }
    }

    pub fn checkbox(&mut self, label: &str, mut value: bool) -> bool {
        if let Some(group_id) = self.group_id {
            let group = self.get_group_mut(group_id);

            self.begin_group_x();

            group.focusStyle = FocusStyle::Underline;

            if self.group_has_focus(FocusType::Mouse) as i32 != 0 && self.activate as i32 != 0 {
                value = !value;
            }

            self.set_padding(4.0f32, 4.0f32);
            self.set_spacing(8.0f32);
            self.set_stretch(1.0f32, 0.0f32);

            self.text(label);
            self.set_align(0.0f32, 0.5f32);
            self.set_stretch(1.0f32, 0.0f32);

            self.begin_group_stack();

            let (colorFrame, colorPrimary) = {
                let style = self.styles.last().expect("Style was not set");

                (style.colorFrame, style.colorPrimary)
            };

            self.rect(
                16.0f32,
                16.0f32,
                colorFrame.x,
                colorFrame.y,
                colorFrame.z,
                colorFrame.w,
            );

            if value {
                self.rect(
                    10.0f32,
                    10.0f32,
                    colorPrimary.x,
                    colorPrimary.y,
                    colorPrimary.z,
                    colorPrimary.w,
                );
                self.set_align(0.5f32, 0.5f32);
            }

            self.end_group();
            self.set_stretch(0.0f32, 0.0f32);
            self.end_group();

            value
        } else {
            unreachable!();
        }
    }

    pub fn slider(&mut self, _lower: f32, _upper: f32, _value: f32) -> f32 {
        self.begin_group_stack();
        self.rect(0.0f32, 2.0f32, 0.5f32, 0.5f32, 0.5f32, 1.0f32);
        self.set_align(0.5f32, 0.5f32);
        self.set_stretch(1.0f32, 0.0f32);
        self.end_group();

        0.0
    }

    pub fn image(&mut self, image: &mut Tex2D) {
        let image_item = HmGuiImage {
            widget_id: Default::default(),
            image,
        };

        let widget = self.init_widget(WidgetItem::Image(image_item));

        widget.stretch = Vec2::ONE;
    }

    pub fn rect(&mut self, sx: f32, sy: f32, r: f32, g: f32, b: f32, a: f32) {
        let rect_item = HmGuiRect {
            widget_id: Default::default(),
            color: Vec4::new(r, g, b, a),
        };

        let widget = self.init_widget(WidgetItem::Rect(rect_item));

        widget.minSize = Vec2::new(sx, sy);
    }

    pub fn text(&mut self, text: &str) {
        let style = self.styles.last().expect("Style was not set");

        self.text_ex(
            unsafe { &mut *style.font },
            text,
            style.colorText.x,
            style.colorText.y,
            style.colorText.z,
            style.colorText.w,
        );
    }

    pub fn text_colored(&mut self, text: &str, r: f32, g: f32, b: f32, a: f32) {
        let style = self.styles.last().expect("Style was not set");

        self.text_ex(unsafe { &mut *style.font }, text, r, g, b, a);
    }

    pub fn text_ex(&mut self, font: &mut Font, text: &str, r: f32, g: f32, b: f32, a: f32) {
        let item = HmGuiText {
            widget_id: Default::default(),
            font,
            text: text.into(),
            color: Vec4::new(r, g, b, a),
        };

        let widget = self.init_widget(WidgetItem::Text(item));

        let mut size = IVec2::ZERO;

        let ctext = CString::new(text).expect("Cannot convert text");

        unsafe { Font_GetSize2(font, &mut size, ctext.as_ptr()) };

        widget.minSize = Vec2::new(size.x as f32, size.y as f32);

        self.set_align(0.0f32, 1.0f32);
    }

    pub fn set_align(&mut self, ax: f32, ay: f32) {
        if let Some(id) = self.last_id {
            let widget = self.get_widget_mut(id);

            widget.align = Vec2::new(ax, ay);
        } else {
            unreachable!();
        }
    }

    pub fn set_padding(&mut self, px: f32, py: f32) {
        if let Some(group_id) = self.group_id {
            let group = self.get_group_mut(group_id);

            group.paddingLower = Vec2::new(px, py);
            group.paddingUpper = Vec2::new(px, py);
        } else {
            unreachable!();
        }
    }

    pub fn set_padding_ex(&mut self, left: f32, top: f32, right: f32, bottom: f32) {
        if let Some(group_id) = self.group_id {
            let group = self.get_group_mut(group_id);

            group.paddingLower = Vec2::new(left, top);
            group.paddingUpper = Vec2::new(right, bottom);
        } else {
            unreachable!();
        }
    }

    pub fn set_padding_left(&mut self, padding: f32) {
        if let Some(group_id) = self.group_id {
            let group = self.get_group_mut(group_id);

            group.paddingLower.x = padding;
        } else {
            unreachable!();
        }
    }

    pub fn set_padding_top(&mut self, padding: f32) {
        if let Some(group_id) = self.group_id {
            let group = self.get_group_mut(group_id);

            group.paddingLower.y = padding;
        } else {
            unreachable!();
        }
    }

    pub fn set_padding_right(&mut self, padding: f32) {
        if let Some(group_id) = self.group_id {
            let group = self.get_group_mut(group_id);

            group.paddingUpper.x = padding;
        } else {
            unreachable!();
        }
    }

    pub fn set_padding_bottom(&mut self, padding: f32) {
        if let Some(group_id) = self.group_id {
            let group = self.get_group_mut(group_id);

            group.paddingUpper.y = padding;
        } else {
            unreachable!();
        }
    }

    pub fn set_spacing(&mut self, spacing: f32) {
        if let Some(group_id) = self.group_id {
            let group = self.get_group_mut(group_id);

            group.spacing = spacing;
        } else {
            unreachable!();
        }
    }

    pub fn set_stretch(&mut self, x: f32, y: f32) {
        if let Some(id) = self.last_id {
            let widget = self.get_widget_mut(id);

            widget.stretch = Vec2::new(x, y);
        } else {
            unreachable!();
        }
    }

    pub fn group_has_focus(&mut self, ty: FocusType) -> bool {
        if let Some(group_id) = self.group_id {
            let group = self.get_group_mut(group_id);
            let widget = self.get_widget_mut(group.widget_id);

            group.focusable[ty as usize] = true;
            self.focus[ty as usize] == widget.hash
        } else {
            unreachable!();
        }
    }

    pub fn push_style(&mut self) {
        self.styles.push(Default::default());
    }

    pub fn push_font(&mut self, font: &mut Font) {
        self.push_style();

        let style = self.styles.last_mut().expect("Style was not set");

        style.font = font;
    }

    pub fn push_text_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.push_style();

        let style = self.styles.last_mut().expect("Style was not set");

        style.colorText = Vec4::new(r, g, b, a);
    }

    pub fn pop_style(&mut self, depth: i32) {
        assert!(self.styles.len() >= depth as usize);

        self.styles.truncate(self.styles.len() - depth as usize);
    }
}
