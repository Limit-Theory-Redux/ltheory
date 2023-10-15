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

#[inline]
pub unsafe extern "C" fn IsClipped(g: *mut HmGuiGroup, p: Vec2) -> bool {
    p.x < (*g).widget.pos.x
        || p.y < (*g).widget.pos.y
        || (*g).widget.pos.x + (*g).widget.size.x < p.x
        || (*g).widget.pos.y + (*g).widget.size.y < p.y
}

pub unsafe extern "C" fn HmGui_ComputeSize(g: *mut HmGuiGroup) {
    let mut e: *mut HmGuiWidget = (*g).head;
    while !e.is_null() {
        if (*e).ty == WidgetType::Group {
            HmGui_ComputeSize(e as *mut HmGuiGroup);
        }
        e = (*e).next;
    }

    (*g).widget.minSize = Vec2::ZERO;

    let mut e: *mut HmGuiWidget = (*g).head;
    while !e.is_null() {
        match (*g).layout {
            LayoutType::Stack => {
                (*g).widget.minSize.x = f32::max((*g).widget.minSize.x, (*e).minSize.x);
                (*g).widget.minSize.y = f32::max((*g).widget.minSize.y, (*e).minSize.y);
            }
            LayoutType::Vertical => {
                (*g).widget.minSize.x = f32::max((*g).widget.minSize.x, (*e).minSize.x);
                (*g).widget.minSize.y += (*e).minSize.y;
                if e != (*g).head {
                    (*g).widget.minSize.y += (*g).spacing;
                }
            }
            LayoutType::Horizontal => {
                (*g).widget.minSize.x += (*e).minSize.x;
                (*g).widget.minSize.y = f32::max((*g).widget.minSize.y, (*e).minSize.y);
                if e != (*g).head {
                    (*g).widget.minSize.x += (*g).spacing;
                }
            }
            _ => {}
        }
        e = (*e).next;
    }

    (*g).widget.minSize.x += (*g).paddingLower.x + (*g).paddingUpper.x;
    (*g).widget.minSize.y += (*g).paddingLower.y + (*g).paddingUpper.y;

    if (*g).storeSize {
        let data: *mut HmGuiData = HmGui_GetData(g);
        (*data).minSize = (*g).widget.minSize;
    }

    (*g).widget.minSize.x = f32::min((*g).widget.minSize.x, (*g).maxSize.x);
    (*g).widget.minSize.y = f32::min((*g).widget.minSize.y, (*g).maxSize.y);
}

pub unsafe extern "C" fn HmGui_LayoutGroup(g: *mut HmGuiGroup) {
    let mut pos = (*g).widget.pos;
    let mut size = (*g).widget.size;
    let mut extra: f32 = 0.0f32;
    let mut totalStretch: f32 = 0.0f32;

    pos.x += (*g).paddingLower.x + (*g).offset.x;
    pos.y += (*g).paddingLower.y + (*g).offset.y;
    size.x -= (*g).paddingLower.x + (*g).paddingUpper.x;
    size.y -= (*g).paddingLower.y + (*g).paddingUpper.y;

    if (*g).expand {
        if (*g).layout == LayoutType::Vertical {
            extra = (*g).widget.size.y - (*g).widget.minSize.y;
            let mut e: *mut HmGuiWidget = (*g).head;
            while !e.is_null() {
                totalStretch += (*e).stretch.y;
                e = (*e).next;
            }
        } else if (*g).layout == LayoutType::Horizontal {
            extra = (*g).widget.size.x - (*g).widget.minSize.x;
            let mut e: *mut HmGuiWidget = (*g).head;
            while !e.is_null() {
                totalStretch += (*e).stretch.x;
                e = (*e).next;
            }
        }

        if totalStretch > 0.0f32 {
            extra /= totalStretch;
        }
    }

    let mut e: *mut HmGuiWidget = (*g).head;
    while !e.is_null() {
        match (*g).layout {
            LayoutType::None => {
                HmGui_LayoutWidget(e, (*e).pos, size.x, size.y);
            }
            LayoutType::Stack => {
                HmGui_LayoutWidget(e, pos, size.x, size.y);
            }
            LayoutType::Vertical => {
                let mut s = (*e).minSize.y;
                if extra > 0.0f32 {
                    s += (*e).stretch.y * extra;
                }
                HmGui_LayoutWidget(e, pos, size.x, s);
                pos.y += (*e).size.y + (*g).spacing;
            }
            LayoutType::Horizontal => {
                let mut s = (*e).minSize.x;
                if extra > 0.0f32 {
                    s += (*e).stretch.x * extra;
                }
                HmGui_LayoutWidget(e, pos, s, size.y);
                pos.x += (*e).size.x + (*g).spacing;
            }
        }

        if (*e).ty == WidgetType::Group {
            HmGui_LayoutGroup(e as *mut HmGuiGroup);
        }

        e = (*e).next;
    }

    if (*g).storeSize {
        let data: *mut HmGuiData = HmGui_GetData(g);
        (*data).size = (*g).widget.size;
    }
}

pub unsafe extern "C" fn HmGui_DrawGroup(g: *mut HmGuiGroup, hmgui_focus: u64) {
    // #if HMGUI_DRAW_GROUP_FRAMES
    //   Draw_Color(0.2f, 0.2f, 0.2f, 0.5f);
    //   Draw_Border(2.0f, g->pos.x, g->pos.y, g->size.x, g->size.y);
    // #endif

    UIRenderer_BeginLayer(
        (*g).widget.pos.x,
        (*g).widget.pos.y,
        (*g).widget.size.x,
        (*g).widget.size.y,
        (*g).clip,
    );

    let mut e: *mut HmGuiWidget = (*g).tail;
    while !e.is_null() {
        match (*e).ty {
            WidgetType::Group => {
                HmGui_DrawGroup(e as *mut HmGuiGroup, hmgui_focus);
            }
            WidgetType::Text => {
                HmGui_DrawText(e as *mut HmGuiText);
            }
            WidgetType::Rect => {
                HmGui_DrawRect(e as *mut HmGuiRect);
            }
            WidgetType::Image => {
                HmGui_DrawImage(e as *mut HmGuiImage);
            }
        }
        e = (*e).prev;
    }

    if (*g).focusable[FocusType::Mouse as usize] {
        let focus: bool = hmgui_focus == (*g).widget.hash;
        if (*g).focusStyle == FocusStyle::None {
            UIRenderer_Panel(
                (*g).widget.pos.x,
                (*g).widget.pos.y,
                (*g).widget.size.x,
                (*g).widget.size.y,
                0.1f32,
                0.12f32,
                0.13f32,
                1.0f32,
                8.0f32,
                (*g).frameOpacity,
            );
        } else if (*g).focusStyle == FocusStyle::Fill {
            if focus {
                UIRenderer_Panel(
                    (*g).widget.pos.x,
                    (*g).widget.pos.y,
                    (*g).widget.size.x,
                    (*g).widget.size.y,
                    0.1f32,
                    0.5f32,
                    1.0f32,
                    1.0f32,
                    0.0f32,
                    1.0f32,
                );
            } else {
                UIRenderer_Panel(
                    (*g).widget.pos.x,
                    (*g).widget.pos.y,
                    (*g).widget.size.x,
                    (*g).widget.size.y,
                    0.15f32,
                    0.15f32,
                    0.15f32,
                    0.8f32,
                    0.0f32,
                    (*g).frameOpacity,
                );
            }
        } else if (*g).focusStyle == FocusStyle::Outline {
            if focus {
                UIRenderer_Rect(
                    (*g).widget.pos.x,
                    (*g).widget.pos.y,
                    (*g).widget.size.x,
                    (*g).widget.size.y,
                    0.1f32,
                    0.5f32,
                    1.0f32,
                    1.0f32,
                    true,
                );
            }
        } else if (*g).focusStyle == FocusStyle::Underline {
            UIRenderer_Rect(
                (*g).widget.pos.x,
                (*g).widget.pos.y,
                (*g).widget.size.x,
                (*g).widget.size.y,
                0.3f32,
                0.3f32,
                0.3f32,
                if focus as i32 != 0 {
                    0.5f32
                } else {
                    (*g).frameOpacity
                },
                false,
            );
        }
    }
    UIRenderer_EndLayer();
}
