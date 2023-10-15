use glam::Vec2;
use internal::*;

use crate::render::UIRenderer_BeginLayer;
use crate::render::UIRenderer_EndLayer;
use crate::render::UIRenderer_Panel;
use crate::render::UIRenderer_Rect;

use super::data::*;
use super::focus::*;
use super::image::*;
use super::rect::*;
use super::text::*;
use super::widget::*;
use super::HmGui_GetData;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGuiGroup {
    pub widget: HmGuiWidget,
    pub head: *mut HmGuiWidget,
    pub tail: *mut HmGuiWidget,
    pub layout: LayoutType,
    pub children: u32,
    pub focusStyle: FocusStyle,
    pub paddingLower: Vec2,
    pub paddingUpper: Vec2,
    pub offset: Vec2,
    pub maxSize: Vec2,
    pub totalStretch: Vec2,
    pub spacing: f32,
    pub frameOpacity: f32,
    pub clip: bool,
    pub expand: bool,
    pub focusable: [bool; 2],
    pub storeSize: bool,
}

pub unsafe extern "C" fn HmGui_FreeGroup(g: *mut HmGuiGroup) {
    let mut e: *mut HmGuiWidget = (*g).head;
    while !e.is_null() {
        let next: *mut HmGuiWidget = (*e).next;
        match (*e).ty {
            WidgetType::Group => {
                HmGui_FreeGroup(e as *mut HmGuiGroup);
            }
            WidgetType::Text => {
                HmGui_FreeText(e as *mut HmGuiText);
            }
            _ => {
                MemFree(e as *const _);
            }
        }
        e = next;
    }
    MemFree(g as *const _);
}

impl HmGuiGroup {
    #[inline]
    pub fn is_clipped(&self, p: Vec2) -> bool {
        p.x < self.widget.pos.x
            || p.y < self.widget.pos.y
            || self.widget.pos.x + self.widget.size.x < p.x
            || self.widget.pos.y + self.widget.size.y < p.y
    }

    pub fn compute_size(&mut self) {
        unsafe {
            let mut e = self.head;
            while !e.is_null() {
                if (*e).ty == WidgetType::Group {
                    (*(e as *mut HmGuiGroup)).compute_size();
                }
                e = (*e).next;
            }

            self.widget.minSize = Vec2::ZERO;

            let mut e = self.head;
            while !e.is_null() {
                match self.layout {
                    LayoutType::Stack => {
                        self.widget.minSize.x = f32::max(self.widget.minSize.x, (*e).minSize.x);
                        self.widget.minSize.y = f32::max(self.widget.minSize.y, (*e).minSize.y);
                    }
                    LayoutType::Vertical => {
                        self.widget.minSize.x = f32::max(self.widget.minSize.x, (*e).minSize.x);
                        self.widget.minSize.y += (*e).minSize.y;
                        if e != self.head {
                            self.widget.minSize.y += self.spacing;
                        }
                    }
                    LayoutType::Horizontal => {
                        self.widget.minSize.x += (*e).minSize.x;
                        self.widget.minSize.y = f32::max(self.widget.minSize.y, (*e).minSize.y);
                        if e != self.head {
                            self.widget.minSize.x += self.spacing;
                        }
                    }
                    _ => {}
                }
                e = (*e).next;
            }

            self.widget.minSize.x += self.paddingLower.x + self.paddingUpper.x;
            self.widget.minSize.y += self.paddingLower.y + self.paddingUpper.y;

            if self.storeSize {
                let data: *mut HmGuiData = HmGui_GetData(self as *const _);
                (*data).minSize = self.widget.minSize;
            }
        }

        self.widget.minSize.x = f32::min(self.widget.minSize.x, self.maxSize.x);
        self.widget.minSize.y = f32::min(self.widget.minSize.y, self.maxSize.y);
    }

    pub fn layout(&self) {
        let mut pos = self.widget.pos;
        let mut size = self.widget.size;
        let mut extra: f32 = 0.0f32;
        let mut totalStretch: f32 = 0.0f32;

        pos.x += self.paddingLower.x + self.offset.x;
        pos.y += self.paddingLower.y + self.offset.y;
        size.x -= self.paddingLower.x + self.paddingUpper.x;
        size.y -= self.paddingLower.y + self.paddingUpper.y;

        unsafe {
            if self.expand {
                if self.layout == LayoutType::Vertical {
                    extra = self.widget.size.y - self.widget.minSize.y;
                    let mut e = self.head;
                    while !e.is_null() {
                        totalStretch += (*e).stretch.y;
                        e = (*e).next;
                    }
                } else if self.layout == LayoutType::Horizontal {
                    extra = self.widget.size.x - self.widget.minSize.x;
                    let mut e = self.head;
                    while !e.is_null() {
                        totalStretch += (*e).stretch.x;
                        e = (*e).next;
                    }
                }

                if totalStretch > 0.0f32 {
                    extra /= totalStretch;
                }
            }

            let mut e = self.head;
            while !e.is_null() {
                match self.layout {
                    LayoutType::None => {
                        (*e).layout((*e).pos, size.x, size.y);
                    }
                    LayoutType::Stack => {
                        (*e).layout(pos, size.x, size.y);
                    }
                    LayoutType::Vertical => {
                        let mut s = (*e).minSize.y;
                        if extra > 0.0f32 {
                            s += (*e).stretch.y * extra;
                        }
                        (*e).layout(pos, size.x, s);
                        pos.y += (*e).size.y + self.spacing;
                    }
                    LayoutType::Horizontal => {
                        let mut s = (*e).minSize.x;
                        if extra > 0.0f32 {
                            s += (*e).stretch.x * extra;
                        }
                        (*e).layout(pos, s, size.y);
                        pos.x += (*e).size.x + self.spacing;
                    }
                }

                if (*e).ty == WidgetType::Group {
                    (*(e as *mut HmGuiGroup)).layout();
                }

                e = (*e).next;
            }

            if self.storeSize {
                let data = HmGui_GetData(self as *const _);
                (*data).size = self.widget.size;
            }
        }
    }

    pub fn draw(&self, hmgui_focus: u64) {
        // #if HMGUI_DRAW_GROUP_FRAMES
        //   Draw_Color(0.2f, 0.2f, 0.2f, 0.5f);
        //   Draw_Border(2.0f, g->pos.x, g->pos.y, g->size.x, g->size.y);
        // #endif

        unsafe {
            UIRenderer_BeginLayer(
                self.widget.pos.x,
                self.widget.pos.y,
                self.widget.size.x,
                self.widget.size.y,
                self.clip,
            );

            let mut e = self.tail;
            while !e.is_null() {
                match (*e).ty {
                    WidgetType::Group => {
                        (*(e as *mut HmGuiGroup)).draw(hmgui_focus);
                    }
                    WidgetType::Text => {
                        (*(e as *mut HmGuiText)).draw();
                    }
                    WidgetType::Rect => {
                        (*(e as *mut HmGuiRect)).draw();
                    }
                    WidgetType::Image => {
                        (*(e as *mut HmGuiImage)).draw();
                    }
                }
                e = (*e).prev;
            }

            if self.focusable[FocusType::Mouse as usize] {
                let focus: bool = hmgui_focus == self.widget.hash;

                match self.focusStyle {
                    FocusStyle::None => {
                        UIRenderer_Panel(
                            self.widget.pos.x,
                            self.widget.pos.y,
                            self.widget.size.x,
                            self.widget.size.y,
                            0.1f32,
                            0.12f32,
                            0.13f32,
                            1.0f32,
                            8.0f32,
                            self.frameOpacity,
                        );
                    }
                    FocusStyle::Fill => {
                        if focus {
                            UIRenderer_Panel(
                                self.widget.pos.x,
                                self.widget.pos.y,
                                self.widget.size.x,
                                self.widget.size.y,
                                0.1f32,
                                0.5f32,
                                1.0f32,
                                1.0f32,
                                0.0f32,
                                1.0f32,
                            );
                        } else {
                            UIRenderer_Panel(
                                self.widget.pos.x,
                                self.widget.pos.y,
                                self.widget.size.x,
                                self.widget.size.y,
                                0.15f32,
                                0.15f32,
                                0.15f32,
                                0.8f32,
                                0.0f32,
                                self.frameOpacity,
                            );
                        }
                    }
                    FocusStyle::Outline => {
                        if focus {
                            UIRenderer_Rect(
                                self.widget.pos.x,
                                self.widget.pos.y,
                                self.widget.size.x,
                                self.widget.size.y,
                                0.1f32,
                                0.5f32,
                                1.0f32,
                                1.0f32,
                                true,
                            );
                        }
                    }
                    FocusStyle::Underline => {
                        UIRenderer_Rect(
                            self.widget.pos.x,
                            self.widget.pos.y,
                            self.widget.size.x,
                            self.widget.size.y,
                            0.3f32,
                            0.3f32,
                            0.3f32,
                            if focus as i32 != 0 {
                                0.5f32
                            } else {
                                self.frameOpacity
                            },
                            false,
                        );
                    }
                }
            }
            UIRenderer_EndLayer();
        }
    }
}
