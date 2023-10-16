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

#[derive(Clone)]
pub struct HmGui {
    pub group: *mut HmGuiGroup,
    pub root: *mut HmGuiGroup,
    pub last: *mut HmGuiWidget,
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
            group: std::ptr::null_mut(),
            root: std::ptr::null_mut(),
            last: std::ptr::null_mut(),
            styles: vec![style],
            data: HashMap::with_capacity(128),
            focus: [0; 2],
            focusPos: Vec2::ZERO,
            activate: false,
        }
    }

    fn init_widget(&mut self, e: *mut HmGuiWidget, ty: WidgetType) {
        unsafe {
            (*e).parent = self.group;
            (*e).next = std::ptr::null_mut();
            (*e).prev = if !(self.group).is_null() {
                (*self.group).tail
            } else {
                std::ptr::null_mut()
            };

            if !((*e).parent).is_null() {
                (*(*e).parent).children = ((*(*e).parent).children).wrapping_add(1);
                (*e).hash = Hash_FNV64_Incremental(
                    (*(*e).parent).widget.hash,
                    &mut (*(*e).parent).children as *mut u32 as *const _,
                    std::mem::size_of::<u32>() as i32,
                );
                if !((*e).next).is_null() {
                    (*(*e).next).prev = e;
                } else {
                    (*(*e).parent).tail = e;
                }
                if !((*e).prev).is_null() {
                    (*(*e).prev).next = e;
                } else {
                    (*(*e).parent).head = e;
                }
            } else {
                (*e).hash = Hash_FNV64_Init();
            }

            (*e).ty = ty;
            (*e).pos = Vec2::ZERO;
            (*e).size = Vec2::ZERO;
            (*e).minSize = Vec2::ZERO;
            (*e).align = Vec2::ZERO;
            (*e).stretch = Vec2::ZERO;
        }

        self.last = e;
    }

    fn begin_group(&mut self, layout: LayoutType) {
        let spacing = self.styles.last().expect("Style was not set").spacing;

        unsafe {
            let e = MemNew!(HmGuiGroup);

            self.init_widget(&mut (*e).widget, WidgetType::Group);

            (*e).head = std::ptr::null_mut();
            (*e).tail = std::ptr::null_mut();
            (*e).layout = layout;
            (*e).children = 0;
            (*e).focusStyle = FocusStyle::None;
            (*e).paddingLower = Vec2::ZERO;
            (*e).paddingUpper = Vec2::ZERO;
            (*e).offset = Vec2::ZERO;
            (*e).maxSize = Vec2::new(1e30f32, 1e30f32);
            (*e).spacing = spacing;
            (*e).frameOpacity = 0.0f32;
            (*e).clip = false;
            (*e).expand = true;
            (*e).focusable.fill(false);
            (*e).storeSize = false;
            self.group = e;

            match layout {
                LayoutType::Stack => {
                    (*e).widget.stretch = Vec2::ONE;
                }
                LayoutType::Vertical => {
                    (*e).widget.stretch = Vec2::X;
                }
                LayoutType::Horizontal => {
                    (*e).widget.stretch = Vec2::Y;
                }
                _ => {}
            };
        }
    }

    pub fn get_data(&mut self, g: *const HmGuiGroup) -> *mut HmGuiData {
        unsafe {
            self.data.entry((*g).widget.hash).or_insert(HmGuiData {
                offset: Vec2::ZERO,
                minSize: Vec2::ZERO,
                size: Vec2::ZERO,
            })
        }
    }

    fn check_focus(&mut self, g: *mut HmGuiGroup) {
        unsafe {
            if (*g).clip as i32 != 0 && (*g).is_clipped(self.focusPos) as i32 != 0 {
                return;
            }

            let mut e: *mut HmGuiWidget = (*g).tail;
            while !e.is_null() {
                if (*e).ty == WidgetType::Group {
                    self.check_focus(e as *mut HmGuiGroup);
                }
                e = (*e).prev;
            }

            for i in 0..self.focus.len() {
                if self.focus[i as usize] == 0
                    && (*g).focusable[i as usize] as i32 != 0
                    && (*g).widget.pos.x <= self.focusPos.x
                    && (*g).widget.pos.y <= self.focusPos.y
                    && self.focusPos.x <= (*g).widget.pos.x + (*g).widget.size.x
                    && self.focusPos.y <= (*g).widget.pos.y + (*g).widget.size.y
                {
                    self.focus[i as usize] = (*g).widget.hash;
                }
            }
        }
    }
}

#[luajit_ffi_gen::luajit_ffi(name = "HmGuiFfi")]
impl HmGui {
    pub fn begin(&mut self, sx: f32, sy: f32, input: &Input) {
        unsafe {
            if !(self.root).is_null() {
                HmGui_FreeGroup(self.root);
                self.root = std::ptr::null_mut();
            }
            self.last = std::ptr::null_mut();
            self.activate = input.mouse().is_pressed(MouseControl::Left);

            self.begin_group(LayoutType::None);

            (*self.group).clip = true;
            (*self.group).widget.pos = Vec2::ZERO;
            (*self.group).widget.size = Vec2::new(sx, sy);

            self.root = self.group;
        }
    }

    pub fn end(&mut self, input: &Input) {
        unsafe {
            Profiler_Begin(c_str!("HmGui_End"));

            self.end_group();

            (*self.root).compute_size(self);
            (*self.root).layout(self);

            self.focus.fill(0);

            let mouse = input.mouse();

            self.focusPos = mouse.position();

            self.check_focus(self.root);

            Profiler_End();
        }
    }

    pub fn draw(&mut self) {
        unsafe {
            Profiler_Begin(c_str!("HmGui_Draw"));

            RenderState_PushBlendMode(1);
            UIRenderer_Begin();

            (*self.root).draw(self.focus[FocusType::Mouse as usize]);

            UIRenderer_End();
            RenderState_PopBlendMode();

            UIRenderer_Draw();

            Profiler_End();
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
        unsafe {
            self.last = &mut (*self.group).widget;
            self.group = (*self.group).widget.parent;
        }
    }

    pub fn begin_scroll(&mut self, maxSize: f32) {
        unsafe {
            self.begin_group_x();
            self.set_stretch(1.0f32, 1.0f32);
            (*self.group).clip = true;
            self.set_spacing(2.0f32);

            self.begin_group_y();
            self.set_padding(6.0f32, 6.0f32);
            self.set_stretch(1.0f32, 1.0f32);

            (*self.group).expand = false;
            (*self.group).storeSize = true;
            (*self.group).maxSize.y = maxSize;

            let data = self.get_data(self.group);
            (*self.group).offset.y = -(*data).offset.y;
        }
    }

    pub fn end_scroll(&mut self, input: &Input) {
        let data = self.get_data(self.group);

        unsafe {
            if self.group_has_focus(FocusType::Scroll) {
                let scroll_y = input.mouse().value(MouseControl::ScrollY);

                (*data).offset.y -= 10.0f32 * scroll_y as f32;
            }

            let maxScroll: f32 = f32::max(0.0f32, (*data).minSize.y - (*data).size.y);
            (*data).offset.y = f32::clamp((*data).offset.y, 0.0f32, maxScroll);

            self.end_group();

            self.begin_group_y();
            self.set_stretch(0.0f32, 1.0f32);
            self.set_spacing(0.0f32);

            if maxScroll > 0.0f32 {
                let handleSize: f32 = (*data).size.y * ((*data).size.y / (*data).minSize.y);
                let handlePos: f32 = Lerp(
                    0.0f64,
                    ((*data).size.y - handleSize) as f64,
                    ((*data).offset.y / maxScroll) as f64,
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
        }
    }

    pub fn begin_window(&mut self, _title: &str, input: &Input) {
        unsafe {
            self.begin_group_stack();
            self.set_stretch(0.0f32, 0.0f32);

            (*self.group).focusStyle = FocusStyle::None;
            (*self.group).frameOpacity = 0.95f32;

            let data = self.get_data(self.group);
            let mouse = input.mouse();

            if self.group_has_focus(FocusType::Mouse) && mouse.is_down(MouseControl::Left) {
                (*data).offset.x += mouse.value(MouseControl::DeltaX);
                (*data).offset.y += mouse.value(MouseControl::DeltaY);
            }

            (*self.group).widget.pos.x += (*data).offset.x;
            (*self.group).widget.pos.y += (*data).offset.y;

            self.begin_group_y();
            (*self.group).clip = true;
            self.set_padding(8.0f32, 8.0f32);
            self.set_stretch(1.0f32, 1.0f32);
            // HmGui_TextColored(title, 1.0f, 1.0f, 1.0f, 0.3f);
            // self.set_align(0.5f, 0.0f);
        }
    }

    pub fn end_window(&mut self) {
        self.end_group();
        self.end_group();
    }

    pub fn button(&mut self, label: &str) -> bool {
        self.begin_group_stack();

        unsafe {
            (*self.group).focusStyle = FocusStyle::Fill;
            (*self.group).frameOpacity = 0.5f32;
        }

        let focus: bool = self.group_has_focus(FocusType::Mouse);

        self.set_padding(8.0f32, 8.0f32);
        self.text(label);
        self.set_align(0.5f32, 0.5f32);

        self.end_group();

        focus as i32 != 0 && self.activate as i32 != 0
    }

    pub fn checkbox(&mut self, label: &str, mut value: bool) -> bool {
        self.begin_group_x();

        unsafe {
            (*self.group).focusStyle = FocusStyle::Underline;
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
        unsafe {
            let e = MemNew!(HmGuiImage);

            self.init_widget(&mut (*e).widget, WidgetType::Image);

            (*e).image = image;
            (*e).widget.stretch = Vec2::ONE;
        }
    }

    pub fn rect(&mut self, sx: f32, sy: f32, r: f32, g: f32, b: f32, a: f32) {
        unsafe {
            let e = MemNew!(HmGuiRect);

            self.init_widget(&mut (*e).widget, WidgetType::Rect);

            (*e).color = Vec4::new(r, g, b, a);
            (*e).widget.minSize = Vec2::new(sx, sy);
        }
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
        unsafe {
            let e = MemNew!(HmGuiText);

            self.init_widget(&mut (*e).widget, WidgetType::Text);

            (*e).font = font;
            (*e).text = text.into();
            (*e).color = Vec4::new(r, g, b, a);

            let mut size = IVec2::ZERO;

            let ctext = CString::new(text).expect("Cannot convert text");
            Font_GetSize2(&mut *(*e).font, &mut size, ctext.as_ptr());

            (*e).widget.minSize = Vec2::new(size.x as f32, size.y as f32);

            self.set_align(0.0f32, 1.0f32);
        }
    }

    pub fn set_align(&mut self, ax: f32, ay: f32) {
        unsafe {
            (*self.last).align = Vec2::new(ax, ay);
        }
    }

    pub fn set_padding(&mut self, px: f32, py: f32) {
        unsafe {
            (*self.group).paddingLower = Vec2::new(px, py);
            (*self.group).paddingUpper = Vec2::new(px, py);
        }
    }

    pub fn set_padding_ex(&mut self, left: f32, top: f32, right: f32, bottom: f32) {
        unsafe {
            (*self.group).paddingLower = Vec2::new(left, top);
            (*self.group).paddingUpper = Vec2::new(right, bottom);
        }
    }

    pub fn set_padding_left(&mut self, padding: f32) {
        unsafe {
            (*self.group).paddingLower.x = padding;
        }
    }

    pub fn set_padding_top(&mut self, padding: f32) {
        unsafe {
            (*self.group).paddingLower.y = padding;
        }
    }

    pub fn set_padding_right(&mut self, padding: f32) {
        unsafe {
            (*self.group).paddingUpper.x = padding;
        }
    }

    pub fn set_padding_bottom(&mut self, padding: f32) {
        unsafe {
            (*self.group).paddingUpper.y = padding;
        }
    }

    pub fn set_spacing(&mut self, spacing: f32) {
        unsafe {
            (*self.group).spacing = spacing;
        }
    }

    pub fn set_stretch(&mut self, x: f32, y: f32) {
        unsafe {
            (*self.last).stretch = Vec2::new(x, y);
        }
    }

    pub fn group_has_focus(&mut self, ty: FocusType) -> bool {
        unsafe {
            (*self.group).focusable[ty as usize] = true;
            self.focus[ty as usize] == (*self.group).widget.hash
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
