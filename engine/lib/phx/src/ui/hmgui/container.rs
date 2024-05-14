use std::cell::RefMut;

use glam::Vec2;

use crate::rf::Rf;

use super::*;

/// Container element layout type.
#[luajit_ffi_gen::luajit_ffi(name = "GuiLayoutType")]
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub enum LayoutType {
    #[default]
    Stack,
    Horizontal,
    Vertical,
}

impl LayoutType {
    pub fn is_horizontal(&self) -> bool {
        *self == Self::Horizontal
    }

    pub fn is_vertical(&self) -> bool {
        *self == Self::Vertical
    }
}

#[derive(Clone, Default, PartialEq)]
pub struct HmGuiContainer {
    pub children: Vec<Rf<HmGuiWidget>>,

    // Layout
    pub layout: LayoutType,
    pub children_alignment: [Alignment; 2],
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
            LayoutType::Stack => {
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

            let stretch_x = if self.layout.is_horizontal() {
                1.0
            } else {
                0.0
            };
            let stretch_y = if self.layout.is_vertical() { 1.0 } else { 0.0 };

            for widget_rf in &self.children {
                let mut widget = widget_rf.as_mut();

                extra_diff.x += stretch_x * Self::widget_percent_size::<0>(&mut widget, size.x);
                extra_diff.y += stretch_y * Self::widget_percent_size::<1>(&mut widget, size.y);
            }

            extra -= extra_diff;
        }

        // 2. Calculate per child extra space distribution
        let extra_size = if self.layout.is_horizontal() {
            self.children_extra_size::<0>(&mut pos.x, extra.x)
        } else if self.layout.is_vertical() {
            self.children_extra_size::<1>(&mut pos.y, extra.y)
        } else {
            vec![0.0; self.children.len()]
        };

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

    fn widget_percent_size<const DIM: usize>(widget: &mut HmGuiWidget, size: f32) -> f32 {
        // Child docking stretch has priority over fixed/percentage size
        // that in turn has higher priority than children stretch.
        if size > 0.0 && !widget.alignment[DIM].is_extend() {
            if let Length::Percent(percent) = widget.default_size[DIM] {
                let widget_size = size * percent / 100.0;

                let extra_diff = widget_size - widget.min_size[DIM];

                widget.min_size[DIM] = widget_size;
                widget.inner_min_size[DIM] = widget_size
                    - widget.border_width * 2.0
                    - widget.margin_upper[DIM]
                    - widget.margin_lower[DIM];

                return extra_diff;
            }
        }

        0.0
    }

    fn children_extra_size<const DIM: usize>(&self, pos: &mut f32, extra: f32) -> Vec<f32> {
        let mut extra_size = vec![0.0; self.children.len()];

        let offset_pos = if extra > 0.0 {
            let mut total_weight = 0.0;

            for (i, widget_rf) in self.children.iter().enumerate() {
                let widget = widget_rf.as_ref();

                if widget.alignment[DIM].is_extend() // child stretching/expanding has always priority
                    || widget.default_size[DIM].is_auto() // fixed/percent size has priority over children docking
                        && self.children_alignment[DIM].is_extend()
                {
                    let weight = 100.0; // weight per extendable widget

                    total_weight += weight;
                    extra_size[i] = extra * weight;
                }
            }

            if total_weight > 0.0 {
                extra_size.iter_mut().for_each(|d| *d /= total_weight);

                false // Do not offset position - children will stretch to fill whole container size
            } else {
                true // There are only fixed size children - center them
            }
        } else if extra < 0.0 {
            // children overwhelm container -> try to shrink expandable widgets
            let mut total_weight = 0.0;

            for (i, widget_rf) in self.children.iter().enumerate() {
                let widget = widget_rf.as_ref();

                if widget.alignment[DIM].is_expand() // child expanding has always priority
                    || widget.default_size[DIM].is_auto() // fixed/percent size has priority over children docking
                        && self.children_alignment[DIM].is_expand()
                {
                    let weight = 100.0; // weight per expandable widget

                    total_weight += weight;
                    extra_size[i] = extra * weight;
                }
            }

            if total_weight > 0.0 {
                extra_size.iter_mut().for_each(|d| *d /= total_weight);
            }

            false // Do not offset position - children can still overwhelm container size
        } else {
            true // children should be centered
        };

        if offset_pos {
            if self.children_alignment[DIM].is_center() {
                // Either stretch or no docking at all - center children
                *pos += extra / 2.0;
            } else if self.children_alignment[DIM].is_end() {
                // Stick children to the right/bottom
                *pos += extra;
            }
        }

        extra_size
    }

    fn calculate_children_layout<const HSTRETCH: bool, const VSTRETCH: bool>(
        &self,
        hmgui: &mut HmGui,
        mut pos: Vec2,
        size: Vec2,
        extra_size: Vec<f32>,
    ) -> Vec2 {
        let mut children_size = size;
        let mut spacing = 0.0;

        match self.layout {
            LayoutType::Stack => {
                for widget_rf in &self.children {
                    let mut widget = widget_rf.as_mut();

                    self.calculate_layout::<0>(&mut widget, pos.x, size.x);
                    self.calculate_layout::<1>(&mut widget, pos.y, size.y);

                    widget.layout(hmgui);

                    if !HSTRETCH {
                        children_size.x = children_size.x.max(widget.size.x);
                    }

                    if !VSTRETCH {
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

                    self.calculate_layout::<1>(&mut widget, pos.y, size.y);

                    widget.layout(hmgui);

                    if !HSTRETCH {
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

                    self.calculate_layout::<0>(&mut widget, pos.x, size.x);

                    widget.pos.y = pos.y;
                    widget.size.y = widget.min_size.y + extra_size[i];
                    pos.y += widget.size.y + self.spacing; // Calculate position of the next widget

                    widget.layout(hmgui);

                    if !VSTRETCH {
                        children_size.y += widget.size.y + spacing;
                        spacing = self.spacing;
                    }
                }

                children_size.y = children_size.y.max(size.y);
            }
        }

        children_size
    }

    fn calculate_layout<const DIM: usize>(
        &self,
        widget: &mut RefMut<'_, HmGuiWidget>,
        pos: f32,
        size: f32,
    ) {
        if widget.alignment[DIM].is_default() {
            // Widget alignment has a priority, so when it's default we can use the children's alignment
            self.calculate_widget_layout::<DIM>(widget, self.children_alignment[DIM], pos, size);
        } else {
            self.calculate_widget_layout::<DIM>(widget, widget.alignment[DIM], pos, size);
        }
    }

    fn calculate_widget_layout<const DIM: usize>(
        &self,
        widget: &mut RefMut<'_, HmGuiWidget>,
        alignment: Alignment,
        pos: f32,
        size: f32,
    ) {
        if alignment.is_stretch() {
            // Stretch widget and center it in the parent one
            // Widget can go out of parent's scope
            widget.pos[DIM] = pos;
            widget.size[DIM] = size;
        } else if alignment.is_center() {
            // Center widget, size == min_size
            widget.pos[DIM] = pos + (size - widget.min_size[DIM]) / 2.0;
            widget.size[DIM] = widget.min_size[DIM];
        } else if alignment.is_end() {
            // Stick to right, size == min_size
            widget.pos[DIM] = pos + size - widget.min_size[DIM];
            widget.size[DIM] = widget.min_size[DIM];
        } else {
            // Otherwise stick to the left
            widget.pos[DIM] = pos;
            widget.size[DIM] = widget.min_size[DIM];
        }
    }

    pub fn draw(&self, hmgui: &mut HmGui, pos: Vec2, size: Vec2) {
        hmgui.renderer.begin_layer(pos, size, self.clip);

        // TODO: [optimization] do not draw children outside of clipped container or screen
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

        println!("{ident_str}- layout:         {:?}", self.layout);
        println!("{ident_str}- children_align: {:?}", self.children_alignment);
        println!("{ident_str}- padding_lower:  {:?}", self.padding_lower);
        println!("{ident_str}- padding_upper:  {:?}", self.padding_upper);
        println!("{ident_str}- spacing:        {}", self.spacing);
        println!("{ident_str}- children_hash:  {}", self.children_hash);
        println!("{ident_str}- total_stretch:  {:?}", self.total_stretch);
        println!("{ident_str}- scroll_dir:     {:?}", self.scroll_dir);
        println!("{ident_str}- children[{}]:", self.children.len());

        for head_rf in &self.children {
            head_rf.as_ref().dump("CHILDREN", ident + 1);
        }
    }
}
