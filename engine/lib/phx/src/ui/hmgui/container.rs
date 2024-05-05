use std::cell::RefMut;

use glam::Vec2;

use crate::rf::Rf;

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
    pub children_horizontal_alignment: AlignHorizontal,
    pub children_vertical_alignment: AlignVertical,
    pub padding_lower: Vec2,
    pub padding_upper: Vec2,
    pub spacing: f32,

    pub clip: bool,
    pub children_hash: u32,
    pub offset: Vec2, // TODO: move to widget?
    pub total_stretch: Vec2,
    pub scroll_dir: Option<ScrollDirection>,
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

        match self.layout {
            LayoutType::None | LayoutType::Stack => {
                for widget_rf in &self.children {
                    let widget = widget_rf.as_ref();
                    let widget_min_size = widget.min_size;

                    min_size = min_size.max(widget_min_size);
                }
            }
            LayoutType::Horizontal => {
                for (i, widget_rf) in self.children.iter().enumerate() {
                    let widget = widget_rf.as_ref();
                    let widget_min_size = widget.min_size;

                    min_size.x += widget_min_size.x;
                    min_size.y = min_size.y.max(widget_min_size.y);

                    if i > 0 {
                        min_size.x += self.spacing;
                    }
                }
            }
            LayoutType::Vertical => {
                for (i, widget_rf) in self.children.iter().enumerate() {
                    let widget = widget_rf.as_ref();
                    let widget_min_size = widget.min_size;

                    min_size.x = min_size.x.max(widget_min_size.x);
                    min_size.y += widget_min_size.y;

                    if i > 0 {
                        min_size.y += self.spacing;
                    }
                }
            }
        }

        min_size + self.padding_lower + self.padding_upper
    }

    /// Go from the top to the bottom of the widgets hierarchy tree to calculate their pos and size.
    pub fn layout(
        &self,
        hmgui: &mut HmGui,
        widget_hstretch: bool,
        widget_vstretch: bool,
        pos: Vec2,
        size: Vec2,
        mut extra: Vec2,
    ) -> Vec2 {
        let mut pos = pos + self.padding_lower + self.offset;
        let size = size - self.padding_lower - self.padding_upper;

        // 1. Calculate percentage size of the children
        if size.x > 0.0 || size.y > 0.0 {
            let mut extra_diff = Vec2::ZERO;

            let stretch_x = if self.layout == LayoutType::Horizontal {
                1.0
            } else {
                0.0
            };

            let stretch_y = if self.layout == LayoutType::Vertical {
                1.0
            } else {
                0.0
            };

            for widget_rf in &self.children {
                let mut widget = widget_rf.as_mut();

                // Horizontal.
                // Child docking stretch has priority over fixed/percentage size
                // that in turn has higher priority than children stretch.
                if size.x > 0.0 && !widget.horizontal_alignment.is_stretch() {
                    if let Some(Length::Percent(percent_width)) = widget.default_width {
                        let widget_width = size.x * percent_width / 100.0;

                        extra_diff.x += stretch_x * (widget_width - widget.min_size.x);

                        widget.min_size.x = widget_width;
                        widget.inner_min_size.x = widget_width
                            - widget.border_width * 2.0
                            - widget.margin_upper.x
                            - widget.margin_lower.x;
                    }
                }

                // Vertical.
                // Docking stretch has priority over fixed/percentage size
                // that in turn has higher priority than children stretch.
                if size.y > 0.0 && !widget.vertical_alignment.is_stretch() {
                    if let Some(Length::Percent(percent_height)) = widget.default_height {
                        let widget_height = size.y * percent_height / 100.0;

                        extra_diff.y += stretch_y * (widget_height - widget.min_size.y);

                        widget.min_size.y = widget_height;
                        widget.inner_min_size.y = widget_height
                            - widget.border_width * 2.0
                            - widget.margin_upper.y
                            - widget.margin_lower.y;
                    }
                }
            }

            extra -= extra_diff;
        }

        // 2. Calculate per child extra space distribution
        let mut extra_size = vec![0.0; self.children.len()];

        if self.layout == LayoutType::Horizontal {
            let offset_pos = if extra.x > 0.0 {
                let mut total_weight = 0.0;

                for (i, widget_rf) in self.children.iter().enumerate() {
                    let widget = widget_rf.as_ref();

                    if widget.horizontal_alignment.is_stretch() // child stretching has always priority
                        || widget.default_width.is_none() // fixed/percent size has priority over children docking
                            && self.children_horizontal_alignment.is_stretch()
                    {
                        let weight = 100.0; // weight per expandable widget

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
                if self.children_horizontal_alignment.is_center() {
                    // Either stretch or no horizontal docking at all - center children
                    pos.x += extra.x / 2.0;
                } else if self.children_horizontal_alignment.is_right() {
                    // Stick children to the right
                    pos.x += extra.x;
                }
            }
        } else if self.layout == LayoutType::Vertical {
            let offset_pos = if extra.y > 0.0 {
                let mut total_weight = 0.0;

                for (i, widget_rf) in self.children.iter().enumerate() {
                    let widget = widget_rf.as_ref();

                    if widget.vertical_alignment.is_stretch() // child stretching has always priority
                        || widget.default_height.is_none() // fixed/percent size has priority over children stretching
                            && self.children_vertical_alignment.is_stretch()
                    {
                        let weight = 100.0; // weight per expandable widget

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
                if self.children_vertical_alignment.is_center() {
                    // Either stretch or no vertical docking at all - center children
                    pos.y += extra.y / 2.0;
                } else if self.children_vertical_alignment.is_bottom() {
                    // Stick children to the bottom
                    pos.y += extra.y;
                }
            }
        }

        // 3. Recalculate widgets position and size
        let children_size = if widget_hstretch {
            if widget_vstretch {
                self.calculate_children_layout::<true, true>(hmgui, pos, size, extra_size)
            } else {
                self.calculate_children_layout::<true, false>(hmgui, pos, size, extra_size)
            }
        } else {
            if widget_vstretch {
                self.calculate_children_layout::<false, true>(hmgui, pos, size, extra_size)
            } else {
                self.calculate_children_layout::<false, false>(hmgui, pos, size, extra_size)
            }
        };

        children_size + self.padding_lower + self.padding_upper
    }

    fn calculate_children_layout<const HStretch: bool, const VStretch: bool>(
        &self,
        hmgui: &mut HmGui,
        mut pos: Vec2,
        size: Vec2,
        extra_size: Vec<f32>,
    ) -> Vec2 {
        let mut children_size = size;
        let mut spacing = 0.0;

        match self.layout {
            LayoutType::None => {
                for widget_rf in &self.children {
                    let mut widget = widget_rf.as_mut();

                    self.calculate_horizontal_layout(&mut widget, pos, size);
                    self.calculate_vertical_layout(&mut widget, pos, size);

                    widget.calculate_inner_pos_size();

                    widget.layout(hmgui);
                }
            }
            LayoutType::Stack => {
                for widget_rf in &self.children {
                    let mut widget = widget_rf.as_mut();

                    self.calculate_horizontal_layout(&mut widget, pos, size);
                    self.calculate_vertical_layout(&mut widget, pos, size);

                    widget.calculate_inner_pos_size();

                    widget.layout(hmgui);

                    if !HStretch {
                        children_size.x = children_size.x.max(widget.size.x);
                    }

                    if !VStretch {
                        children_size.y = children_size.y.max(widget.size.y);
                    }
                }
            }
            LayoutType::Horizontal => {
                children_size.x = 0.0;

                for (i, widget_rf) in self.children.iter().enumerate() {
                    let mut widget = widget_rf.as_mut();

                    widget.pos.x = pos.x;
                    widget.size.x = widget.min_size.x + extra_size[i];
                    pos.x += widget.size.x + self.spacing; // Calculate position of the next widget

                    self.calculate_vertical_layout(&mut widget, pos, size);

                    widget.calculate_inner_pos_size();

                    widget.layout(hmgui);

                    if !HStretch {
                        children_size.x += widget.size.x + spacing;
                        spacing = self.spacing;
                    }
                }

                children_size.x = children_size.x.max(size.x);
            }
            LayoutType::Vertical => {
                children_size.y = 0.0;

                for (i, widget_rf) in self.children.iter().enumerate() {
                    let mut widget = widget_rf.as_mut();

                    self.calculate_horizontal_layout(&mut widget, pos, size);

                    widget.pos.y = pos.y;
                    widget.size.y = widget.min_size.y + extra_size[i];
                    pos.y += widget.size.y + self.spacing; // Calculate position of the next widget

                    widget.calculate_inner_pos_size();

                    widget.layout(hmgui);

                    if !VStretch {
                        children_size.y += widget.size.y + spacing;
                        spacing = self.spacing;
                    }
                }

                children_size.y = children_size.y.max(size.y);
            }
        }

        children_size
    }

    fn calculate_horizontal_layout(
        &self,
        widget: &mut RefMut<'_, HmGuiWidget>,
        pos: Vec2,
        size: Vec2,
    ) {
        if widget.horizontal_alignment.is_default() {
            // Widget alignment has a priority, so when it's default we can use the children's alignment
            if self.children_horizontal_alignment.is_stretch() {
                // Stretch widget and center it in the parent one
                // Widget can go out of parent's scope
                widget.pos.x = pos.x;
                widget.size.x = size.x;
            } else if self.children_horizontal_alignment.is_center() {
                // Center widget, size == min_size
                widget.pos.x = pos.x + (size.x - widget.min_size.x) / 2.0;
                widget.size.x = widget.min_size.x;
            } else if self.children_horizontal_alignment.is_right() {
                // Stick to right, size == min_size
                widget.pos.x = pos.x + size.x - widget.min_size.x;
                widget.size.x = widget.min_size.x;
            } else {
                // Otherwise stick to the left
                widget.pos.x = pos.x;
                widget.size.x = widget.min_size.x;
            }
        } else {
            if widget.horizontal_alignment.is_stretch() {
                // Stretch widget and center it in the parent one
                // Widget can go out of parent's scope
                widget.pos.x = pos.x;
                widget.size.x = size.x;
            } else if widget.horizontal_alignment.is_center() {
                // Center widget, size == min_size
                widget.pos.x = pos.x + (size.x - widget.min_size.x) / 2.0;
                widget.size.x = widget.min_size.x;
            } else if widget.horizontal_alignment.is_right() {
                // Stick to right, size == min_size
                widget.pos.x = pos.x + size.x - widget.min_size.x;
                widget.size.x = widget.min_size.x;
            } else {
                // Otherwise stick to the left
                widget.pos.x = pos.x;
                widget.size.x = widget.min_size.x;
            }
        }
    }

    fn calculate_vertical_layout(
        &self,
        widget: &mut RefMut<'_, HmGuiWidget>,
        pos: Vec2,
        size: Vec2,
    ) {
        if widget.vertical_alignment.is_default() {
            // Widget alignment has a priority, so when it's default we can use the children's alignment
            if self.children_vertical_alignment.is_stretch() {
                // Stretch widget and center it in the parent one
                // Widget can go out of parent's scope
                widget.pos.y = pos.y;
                widget.size.y = size.y;
            } else if self.children_vertical_alignment.is_center() {
                // Center widget, size == min_size
                widget.pos.y = pos.y + (size.y - widget.min_size.y) / 2.0;
                widget.size.y = widget.min_size.y;
            } else if self.children_vertical_alignment.is_bottom() {
                // Stick to bottom, size == min_size
                widget.pos.y = pos.y + size.y - widget.min_size.y;
                widget.size.y = widget.min_size.y;
            } else {
                // Otherwise stick to the top
                widget.pos.y = pos.y;
                widget.size.y = widget.min_size.y;
            }
        } else {
            if widget.vertical_alignment.is_stretch() {
                // Stretch widget and center it in the parent one
                // Widget can go out of parent's scope
                widget.pos.y = pos.y;
                widget.size.y = size.y;
            } else if widget.vertical_alignment.is_center() {
                // Center widget, size == min_size
                widget.pos.y = pos.y + (size.y - widget.min_size.y) / 2.0;
                widget.size.y = widget.min_size.y;
            } else if widget.vertical_alignment.is_bottom() {
                // Stick to bottom, size == min_size
                widget.pos.y = pos.y + size.y - widget.min_size.y;
                widget.size.y = widget.min_size.y;
            } else {
                // Otherwise stick to the top
                widget.pos.y = pos.y;
                widget.size.y = widget.min_size.y;
            }
        }
    }

    pub fn draw(&self, hmgui: &mut HmGui, pos: Vec2, size: Vec2) {
        let clip = hmgui.get_property_bool(HmGuiProperties::ContainerClip.id());

        hmgui.renderer.begin_layer(pos, size, clip);

        for widget_rf in self.children.iter().rev() {
            widget_rf.as_ref().draw(hmgui);
        }

        hmgui.renderer.end_layer();
    }

    // For testing.
    #[allow(dead_code)]
    #[rustfmt::skip]
    pub(crate) fn dump(&self, ident: usize) {
        let ident_str = format!("{}", IDENT.repeat(ident));

        println!("{ident_str}- layout:           {:?}", self.layout);
        println!("{ident_str}- children_halign:  {:?}", self.children_horizontal_alignment);
        println!("{ident_str}- children_valign:  {:?}", self.children_vertical_alignment);
        println!("{ident_str}- padding_lower:    {:?}", self.padding_lower);
        println!("{ident_str}- padding_upper:    {:?}", self.padding_upper);
        println!("{ident_str}- spacing:          {}", self.spacing);
        println!("{ident_str}- children_hash:    {}", self.children_hash);
        println!("{ident_str}- total_stretch:    {:?}", self.total_stretch);
        println!("{ident_str}- scroll_dir:       {:?}", self.scroll_dir);
        println!("{ident_str}- children[{}]:", self.children.len());

        for head_rf in &self.children {
            head_rf.as_ref().dump(ident + 1);
        }
    }
}
