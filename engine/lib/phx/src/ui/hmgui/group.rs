use glam::Vec2;
use internal::*;

use crate::render::{
    UIRenderer_BeginLayer, UIRenderer_EndLayer, UIRenderer_Panel, UIRenderer_Rect,
};

use super::data::*;
use super::focus::*;
use super::image::*;
use super::rect::*;
use super::text::*;
use super::widget::*;
use super::HmGui;
use super::HmGuiWidgetId;

#[derive(Clone, Default)]
pub struct HmGuiGroup {
    pub widget_id: HmGuiWidgetId,
    pub head_id: Option<HmGuiWidgetId>,
    pub tail_id: Option<HmGuiWidgetId>,

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
        let mut head_id = self.head_id;
        while let Some(id) = head_id {
            let head = hmgui.get_widget_mut(id);
            let next_id = head.next_id;

            if let WidgetItem::Group(id) = head.item {
                let group = hmgui.get_group_mut(id);

                group.compute_size(hmgui);
            }

            head_id = next_id;
        }

        let widget = hmgui.get_widget_mut(self.widget_id);

        widget.minSize = Vec2::ZERO;

        let mut head_id = self.head_id;
        while let Some(id) = head_id {
            let head = hmgui.get_widget_mut(id);

            match self.layout {
                LayoutType::None => {}
                LayoutType::Stack => {
                    widget.minSize.x = f32::max(widget.minSize.x, head.minSize.x);
                    widget.minSize.y = f32::max(widget.minSize.y, head.minSize.y);
                }
                LayoutType::Vertical => {
                    widget.minSize.x = f32::max(widget.minSize.x, head.minSize.x);
                    widget.minSize.y += head.minSize.y;

                    if head_id != self.head_id {
                        widget.minSize.y += self.spacing;
                    }
                }
                LayoutType::Horizontal => {
                    widget.minSize.x += head.minSize.x;
                    widget.minSize.y = f32::max(widget.minSize.y, head.minSize.y);

                    if head_id != self.head_id {
                        widget.minSize.x += self.spacing;
                    }
                }
            }
            head_id = head.next_id;
        }

        widget.minSize.x += self.paddingLower.x + self.paddingUpper.x;
        widget.minSize.y += self.paddingLower.y + self.paddingUpper.y;

        if self.storeSize {
            let data = hmgui.get_data(self);

            data.minSize = widget.minSize;
        }

        widget.minSize.x = f32::min(widget.minSize.x, self.maxSize.x);
        widget.minSize.y = f32::min(widget.minSize.y, self.maxSize.y);
    }

    pub fn layout(&self, hmgui: &mut HmGui) {
        let widget = hmgui.get_widget_mut(self.widget_id);

        let mut pos = widget.pos;
        let mut size = widget.size;
        let mut extra: f32 = 0.0f32;
        let mut totalStretch: f32 = 0.0f32;

        pos.x += self.paddingLower.x + self.offset.x;
        pos.y += self.paddingLower.y + self.offset.y;
        size.x -= self.paddingLower.x + self.paddingUpper.x;
        size.y -= self.paddingLower.y + self.paddingUpper.y;

        unsafe {
            if self.expand {
                if self.layout == LayoutType::Vertical {
                    extra = widget.size.y - widget.minSize.y;

                    let mut head_id = self.head_id;
                    while let Some(id) = head_id {
                        let head = hmgui.get_widget_mut(id);

                        totalStretch += head.stretch.y;
                        head_id = head.next_id;
                    }
                } else if self.layout == LayoutType::Horizontal {
                    extra = widget.size.x - widget.minSize.x;

                    let mut head_id = self.head_id;
                    while let Some(id) = head_id {
                        let head = hmgui.get_widget_mut(id);

                        totalStretch += head.stretch.x;
                        head_id = head.next_id;
                    }
                }

                if totalStretch > 0.0f32 {
                    extra /= totalStretch;
                }
            }

            let mut head_id = self.head_id;
            while let Some(id) = head_id {
                let head = hmgui.get_widget_mut(id);

                match self.layout {
                    LayoutType::None => {
                        head.layout(head.pos, size.x, size.y);
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

                if let WidgetItem::Group(id) = head.item {
                    let group = hmgui.get_group_mut(id);

                    group.layout(hmgui);
                }

                head_id = head.next_id;
            }

            if self.storeSize {
                let data = hmgui.get_data(self);

                data.size = widget.size;
            }
        }
    }

    pub fn draw(&self, hmgui: &mut HmGui, hmgui_focus: u64) {
        let widget = hmgui.get_widget_mut(self.widget_id);

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
            );

            let mut tail_id = self.tail_id;
            while let Some(id) = tail_id {
                let tail = hmgui.get_widget_mut(id);

                match &mut tail.item {
                    WidgetItem::Group(id) => {
                        let group = hmgui.get_group_mut(*id);

                        group.draw(hmgui, hmgui_focus);
                    }
                    WidgetItem::Text(item) => {
                        item.draw(hmgui);
                    }
                    WidgetItem::Rect(item) => {
                        item.draw(hmgui);
                    }
                    WidgetItem::Image(item) => {
                        item.draw(hmgui);
                    }
                }

                tail_id = tail.prev_id;
            }

            if self.focusable[FocusType::Mouse as usize] {
                let focus: bool = hmgui_focus == widget.hash;

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
            UIRenderer_EndLayer();
        }
    }
}
