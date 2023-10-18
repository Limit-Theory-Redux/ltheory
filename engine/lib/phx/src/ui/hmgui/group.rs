use std::borrow::{Borrow, BorrowMut};

use glam::Vec2;
use internal::*;

use crate::render::{
    UIRenderer_BeginLayer, UIRenderer_EndLayer, UIRenderer_Panel, UIRenderer_Rect,
};

use super::*;

#[derive(Clone, Default, PartialEq)]
pub struct HmGuiGroup {
    pub widget: Rf<HmGuiWidget>,
    pub head: Option<Rf<HmGuiWidget>>,
    pub tail: Option<Rf<HmGuiWidget>>,

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

impl HmGuiGroup {
    pub fn compute_size(&mut self, hmgui: &mut HmGui) {
        let mut head_opt = self.head.clone();
        while let Some(head_widget) = head_opt {
            if let WidgetItem::Group(group) = &head_widget.as_ref().item {
                group.as_mut().compute_size(hmgui);
            }

            head_opt = head_widget.as_ref().next.clone();
        }

        let mut widget = self.widget.as_mut();
        widget.minSize = Vec2::ZERO;

        let mut head_opt = self.head.clone();
        while let Some(head_widget) = head_opt.clone() {
            let head = head_widget.as_mut();

            match self.layout {
                LayoutType::None => {}
                LayoutType::Stack => {
                    widget.minSize.x = f32::max(widget.minSize.x, head.minSize.x);
                    widget.minSize.y = f32::max(widget.minSize.y, head.minSize.y);
                }
                LayoutType::Vertical => {
                    widget.minSize.x = f32::max(widget.minSize.x, head.minSize.x);
                    widget.minSize.y += head.minSize.y;

                    if head_opt != self.head {
                        widget.minSize.y += self.spacing;
                    }
                }
                LayoutType::Horizontal => {
                    widget.minSize.x += head.minSize.x;
                    widget.minSize.y = f32::max(widget.minSize.y, head.minSize.y);

                    if head_opt != self.head {
                        widget.minSize.x += self.spacing;
                    }
                }
            }

            head_opt = head.next.clone();
        }

        widget.minSize.x += self.paddingLower.x + self.paddingUpper.x;
        widget.minSize.y += self.paddingLower.y + self.paddingUpper.y;

        widget.minSize.x = f32::min(widget.minSize.x, self.maxSize.x);
        widget.minSize.y = f32::min(widget.minSize.y, self.maxSize.y);

        if self.storeSize {
            let data = hmgui.get_data(widget.hash);

            data.minSize = widget.minSize;
        }
    }

    pub fn layout(&self, hmgui: &mut HmGui) {
        let widget = self.widget.as_mut();

        let mut pos = widget.pos;
        let mut size = widget.size;
        let mut extra: f32 = 0.0f32;
        let mut totalStretch: f32 = 0.0f32;

        pos.x += self.paddingLower.x + self.offset.x;
        pos.y += self.paddingLower.y + self.offset.y;
        size.x -= self.paddingLower.x + self.paddingUpper.x;
        size.y -= self.paddingLower.y + self.paddingUpper.y;

        if self.expand {
            if self.layout == LayoutType::Vertical {
                extra = widget.size.y - widget.minSize.y;

                let mut head_opt = self.head.clone();
                while let Some(head_rf) = head_opt {
                    let head = head_rf.as_ref();

                    totalStretch += head.stretch.y;
                    head_opt = head.next.clone();
                }
            } else if self.layout == LayoutType::Horizontal {
                extra = widget.size.x - widget.minSize.x;

                let mut head_opt = self.head.clone();
                while let Some(head_rf) = head_opt {
                    let head = head_rf.as_ref();

                    totalStretch += head.stretch.x;
                    head_opt = head.next.clone();
                }
            }

            if totalStretch > 0.0f32 {
                extra /= totalStretch;
            }
        }

        let mut head_opt = self.head.clone();
        while let Some(head_rf) = head_opt {
            let mut head = head_rf.as_mut();

            match self.layout {
                LayoutType::None => {
                    let pos = head.pos;
                    head.layout(pos, size.x, size.y);
                }
                LayoutType::Stack => {
                    head.layout(pos, size.x, size.y);
                }
                LayoutType::Vertical => {
                    let mut s = head.minSize.y;
                    if extra > 0.0f32 {
                        s += head.stretch.y * extra;
                    }
                    head.layout(pos, size.x, s);
                    pos.y += head.size.y + self.spacing;
                }
                LayoutType::Horizontal => {
                    let mut s = head.minSize.x;
                    if extra > 0.0f32 {
                        s += head.stretch.x * extra;
                    }
                    head.layout(pos, s, size.y);
                    pos.x += head.size.x + self.spacing;
                }
            }

            if let WidgetItem::Group(group_rf) = &head.item {
                let group = group_rf.as_ref();

                group.layout(hmgui);
            }

            head_opt = head.next.clone();
        }

        if self.storeSize {
            let data = hmgui.get_data(widget.hash);

            data.size = widget.size;
        }
    }

    pub fn draw(&self, hmgui: &mut HmGui, hmgui_focus: u64) {
        let widget = self.widget.as_mut();

        // #if HMGUI_DRAW_GROUP_FRAMES
        //   Draw_Color(0.2f, 0.2f, 0.2f, 0.5f);
        //   Draw_Border(2.0f, g->pos.x, g->pos.y, g->size.x, g->size.y);
        // #endif

        unsafe {
            UIRenderer_BeginLayer(
                widget.pos.x,
                widget.pos.y,
                widget.size.x,
                widget.size.y,
                self.clip,
            )
        };

        let mut tail_opt = self.tail.clone();
        while let Some(tail_rf) = tail_opt {
            let mut tail = tail_rf.as_mut();

            match &mut tail.item {
                WidgetItem::Undefined => {}
                WidgetItem::Group(group_rf) => {
                    let group = group_rf.as_mut();

                    group.draw(hmgui, hmgui_focus);
                }
                WidgetItem::Text(item) => {
                    item.draw();
                }
                WidgetItem::Rect(item) => {
                    item.draw();
                }
                WidgetItem::Image(item) => {
                    item.draw();
                }
            }

            tail_opt = tail.prev.clone();
        }

        if self.focusable[FocusType::Mouse as usize] {
            let focus: bool = hmgui_focus == widget.hash;

            unsafe {
                match self.focusStyle {
                    FocusStyle::None => {
                        UIRenderer_Panel(
                            widget.pos.x,
                            widget.pos.y,
                            widget.size.x,
                            widget.size.y,
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
                                widget.pos.x,
                                widget.pos.y,
                                widget.size.x,
                                widget.size.y,
                                0.1f32,
                                0.5f32,
                                1.0f32,
                                1.0f32,
                                0.0f32,
                                1.0f32,
                            );
                        } else {
                            UIRenderer_Panel(
                                widget.pos.x,
                                widget.pos.y,
                                widget.size.x,
                                widget.size.y,
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
                                widget.pos.x,
                                widget.pos.y,
                                widget.size.x,
                                widget.size.y,
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
                            widget.pos.x,
                            widget.pos.y,
                            widget.size.x,
                            widget.size.y,
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
        }

        unsafe { UIRenderer_EndLayer() };
    }
}
