mod data;
mod focus;
mod group;
mod image;
mod rect;
mod rf;
mod style;
mod text;
mod widget;

use std::borrow::{Borrow, BorrowMut};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::ffi::CString;
use std::fs::File;
use std::path::PathBuf;

use internal::*;

pub(crate) use self::data::*;
pub(crate) use self::focus::*;
pub(crate) use self::group::*;
pub(crate) use self::image::*;
pub(crate) use self::rect::*;
pub(crate) use self::rf::*;
pub(crate) use self::style::*;
pub(crate) use self::text::*;
pub(crate) use self::widget::*;

use super::*;
use crate::common::*;
use crate::input::*;
use crate::math::*;
use crate::render::*;
use crate::system::{Hash_FNV64_Incremental, Hash_FNV64_Init, Profiler_Begin, Profiler_End};
use crate::*;

pub(crate) const IDENT: &str = "  ";

pub struct HmGui {
    /// Current active group
    pub group: Option<Rf<HmGuiWidget>>,
    /// Top level group object. Used for recalculating sizes, layouts and drawing of the whole gui
    pub root: Option<Rf<HmGuiWidget>>,
    /// Either last created/initialized widget (group, image, text, rect) or the last widget of the ended group
    pub last: Option<Rf<HmGuiWidget>>,

    pub styles: Vec<HmGuiStyle>,
    pub data: HashMap<u64, HmGuiData>,
    pub focus: [u64; 2],
    pub focusPos: Vec2,
    pub activate: bool,
}

impl HmGui {
    pub fn new(default_font: *mut Font) -> Self {
        let style = HmGuiStyle {
            font: default_font,
            spacing: 6.0f32,
            colorPrimary: Vec4::new(0.1f32, 0.5f32, 1.0f32, 1.0f32),
            colorFrame: Vec4::new(0.1f32, 0.1f32, 0.1f32, 0.5f32),
            colorText: Vec4::ONE,
        };

        Self {
            group: None,
            root: None,
            last: None,
            styles: vec![style],
            data: HashMap::with_capacity(128),
            focus: [0; 2],
            focusPos: Vec2::ZERO,
            activate: false,
        }
    }

    pub fn mouse_focus_hash(&self) -> u64 {
        self.focus[FocusType::Mouse as usize]
    }

    fn init_widget(&mut self, item: WidgetItem) -> Rf<HmGuiWidget> {
        let prev = self
            .group
            .as_ref()
            .map(|group_rf| {
                if let WidgetItem::Group(group) = &group_rf.as_ref().item {
                    group.tail.clone()
                } else {
                    unreachable!();
                }
            })
            .flatten();
        let widget = HmGuiWidget {
            parent: self.group.clone(),
            next: None,
            prev,
            hash: 0,
            item,
            pos: Default::default(),
            size: Default::default(),
            minSize: Default::default(),
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

            parent_group.children = (parent_group.children).wrapping_add(1);

            widget.hash = unsafe {
                Hash_FNV64_Incremental(
                    parent_hash,
                    &mut parent_group.children as *mut u32 as *const _,
                    std::mem::size_of::<u32>() as i32,
                )
            };

            if let Some(next_rf) = &widget.next {
                let mut next = next_rf.as_mut();
                next.prev = Some(widget_rf.clone());
            } else {
                parent_group.tail = Some(widget_rf.clone());
            }

            if let Some(prev_rf) = &widget.prev {
                let mut prev = prev_rf.as_mut();
                prev.next = Some(widget_rf.clone());
            } else {
                parent_group.head = Some(widget_rf.clone());
            }
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
            maxSize: Vec2::new(1e30f32, 1e30f32),
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
            minSize: Vec2::ZERO,
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

        if group.clip && self.is_clipped(widget.pos, widget.size, self.focusPos) {
            return;
        }

        let mut widget_opt = group.tail.clone();
        while let Some(widget_rf) = widget_opt {
            self.check_focus(widget_rf.clone());

            let widget = widget_rf.as_ref();
            widget_opt = widget.prev.clone();
        }

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

    // For testing.
    #[allow(dead_code)]
    pub(crate) fn dump_widgets(&self) {
        let file_path = PathBuf::new().join("widgets.txt");

        if file_path.exists() {
            return;
        }

        let mut file = File::create(file_path).expect("Cannot create widgets.txt");

        println!("Widgets:");

        if let Some(group_opt) = &self.root {
            let group = group_opt.as_ref();

            group.dump(1, &mut file);
        } else {
            println!("{IDENT}No widgets");
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

                self.focusPos = mouse.position();

                root_rf.clone()
            };

            self.check_focus(group_rf);
        } else {
            unreachable!();
        }

        unsafe { Profiler_End() };
    }

    pub fn draw(&mut self) {
        if let Some(root_rf) = self.root.clone() {
            unsafe {
                Profiler_Begin(c_str!("HmGui_Draw"));

                RenderState_PushBlendMode(1);
                UIRenderer_Begin();
            }

            root_rf.as_ref().draw(self);

            unsafe {
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
        if let Some(widget_rf) = self.group.clone() {
            let widget = widget_rf.as_ref();

            self.last = Some(widget_rf.clone());
            self.group = widget.parent.clone();
        } else {
            unreachable!();
        }
    }

    pub fn begin_scroll(&mut self, maxSize: f32) {
        if let Some(widget_rf) = self.group.clone() {
            let mut widget = widget_rf.as_mut();
            let widget_hash = widget.hash;
            let WidgetItem::Group(group) = &mut widget.item else {
                unreachable!()
            };

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

                data.offset.y -= 10.0f32 * scroll_y as f32;
            }

            let maxScroll = f32::max(0.0f32, data.minSize.y - data.size.y);
            data.offset.y = f32::clamp(data.offset.y, 0.0f32, maxScroll);

            self.end_group();

            self.begin_group_y();
            self.set_stretch(0.0f32, 1.0f32);
            self.set_spacing(0.0f32);

            if maxScroll > 0.0f32 {
                let data = self.get_data(widget.hash);
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
        if let Some(widget_rf) = self.group.clone() {
            self.begin_group_stack();
            self.set_stretch(0.0f32, 0.0f32);

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
                group.focusStyle = FocusStyle::None;
                group.frameOpacity = 0.95f32;
                group.clip = true;
            }

            self.begin_group_y();
            self.set_padding(8.0f32, 8.0f32);
            self.set_stretch(1.0f32, 1.0f32);
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

                group.focusStyle = FocusStyle::Fill;
                group.frameOpacity = 0.5f32;
            }

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
        if let Some(widget_rf) = self.group.clone() {
            self.begin_group_x();

            {
                let mut widget = widget_rf.as_mut();
                let WidgetItem::Group(group) = &mut widget.item else {
                    unreachable!()
                };

                group.focusStyle = FocusStyle::Underline;
            }

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
            font,
            text: text.into(),
            color: Vec4::new(r, g, b, a),
        };

        let widget_rf = self.init_widget(WidgetItem::Text(item));

        let mut size = IVec2::ZERO;

        let ctext = CString::new(text).expect("Cannot convert text");

        unsafe { Font_GetSize2(font, &mut size, ctext.as_ptr()) };

        // NOTE: This scope is needed to prevent widget be mut borrowed twice here and in set_align below
        {
            let mut widget = widget_rf.as_mut();
            widget.minSize = Vec2::new(size.x as f32, size.y as f32);
        }

        self.set_align(0.0f32, 1.0f32);
    }

    pub fn set_align(&mut self, ax: f32, ay: f32) {
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

            group.paddingLower = Vec2::new(px, py);
            group.paddingUpper = Vec2::new(px, py);
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

            group.paddingLower = Vec2::new(left, top);
            group.paddingUpper = Vec2::new(right, bottom);
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

            group.paddingLower.x = padding;
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

            group.paddingLower.y = padding;
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

            group.paddingUpper.x = padding;
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

            group.paddingUpper.y = padding;
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

#[cfg(test)]
mod tests {
    use crate::{input::Input, render::Font};

    use super::HmGui;

    #[test]
    fn test_hmgui_rect() {
        let input = Input::default();
        let mut gui = HmGui::new(std::ptr::null_mut());

        gui.begin_gui(640.0, 480.0, &input);
        gui.rect(100.0, 50.0, 1.0, 0.0, 0.0, 0.0);
        gui.end_gui(&input);

        // gui.dump_widgets();
    }

    #[test]
    #[ignore]
    fn test_hmgui_main_menu() {
        let input = Input::default();
        let mut gui = HmGui::new(std::ptr::null_mut());

        let res_x = 640.0;
        let res_y = 480.0;

        gui.begin_gui(res_x, res_y, &input);

        // let scalefactor = (res_x / 22.0) / 72.0;
        // let scalefactorMenuX = 352.8 / res_x;
        // let scalefactorMenuY = 549.0 / res_y;

        // let mut glyphsAscii = [std::ptr::null_mut(); 256];
        // let mut font = Font {
        //     _refCount: 0,
        //     handle: std::ptr::null_mut(),
        //     glyphs: std::ptr::null_mut(),
        //     glyphsAscii,
        // };

        gui.begin_group_stack();
        // gui.text_ex(&mut font, "LIMIT THEORY", 0.2, 0.2, 0.2, 1.0);
        // gui.set_align(0.031, 0.042);
        // gui.text_ex(&mut font, "LIMIT THEORY", 0.9, 0.9, 0.9, 1.0);
        // gui.set_align(0.03, 0.04);
        // gui.text_ex(&mut font, "REDUX", 0.2, 0.2, 0.2, 1.0);
        // gui.set_align(0.181, 0.132);
        // gui.text_ex(&mut font, "REDUX", 0.9, 0.9, 0.9, 1.0);
        // gui.set_align(0.18, 0.13);

        // gui.text_ex(&mut font, Config.gameVersion, 0.2, 0.2, 0.2, 1.0);
        // gui.set_align(0.012, 0.971);
        // gui.text_ex(&mut font, Config.gameVersion, 0.9, 0.9, 0.9, 1.0);
        // gui.set_align(0.011, 0.970);

        // gui.text_ex(&mut font,
        //     "Resolution = " .. res_X .. " x " .. res_Y, 0.2, 0.2, 0.2, 1.0);
        // gui.set_align(0.221, 0.971);
        // gui.text_ex(&mut font,
        //     "Resolution = " .. res_X .. " x " .. res_Y, 0.9, 0.9, 0.9, 1.0);
        // gui.set_align(0.220, 0.970);

        // let scalefactor = (res_x / 24.0) / 72.0;

        gui.begin_group_y();
        gui.push_text_color(0.9, 0.9, 0.9, 1.0);
        // gui.push_font(&mut font, 36 * scalefactor));

        if gui.button("NEW GAME") {
            println!("NEW GAME");
        }

        if gui.button("LOAD GAME") {
            println!("LOAD GAME");
        }

        if gui.button("SETTINGS") {
            println!("SETTINGS");
        }

        if gui.button("CREDITS") {
            println!("CREDITS");
        }

        if gui.button("BACKGROUND") {
            println!("BACKGROUND");
        }

        if gui.button("EXIT GAME") {
            println!("EXIT GAME");
        }

        // gui.pop_style(2);
        gui.end_group();

        gui.set_stretch(0.18, 0.5);
        gui.set_align(0.0065, 0.8);
        gui.end_group();

        gui.end_gui(&input);

        gui.dump_widgets();
    }
}
