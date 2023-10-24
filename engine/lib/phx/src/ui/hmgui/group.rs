use glam::Vec2;

use super::*;

#[derive(Clone, Default, PartialEq)]
pub struct HmGuiGroup {
    pub children: Vec<Rf<HmGuiWidget>>,

    pub layout: LayoutType,
    pub children_hash: u32,
    pub focus_style: FocusStyle,
    pub padding_lower: Vec2,
    pub padding_upper: Vec2,
    pub offset: Vec2,
    pub max_size: Vec2,
    pub total_stretch: Vec2,
    pub spacing: f32,
    pub frame_opacity: f32,
    pub clip: bool,
    pub expand: bool,
    pub focusable: [bool; 2],
    pub store_size: bool,
}

impl HmGuiGroup {
    pub fn compute_size(&self, hmgui: &mut HmGui, min_size: &mut Vec2) {
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
                    min_size.x = f32::max(min_size.x, head.min_size.x);
                    min_size.y = f32::max(min_size.y, head.min_size.y);
                }
                LayoutType::Vertical => {
                    min_size.x = f32::max(min_size.x, head.min_size.x);
                    min_size.y += head.min_size.y;

                    if not_head {
                        min_size.y += self.spacing;
                    }
                }
                LayoutType::Horizontal => {
                    min_size.x += head.min_size.x;
                    min_size.y = f32::max(min_size.y, head.min_size.y);

                    if not_head {
                        min_size.x += self.spacing;
                    }
                }
            }

            head_opt = head.next.clone();
            not_head = true;
        }

        min_size.x += self.padding_lower.x + self.padding_upper.x;
        min_size.y += self.padding_lower.y + self.padding_upper.y;

        min_size.x = f32::min(min_size.x, self.max_size.x);
        min_size.y = f32::min(min_size.y, self.max_size.y);
    }

    pub fn layout(&self, hmgui: &mut HmGui, mut pos: Vec2, mut size: Vec2, extra: Vec2) {
        let mut extra_dim: f32 = 0.0f32;
        let mut total_stretch: f32 = 0.0f32;

        pos.x += self.padding_lower.x + self.offset.x;
        pos.y += self.padding_lower.y + self.offset.y;
        size.x -= self.padding_lower.x + self.padding_upper.x;
        size.y -= self.padding_lower.y + self.padding_upper.y;

        if self.expand {
            if self.layout == LayoutType::Vertical {
                extra_dim = extra.y;

                let mut head_opt = self.children.first().cloned();
                while let Some(head_rf) = head_opt {
                    let head = head_rf.as_ref();

                    total_stretch += head.stretch.y;
                    head_opt = head.next.clone();
                }
            } else if self.layout == LayoutType::Horizontal {
                extra_dim = extra.x;

                let mut head_opt = self.children.first().cloned();
                while let Some(head_rf) = head_opt {
                    let head = head_rf.as_ref();

                    total_stretch += head.stretch.x;
                    head_opt = head.next.clone();
                }
            }

            if total_stretch > 0.0f32 {
                extra_dim /= total_stretch;
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
                    let mut s = head.min_size.y;
                    if extra_dim > 0.0f32 {
                        s += head.stretch.y * extra_dim;
                    }
                    head.layout_item(pos, size.x, s);
                    pos.y += head.size.y + self.spacing;
                }
                LayoutType::Horizontal => {
                    let mut s = head.min_size.x;
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

        hmgui.renderer.begin_layer(pos, size, self.clip);

        let mut tail_opt = self.children.last().cloned();
        while let Some(tail_rf) = tail_opt {
            let tail = tail_rf.as_ref();

            tail.draw(hmgui);

            tail_opt = tail.prev.clone();
        }

        if self.focusable[FocusType::Mouse as usize] {
            match self.focus_style {
                FocusStyle::None => {
                    let color = Vec4::new(0.1f32, 0.12f32, 0.13f32, 1.0f32);

                    hmgui
                        .renderer
                        .panel(pos, size, color, 8.0f32, self.frame_opacity);
                }
                FocusStyle::Fill => {
                    if focus {
                        let color = Vec4::new(0.1f32, 0.5f32, 1.0f32, 1.0f32);

                        hmgui.renderer.panel(pos, size, color, 0.0f32, 1.0f32);
                    } else {
                        let color = Vec4::new(0.15f32, 0.15f32, 0.15f32, 0.8f32);

                        hmgui
                            .renderer
                            .panel(pos, size, color, 0.0f32, self.frame_opacity);
                    }
                }
                FocusStyle::Outline => {
                    if focus {
                        let color = Vec4::new(0.1f32, 0.5f32, 1.0f32, 1.0f32);

                        hmgui.renderer.rect(pos, size, color, true);
                    }
                }
                FocusStyle::Underline => {
                    let color = Vec4::new(
                        0.3f32,
                        0.3f32,
                        0.3f32,
                        if focus as i32 != 0 {
                            0.5f32
                        } else {
                            self.frame_opacity
                        },
                    );

                    hmgui.renderer.rect(pos, size, color, false);
                }
            }
        }

        hmgui.renderer.end_layer();
    }

    // For testing.
    #[allow(dead_code)]
    pub(crate) fn dump(&self, ident: usize, file: &mut File) {
        let ident_str = format!("{}", IDENT.repeat(ident));

        println!("{ident_str}- layout:        {:?}", self.layout);
        println!("{ident_str}- children_hash: {}", self.children_hash);
        println!("{ident_str}- focus_style:   {:?}", self.focus_style);
        println!("{ident_str}- padding_lower: {:?}", self.padding_lower);
        println!("{ident_str}- padding_upper: {:?}", self.padding_upper);
        println!("{ident_str}- max_size:      {:?}", self.max_size);
        println!("{ident_str}- total_stretch: {:?}", self.total_stretch);
        println!("{ident_str}- spacing:       {}", self.spacing);
        println!("{ident_str}- frame_opacity: {}", self.frame_opacity);
        println!("{ident_str}- clip:          {}", self.clip);
        println!("{ident_str}- expand:        {:?}", self.expand);
        println!("{ident_str}- focusable:     {:?}", self.focusable);
        println!("{ident_str}- store_size:    {:?}", self.store_size);
        println!("{ident_str}- content:");

        for head_rf in &self.children {
            head_rf.as_ref().dump(ident + 1, file);
        }
    }
}
