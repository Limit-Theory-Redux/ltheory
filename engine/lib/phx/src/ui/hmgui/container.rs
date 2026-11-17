use std::cell::{Ref, RefMut};

use glam::Vec2;

use super::*;
use crate::input::Input;
use crate::render::Renderer;
use crate::rf::Rf;

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

#[derive(Clone)]
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
}

impl Default for HmGuiContainer {
    fn default() -> Self {
        Self {
            children: Default::default(),
            layout: Default::default(),
            children_alignment: Default::default(),
            padding_lower: Default::default(),
            padding_upper: Default::default(),
            spacing: Default::default(),
            clip: true,
            children_hash: Default::default(),
            offset: Default::default(),
        }
    }
}

impl HmGuiContainer {
    /// Compute min size of the container widget.
    ///
    /// Go from the bottom to the top of widget's hierarchy tree to calculate widget min sizes.
    pub fn compute_size(&self, hmgui: &mut HmGui, r: &mut Renderer) -> Vec2 {
        for widget_rf in &self.children {
            widget_rf.as_mut().compute_size(hmgui, r);
        }

        // Positioned children (`set_position`) are out of the layout: they don't
        // make the container bigger.
        let flow_children: Vec<Ref<'_, HmGuiWidget>> = self
            .children
            .iter()
            .map(|widget_rf| widget_rf.as_ref())
            .filter(|widget| widget.position.is_none())
            .collect();

        let min_size = match self.layout {
            LayoutType::Stack => Vec2::new(
                self.min_size_across::<0>(&flow_children),
                self.min_size_across::<1>(&flow_children),
            ),
            LayoutType::Horizontal => Vec2::new(
                self.min_size_along::<0>(&flow_children),
                self.min_size_across::<1>(&flow_children),
            ),
            LayoutType::Vertical => Vec2::new(
                self.min_size_across::<0>(&flow_children),
                self.min_size_along::<1>(&flow_children),
            ),
        };

        min_size + self.padding_lower + self.padding_upper
    }

    /// Min size along the layout direction: the children one after another with
    /// the spacing between them, and enough for every percent-sized child's
    /// content to fit at its percentage.
    fn min_size_along<const DIM: usize>(&self, children: &[Ref<'_, HmGuiWidget>]) -> f32 {
        let mut size = 0.0;
        let mut needed: f32 = 0.0;

        for (i, widget) in children.iter().enumerate() {
            if i > 0 {
                size += self.spacing;
            }
            size += widget.min_size[DIM];

            if let Some(fraction) = self.percent_fraction::<DIM>(widget, true) {
                needed = needed.max(Self::size_for_percent_child::<DIM>(widget, fraction));
            }
        }

        size.max(needed)
    }

    /// Min size across the layout direction (in both directions in a stack): the
    /// biggest child, and enough for every percent-sized child's content to fit
    /// at its percentage.
    fn min_size_across<const DIM: usize>(&self, children: &[Ref<'_, HmGuiWidget>]) -> f32 {
        let mut size: f32 = 0.0;

        for widget in children {
            size = size.max(widget.min_size[DIM]);

            if let Some(fraction) = self.percent_fraction::<DIM>(widget, false) {
                size = size.max(Self::size_for_percent_child::<DIM>(widget, fraction));
            }
        }

        size
    }

    /// The fraction of this container a child takes in a dimension, if its
    /// percent size is used there. The child's own stretch/expand alignment wins
    /// over a percentage; across the layout direction (and in a stack) so does a
    /// stretch it takes from the children alignment (as in `calculate_layout`).
    fn percent_fraction<const DIM: usize>(&self, widget: &HmGuiWidget, along: bool) -> Option<f32> {
        let Length::Percent(percent) = widget.default_size[DIM] else {
            return None;
        };

        if widget.alignment[DIM].is_extend() {
            return None;
        }

        if !along && widget.alignment[DIM].is_default() && self.children_alignment[DIM].is_stretch()
        {
            return None;
        }

        Some(percent / 100.0)
    }

    /// How big this container has to be for a percent-sized child's content to
    /// fit at its percentage: a content-sized container makes room for it, like
    /// a CSS grid sizing its `fr` tracks to their content. Rects, images and
    /// shapes have no content of their own, so only their min size request,
    /// border and margins count (a 1 % fill bar doesn't make its parent a
    /// hundred times its default size).
    fn size_for_percent_child<const DIM: usize>(widget: &HmGuiWidget, fraction: f32) -> f32 {
        if fraction <= 0.0 {
            return 0.0;
        }

        let content = match &widget.item {
            WidgetItem::Rect | WidgetItem::Image(_) | WidgetItem::Shape(_) => {
                widget.min_size_request[DIM] + widget.outer_extra_dim::<DIM>()
            }
            _ => widget.min_size[DIM],
        };

        content / fraction
    }

    /// Go from the top to the bottom of the widgets hierarchy tree to calculate their pos and size.
    pub fn layout(
        &self,
        hmgui: &mut HmGui,
        r: &mut Renderer,
        input: &Input,
        widget_hstretch: bool,
        widget_vstretch: bool,
        pos: Vec2,
        size: Vec2,
    ) -> Vec2 {
        // positioned children are placed from the top left inside the border
        let positioned_origin = pos + self.offset;
        let positioned_area = size;

        let mut pos = pos + self.padding_lower + self.offset;
        let size = size - self.padding_lower - self.padding_upper;

        // 1. Resolve the children's percentage sizes against this container's
        //    inner size (its content box: the containing block, as in CSS)
        if size.x > 0.0 || size.y > 0.0 {
            for widget_rf in &self.children {
                let mut widget = widget_rf.as_mut();

                if widget.position.is_some() {
                    // a percentage of the area it's placed in; takes no space
                    Self::widget_percent_size::<0>(&mut widget, positioned_area.x);
                    Self::widget_percent_size::<1>(&mut widget, positioned_area.y);
                } else {
                    Self::widget_percent_size::<0>(&mut widget, size.x);
                    Self::widget_percent_size::<1>(&mut widget, size.y);
                }
            }
        }

        // 2. Share the space the children leave along the layout direction
        let extra_size = if self.layout.is_horizontal() {
            let extra = size.x - self.children_size_along::<0>();
            self.children_extra_size::<0>(&mut pos.x, extra)
        } else if self.layout.is_vertical() {
            let extra = size.y - self.children_size_along::<1>();
            self.children_extra_size::<1>(&mut pos.y, extra)
        } else {
            vec![0.0; self.children.len()]
        };

        // 3. Recalculate widgets position and size
        let children_size = if widget_hstretch {
            if widget_vstretch {
                self.calculate_children_layout::<true, true>(hmgui, r, input, pos, size, extra_size)
            } else {
                self.calculate_children_layout::<true, false>(
                    hmgui, r, input, pos, size, extra_size,
                )
            }
        } else if widget_vstretch {
            self.calculate_children_layout::<false, true>(hmgui, r, input, pos, size, extra_size)
        } else {
            self.calculate_children_layout::<false, false>(hmgui, r, input, pos, size, extra_size)
        };

        self.layout_positioned_children(hmgui, r, input, positioned_origin);

        children_size + self.padding_lower + self.padding_upper
    }

    /// Place the positioned children (`set_position`) at their positions, at
    /// their min (fixed, percent) size.
    fn layout_positioned_children(
        &self,
        hmgui: &mut HmGui,
        r: &mut Renderer,
        input: &Input,
        origin: Vec2,
    ) {
        for widget_rf in &self.children {
            let mut widget = widget_rf.as_mut();

            if let Some(position) = widget.position {
                widget.pos = origin + position;
                widget.size = widget.min_size;
                widget.layout(hmgui, r, input);
            }
        }
    }

    fn widget_percent_size<const DIM: usize>(widget: &mut HmGuiWidget, size: f32) {
        // Child docking stretch has priority over fixed/percentage size
        // that in turn has higher priority than children stretch.
        if size > 0.0 && !widget.alignment[DIM].is_extend() {
            if let Length::Percent(percent) = widget.default_size[DIM] {
                let widget_size = size * percent / 100.0;

                widget.min_size[DIM] = widget_size;
                widget.inner_min_size[DIM] = widget_size
                    - widget.border_width * 2.0
                    - widget.margin_upper[DIM]
                    - widget.margin_lower[DIM];
            }
        }
    }

    /// The size the flow children take along the layout direction at their min
    /// sizes (percentages resolved), with the spacing between them.
    fn children_size_along<const DIM: usize>(&self) -> f32 {
        let flow_children = self
            .children
            .iter()
            .filter(|widget_rf| widget_rf.as_ref().position.is_none());
        let mut size = 0.0;

        for (i, widget_rf) in flow_children.enumerate() {
            if i > 0 {
                size += self.spacing;
            }
            size += widget_rf.as_ref().min_size[DIM];
        }

        size
    }

    fn children_extra_size<const DIM: usize>(&self, pos: &mut f32, extra: f32) -> Vec<f32> {
        let mut extra_size = vec![0.0; self.children.len()];

        let offset_pos = if extra > 0.0 {
            let mut total_weight = 0.0;

            for (i, widget_rf) in self.children.iter().enumerate() {
                let widget = widget_rf.as_ref();

                if widget.position.is_some() {
                    continue; // out of the layout
                }

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

                if widget.position.is_some() {
                    continue; // out of the layout
                }

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

                false // Expandable widgets absorb the overflow - no offset needed
            } else {
                true // Nothing can shrink - center/end the overflowing content block
            }
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
        r: &mut Renderer,
        input: &Input,
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

                    if widget.position.is_some() {
                        continue; // placed by layout_positioned_children
                    }

                    self.calculate_layout::<0>(&mut widget, pos.x, size.x);
                    self.calculate_layout::<1>(&mut widget, pos.y, size.y);

                    widget.layout(hmgui, r, input);

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

                    if widget.position.is_some() {
                        continue; // placed by layout_positioned_children
                    }

                    widget.pos.x = pos.x;
                    widget.size.x = widget.min_size.x + extra_size[i];
                    pos.x += widget.size.x + self.spacing; // Calculate position of the next widget

                    self.calculate_layout::<1>(&mut widget, pos.y, size.y);

                    widget.layout(hmgui, r, input);

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

                    if widget.position.is_some() {
                        continue; // placed by layout_positioned_children
                    }

                    self.calculate_layout::<0>(&mut widget, pos.x, size.x);

                    widget.pos.y = pos.y;
                    widget.size.y = widget.min_size.y + extra_size[i];
                    pos.y += widget.size.y + self.spacing; // Calculate position of the next widget

                    widget.layout(hmgui, r, input);

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

        // In order: a later child is drawn over an earlier one, as the mouse
        // finds it first (`HmGui::check_mouse_over`).
        // TODO: [optimization] do not draw children outside of clipped container or screen
        for widget_rf in &self.children {
            widget_rf.as_ref().draw(hmgui);
        }

        hmgui.renderer.end_layer();
    }

    // For testing.
    #[allow(dead_code)]
    #[rustfmt::skip]
    pub(crate) fn dump(&self, ident: usize) {
        let ident_str = IDENT.repeat(ident).to_string();

        println!("{ident_str}- layout:         {:?}", self.layout);
        println!("{ident_str}- children_align: {:?}", self.children_alignment);
        println!("{ident_str}- padding_lower:  {:?}", self.padding_lower);
        println!("{ident_str}- padding_upper:  {:?}", self.padding_upper);
        println!("{ident_str}- spacing:        {}", self.spacing);
        println!("{ident_str}- children_hash:  {}", self.children_hash);
        println!("{ident_str}- children[{}]:", self.children.len());

        for head_rf in &self.children {
            head_rf.as_ref().dump("CHILDREN", ident + 1);
        }
    }
}
