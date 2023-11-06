use std::{collections::HashMap, fs::File};

use glam::{Vec2, Vec4};

use super::*;

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub enum LayoutType {
    #[default]
    None,
    Stack,
    Horizontal,
    Vertical,
}

#[derive(Clone, Default, PartialEq)]
pub struct HmGuiContainer {
    pub children: Vec<Rf<HmGuiWidget>>,

    // Layout
    pub layout: LayoutType,
    pub children_docking: DockingType,
    pub padding_lower: Vec2,
    pub padding_upper: Vec2,
    pub spacing: f32,

    pub children_hash: u32,
    pub focus_style: FocusStyle,
    pub offset: Vec2,
    pub max_size: Vec2, // TODO: do we still need this?
    pub total_stretch: Vec2,
    pub frame_opacity: f32,
    pub clip: bool,
    pub focusable: [bool; 2],
    pub store_size: bool,
}

impl HmGuiContainer {
    /// Compute min size of the container widget.
    ///
    /// Go from the bottom to the top of widget's hierarchy tree to calculate widget min sizes.
    pub fn compute_size(&self, hmgui: &mut HmGui) -> Vec2 {
        for widget_rf in &self.children {
            widget_rf.as_mut().compute_size(hmgui);
        }

        let mut min_size = Vec2::ZERO;

        let mut not_head = false;
        for widget_rf in &self.children {
            let widget = widget_rf.as_ref();
            let widget_min_size = widget.min_size;

            match self.layout {
                LayoutType::None | LayoutType::Stack => {
                    min_size = min_size.max(widget_min_size);
                }
                LayoutType::Horizontal => {
                    min_size.x += widget_min_size.x;
                    min_size.y = min_size.y.max(widget_min_size.y);

                    if not_head {
                        min_size.x += self.spacing;
                    }
                }
                LayoutType::Vertical => {
                    min_size.x = min_size.x.max(widget_min_size.x);
                    min_size.y += widget_min_size.y;

                    if not_head {
                        min_size.y += self.spacing;
                    }
                }
            }

            not_head = true;
        }

        min_size += self.padding_lower + self.padding_upper;

        min_size //.min(self.max_size)
    }

    /// Go from the top to the bottom of the widget's hierarchy tree to calculate widget's pos and size.
    pub fn layout(&self, hmgui: &mut HmGui, pos: Vec2, size: Vec2, mut extra: Vec2) {
        let mut pos = pos + self.padding_lower + self.offset;
        let size = size - self.padding_lower - self.padding_upper;

        // Algorithm:
        // 1. Calculate percentage size of the children. Calculate for the widgets without horizontal/vertical stretching.
        // 2. Calculate extra space distribution. If there is extra space, distribute it between widgets with horizontal/vertical stretching.
        // 3. Recalculate widgets position and size.
        //
        // Layout logic per layout type:
        // 1. None layout (top level widget)
        //    - preserve left/top position
        //    - behavior is the same as for Stack layout
        // 2. Stack layout
        //    - if child widget has horizontal docking (left and/or right) then it's used in calculation
        //      otherwise container children horizontal docking is used (if defined).
        //      If neither widget nor container children dockings are set then widget is centered horizontally.
        //      If both, left and right, dockings are set then widget is stretched horizontally, otherwise min size is used.
        //    - same for vertical
        // 3. Vertical layout
        //    - horizontal layout is the same as horizontal layout for Stack
        //    - if no vertical docking set then all children are centered with min size each
        //    - if children docking has only top set then all children are moved to the top with min size each
        //    - same when only bottom docking set
        //    - if both, top and bottom, dockings are set then if there is extra vertical space and there are children without fixed height,
        //      then extra space is proportionally divided between them (subtracting widget margins and container spacing).
        //      If there is no extra space then widgets just centered vertically with min height (can go out of container's rect)
        //      TODO: take in account percent height length
        // 4. Horizontal layout
        //    - same as for the vertical

        // 1. Calculate percentage size of the children
        for widget_rf in &self.children {
            let mut widget = widget_rf.as_mut();

            // Docking stretch has priority over fixed/percentage size
            if !widget.docking.has_horizontal_stretch()
                && !self.children_docking.has_horizontal_stretch()
            {
                if let Some(Length::Percent(percent_width)) = widget.default_width {
                    let widget_width = size.x * percent_width / 100.0;

                    widget.inner_min_size.x = widget_width;
                    widget.min_size.x = widget_width
                        + widget.border_width * 2.0
                        + widget.margin_upper.x
                        + widget.margin_lower.x;

                    if self.layout == LayoutType::Horizontal {
                        extra.x -= widget_width;
                    }
                }
            }

            // Docking stretch has priority over fixed/percentage size
            if !widget.docking.has_vertical_stretch()
                && !self.children_docking.has_vertical_stretch()
            {
                if let Some(Length::Percent(percent_height)) = widget.default_height {
                    let widget_height = size.y * percent_height / 100.0;

                    widget.inner_min_size.y = widget_height;
                    widget.min_size.y = widget_height
                        + widget.border_width * 2.0
                        + widget.margin_upper.y
                        + widget.margin_lower.y;

                    if self.layout == LayoutType::Vertical {
                        extra.y -= widget_height;
                    }
                }
            }
        }

        // 2. Calculate per child extra space distribution
        let mut extra_size = vec![0.0; self.children.len()];

        if self.layout == LayoutType::Horizontal {
            let offset_pos = if extra.x > 0.0 {
                let mut total_weight = 0.0;

                for (i, widget_rf) in self.children.iter().enumerate() {
                    let widget = widget_rf.as_ref();

                    if widget.docking.has_horizontal_stretch()
                        || self.children_docking.has_horizontal_stretch()
                    {
                        let weight = 100.0; // TODO: use percent width if set

                        total_weight += weight;
                        extra_size[i] = extra.x * weight;
                    }
                }

                if total_weight > 0.0 {
                    extra_size.iter_mut().for_each(|d| *d /= total_weight);

                    false // Do not offset position - children will stretch to fill whole container width
                } else {
                    true // There are only fixed size children - center them
                }
            } else {
                true // children should be centered
            };

            if offset_pos {
                if self.children_docking.is_dock_left() == self.children_docking.is_dock_right() {
                    // Either stretch or no horizontal docking at all - center children
                    pos.x += extra.x / 2.0;
                } else if self.children_docking.is_dock_right() {
                    // Stick children to the right
                    pos.x += extra.x;
                }
            }
        } else if self.layout == LayoutType::Vertical {
            let offset_pos = if extra.y > 0.0 {
                let mut total_weight = 0.0;

                for (i, widget_rf) in self.children.iter().enumerate() {
                    let widget = widget_rf.as_ref();

                    if widget.docking.has_vertical_stretch()
                        || self.children_docking.has_vertical_stretch()
                    {
                        let weight = 100.0; // TODO: use percent height if set

                        total_weight += weight;
                        extra_size[i] = extra.y * weight;
                    }
                }

                if total_weight > 0.0 {
                    extra_size.iter_mut().for_each(|d| *d /= total_weight);

                    false // Do not offset position - children will stretch to fill whole container height
                } else {
                    true // There are only fixed size children - center them
                }
            } else {
                true // children should be centered
            };

            if offset_pos {
                if self.children_docking.is_dock_top() == self.children_docking.is_dock_bottom() {
                    // Either stretch or no vertical docking at all - center children
                    pos.y += extra.y / 2.0;
                } else if self.children_docking.is_dock_bottom() {
                    // Stick children to the bottom
                    pos.y += extra.y;
                }
            }
        }

        // 3. Recalculate widgets position and size
        for (i, widget_rf) in self.children.iter().enumerate() {
            let mut widget = widget_rf.as_mut();

            if self.layout == LayoutType::Horizontal {
                widget.pos.x = pos.x;
                widget.size.x = widget.min_size.x + extra_size[i];
                pos.x += widget.size.x + self.spacing;
            } else {
                if widget.docking.has_horizontal_stretch()
                    || self.children_docking.has_horizontal_stretch()
                {
                    // Stretch widget and center it in the parent one
                    // Widget can go out of parent scope
                    widget.pos.x = pos.x;
                    widget.size.x = size.x;
                } else if widget.docking.is_dock_right() || self.children_docking.is_dock_right() {
                    // Stick to right, size == min_size
                    widget.pos.x = pos.x + size.x - widget.min_size.x;
                    widget.size.x = widget.min_size.x;
                } else if !(widget.docking.is_dock_left() || self.children_docking.is_dock_left()) {
                    // Center widget, size == min_size
                    widget.pos.x = pos.x + (size.x - widget.min_size.x) / 2.0;
                    widget.size.x = widget.min_size.x;
                } else {
                    // Otherwise stick to the left
                    widget.pos.x = pos.x;
                    widget.size.x = widget.min_size.x;
                }
            }

            if self.layout == LayoutType::Vertical {
                widget.pos.y = pos.y;
                widget.size.y = widget.min_size.y + extra_size[i];
                pos.y += widget.size.y + self.spacing;
            } else {
                if widget.docking.has_vertical_stretch()
                    || self.children_docking.has_vertical_stretch()
                {
                    // Stretch widget and center it in the parent one
                    // Widget can go out of parent scope
                    widget.pos.y = pos.y;
                    widget.size.y = size.y;
                } else if widget.docking.is_dock_bottom() || self.children_docking.is_dock_bottom()
                {
                    // Stick to bottom, size == min_size
                    widget.pos.y = pos.y + size.y - widget.min_size.y;
                    widget.size.y = widget.min_size.y;
                } else if !(widget.docking.is_dock_top() || self.children_docking.is_dock_top()) {
                    // Center widget, size == min_size
                    widget.pos.y = pos.y + (size.y - widget.min_size.y) / 2.0;
                    widget.size.y = widget.min_size.y;
                } else {
                    // Otherwise stick to the top
                    widget.pos.y = pos.y;
                    widget.size.y = widget.min_size.y;
                }
            }

            widget.calculate_inner_pos_size();

            widget.layout(hmgui);
        }
    }

    pub fn draw(&self, hmgui: &mut HmGui, pos: Vec2, size: Vec2, focus: bool) {
        // #if HMGUI_DRAW_GROUP_FRAMES
        //   Draw_Color(0.2f, 0.2f, 0.2f, 0.5f);
        //   Draw_Border(2.0f, g->pos.x, g->pos.y, g->size.x, g->size.y);
        // #endif

        hmgui.renderer.begin_layer(pos, size, self.clip);

        for widget_rf in self.children.iter().rev() {
            widget_rf.as_ref().draw(hmgui);
        }

        if self.focusable[FocusType::Mouse as usize] {
            match self.focus_style {
                FocusStyle::None => {
                    let color = Vec4::new(0.1, 0.12, 0.13, 1.0);

                    hmgui
                        .renderer
                        .panel(pos, size, color, 8.0, self.frame_opacity);
                }
                FocusStyle::Fill => {
                    if focus {
                        let color = Vec4::new(0.1, 0.5, 1.0, 1.0);

                        hmgui.renderer.panel(pos, size, color, 0.0, 1.0);
                    } else {
                        let color = Vec4::new(0.15, 0.15, 0.15, 0.8);

                        hmgui
                            .renderer
                            .panel(pos, size, color, 0.0, self.frame_opacity);
                    }
                }
                FocusStyle::Outline => {
                    if focus {
                        let color = Vec4::new(0.1, 0.5, 1.0, 1.0);

                        hmgui.renderer.rect(pos, size, color, true);
                    }
                }
                FocusStyle::Underline => {
                    let color = Vec4::new(
                        0.3,
                        0.3,
                        0.3,
                        if focus as i32 != 0 {
                            0.5
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
    pub(crate) fn dump(&self, ident: usize, file: &mut Option<File>) {
        let ident_str = format!("{}", IDENT.repeat(ident));

        println!("{ident_str}- layout:           {:?}", self.layout);
        println!("{ident_str}- children_docking: {:?}", self.children_docking);
        println!("{ident_str}- padding_lower:    {:?}", self.padding_lower);
        println!("{ident_str}- padding_upper:    {:?}", self.padding_upper);
        println!("{ident_str}- spacing:          {}", self.spacing);
        println!("{ident_str}- children_hash:    {}", self.children_hash);
        println!("{ident_str}- focus_style:      {:?}", self.focus_style);
        println!("{ident_str}- max_size:         {:?}", self.max_size);
        println!("{ident_str}- total_stretch:    {:?}", self.total_stretch);
        println!("{ident_str}- frame_opacity:    {}", self.frame_opacity);
        println!("{ident_str}- clip:             {}", self.clip);
        println!("{ident_str}- focusable:        {:?}", self.focusable);
        println!("{ident_str}- store_size:       {:?}", self.store_size);
        println!("{ident_str}- children[{}]:", self.children.len());

        for head_rf in &self.children {
            head_rf.as_ref().dump(ident + 1, file);
        }
    }
}
