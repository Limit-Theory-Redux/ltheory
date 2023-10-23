use std::borrow::{Borrow, BorrowMut};

use glam::Vec2;
use internal::*;

use crate::render::{
    UIRenderer_BeginLayer, UIRenderer_EndLayer, UIRenderer_Panel, UIRenderer_Rect,
};

use super::*;

#[derive(Clone, Default, PartialEq)]
pub struct HmGuiGroup {
    pub children: Vec<Rf<HmGuiWidget>>,

    pub layout: LayoutType,
    pub children_hash: u32,
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
    pub fn compute_size(&self, hmgui: &mut HmGui, minSize: &mut Vec2) {
        let mut head_opt = self.children.first().cloned();
        while let Some(head_rf) = head_opt {
            let mut head = head_rf.as_mut();

            head.compute_size(hmgui);

            head_opt = head.next.clone();
        }

        let mut head_opt = self.children.first().cloned();
        let mut not_head = false;
        while let Some(head_rf) = head_opt.clone() {
            let head = head_rf.as_ref();

            match self.layout {
                LayoutType::None => {}
                LayoutType::Stack => {
                    minSize.x = f32::max(minSize.x, head.minSize.x);
                    minSize.y = f32::max(minSize.y, head.minSize.y);
                }
                LayoutType::Vertical => {
                    minSize.x = f32::max(minSize.x, head.minSize.x);
                    minSize.y += head.minSize.y;

                    if not_head {
                        minSize.y += self.spacing;
                    }
                }
                LayoutType::Horizontal => {
                    minSize.x += head.minSize.x;
                    minSize.y = f32::max(minSize.y, head.minSize.y);

                    if not_head {
                        minSize.x += self.spacing;
                    }
                }
            }

            head_opt = head.next.clone();
            not_head = true;
        }

        minSize.x += self.paddingLower.x + self.paddingUpper.x;
        minSize.y += self.paddingLower.y + self.paddingUpper.y;

        minSize.x = f32::min(minSize.x, self.maxSize.x);
        minSize.y = f32::min(minSize.y, self.maxSize.y);
    }

    pub fn layout(&self, hmgui: &mut HmGui, mut pos: Vec2, mut size: Vec2, extra: Vec2) {
        let mut extra_dim: f32 = 0.0f32;
        let mut totalStretch: f32 = 0.0f32;

        pos.x += self.paddingLower.x + self.offset.x;
        pos.y += self.paddingLower.y + self.offset.y;
        size.x -= self.paddingLower.x + self.paddingUpper.x;
        size.y -= self.paddingLower.y + self.paddingUpper.y;

        if self.expand {
            if self.layout == LayoutType::Vertical {
                extra_dim = extra.y;

                let mut head_opt = self.children.first().cloned();
                while let Some(head_rf) = head_opt {
                    let head = head_rf.as_ref();

                    totalStretch += head.stretch.y;
                    head_opt = head.next.clone();
                }
            } else if self.layout == LayoutType::Horizontal {
                extra_dim = extra.x;

                let mut head_opt = self.children.first().cloned();
                while let Some(head_rf) = head_opt {
                    let head = head_rf.as_ref();

                    totalStretch += head.stretch.x;
                    head_opt = head.next.clone();
                }
            }

            if totalStretch > 0.0f32 {
                extra_dim /= totalStretch;
            }
        }

        let mut head_opt = self.children.first().cloned();
        while let Some(head_rf) = head_opt {
            let mut head = head_rf.as_mut();

            match self.layout {
                LayoutType::None => {
                    let pos = head.pos;
                    head.layout_item(pos, size.x, size.y);
                }
                LayoutType::Stack => {
                    head.layout_item(pos, size.x, size.y);
                }
                LayoutType::Vertical => {
                    let mut s = head.minSize.y;
                    if extra_dim > 0.0f32 {
                        s += head.stretch.y * extra_dim;
                    }
                    head.layout_item(pos, size.x, s);
                    pos.y += head.size.y + self.spacing;
                }
                LayoutType::Horizontal => {
                    let mut s = head.minSize.x;
                    if extra_dim > 0.0f32 {
                        s += head.stretch.x * extra_dim;
                    }
                    head.layout_item(pos, s, size.y);
                    pos.x += head.size.x + self.spacing;
                }
            }

            head.layout(hmgui);

            head_opt = head.next.clone();
        }
    }

    pub fn draw(&self, hmgui: &mut HmGui, pos: Vec2, size: Vec2, focus: bool) {
        // #if HMGUI_DRAW_GROUP_FRAMES
        //   Draw_Color(0.2f, 0.2f, 0.2f, 0.5f);
        //   Draw_Border(2.0f, g->pos.x, g->pos.y, g->size.x, g->size.y);
        // #endif

        unsafe { UIRenderer_BeginLayer(pos.x, pos.y, size.x, size.y, self.clip) };

        let mut tail_opt = self.children.last().cloned();
        while let Some(tail_rf) = tail_opt {
            let tail = tail_rf.as_ref();

            tail.draw(hmgui);

            tail_opt = tail.prev.clone();
        }

        if self.focusable[FocusType::Mouse as usize] {
            unsafe {
                match self.focusStyle {
                    FocusStyle::None => {
                        UIRenderer_Panel(
                            pos.x,
                            pos.y,
                            size.x,
                            size.y,
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
                                pos.x, pos.y, size.x, size.y, 0.1f32, 0.5f32, 1.0f32, 1.0f32,
                                0.0f32, 1.0f32,
                            );
                        } else {
                            UIRenderer_Panel(
                                pos.x,
                                pos.y,
                                size.x,
                                size.y,
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
                                pos.x, pos.y, size.x, size.y, 0.1f32, 0.5f32, 1.0f32, 1.0f32, true,
                            );
                        }
                    }
                    FocusStyle::Underline => {
                        UIRenderer_Rect(
                            pos.x,
                            pos.y,
                            size.x,
                            size.y,
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

    // For testing.
    #[allow(dead_code)]
    pub(crate) fn dump(&self, ident: usize, file: &mut File) {
        let ident_str = format!("{}", IDENT.repeat(ident));

        println!("{ident_str}- layout:        {:?}", self.layout);
        println!("{ident_str}- children_hash: {}", self.children_hash);
        println!("{ident_str}- focus_style:   {:?}", self.focusStyle);
        println!("{ident_str}- padding_lower: {:?}", self.paddingLower);
        println!("{ident_str}- padding_upper: {:?}", self.paddingUpper);
        println!("{ident_str}- max_size:      {:?}", self.maxSize);
        println!("{ident_str}- total_stretch: {:?}", self.totalStretch);
        println!("{ident_str}- spacing:       {}", self.spacing);
        println!("{ident_str}- frame_opacity: {}", self.frameOpacity);
        println!("{ident_str}- clip:          {}", self.clip);
        println!("{ident_str}- expand:        {:?}", self.expand);
        println!("{ident_str}- focusable:     {:?}", self.focusable);
        println!("{ident_str}- store_size:    {:?}", self.storeSize);
        println!("{ident_str}- content:");

        for head_rf in &self.children {
            head_rf.as_ref().dump(ident + 1, file);
        }
    }
}
