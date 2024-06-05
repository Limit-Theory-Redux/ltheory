use std::collections::HashMap;

use glam::*;

use crate::common::*;
use crate::input::*;
use crate::render::*;
use crate::rf::Rf;
use crate::system::*;

use super::*;

pub struct HmGui {
    pub(super) renderer: UIRenderer,

    screen_size: Vec2,

    layers: Vec<HmGuiLayer>,
    /// Current layer index in layers vector
    layer_index: usize,

    data: HashMap<u64, HmGuiData>,
    mouse_over_widget_hash: [u64; 2],
    focus_pos: Vec2,
}

impl HmGui {
    pub fn new() -> Self {
        Self {
            renderer: Default::default(),
            screen_size: Default::default(),
            layers: vec![],
            layer_index: UNDEFINED_LAYER_INDEX,
            data: HashMap::with_capacity(128),
            mouse_over_widget_hash: [0; 2],
            focus_pos: Vec2::ZERO,
        }
    }

    #[inline]
    pub fn root(&self) -> Rf<HmGuiWidget> {
        self.layers[self.layer_index].root.clone()
    }

    #[inline]
    pub fn container(&self) -> Rf<HmGuiWidget> {
        self.layers[self.layer_index].container.clone()
    }

    #[inline]
    pub fn set_container(&mut self, container: Rf<HmGuiWidget>) {
        self.layers[self.layer_index].container = container;
    }

    #[inline]
    pub fn last(&self) -> Rf<HmGuiWidget> {
        self.layers[self.layer_index].last.clone()
    }

    #[inline]
    pub fn set_last(&mut self, last: Rf<HmGuiWidget>) {
        self.layers[self.layer_index].last = last;
    }

    #[inline]
    pub fn mouse_over_widget_hash(&self) -> u64 {
        self.mouse_over_widget_hash[FocusType::Mouse as usize]
    }

    /// Add a new widget into the current container.
    fn init_widget(&mut self, item: WidgetItem) -> Rf<HmGuiWidget> {
        let parent_rf = self.container();
        let mut parent = parent_rf.as_mut();
        let parent_hash = parent.hash;
        let parent_container = parent.get_container_item_mut();

        parent_container.children_hash = (parent_container.children_hash).wrapping_add(1);

        let mut widget = HmGuiWidget::new(Some(parent_rf.clone()), item);

        widget.hash = unsafe {
            Hash_FNV64_Incremental(
                parent_hash,
                &mut parent_container.children_hash as *mut u32 as *const _,
                std::mem::size_of::<u32>() as i32,
            )
        };

        let widget_rf = Rf::new(widget);

        parent_container.children.push(widget_rf.clone());

        self.set_last(widget_rf.clone());

        widget_rf.clone()
    }

    /// Get persistent data of the widget by its hash.
    pub fn get_data(&mut self, widget_hash: u64) -> &mut HmGuiData {
        self.data.entry(widget_hash).or_insert(HmGuiData::default())
    }

    /// Calculate if mouse is over the widget. Recursively iterate over container widgets.
    /// Setting mouse over hash at the end of the method guarantees that the last (top most) widget will get the mouse over flag set.
    // TODO: take in account container clipping
    fn check_mouse_over(&mut self, widget_rf: Rf<HmGuiWidget>) {
        let widget = widget_rf.as_ref();
        let is_mouse_over = widget.contains_point(&self.focus_pos);

        if let WidgetItem::Container(container) = &widget.item {
            if !container.clip || is_mouse_over {
                for widget_rf in container.children.iter().rev() {
                    self.check_mouse_over(widget_rf.clone());
                }
            }
        }

        if !is_mouse_over {
            return;
        }

        for i in 0..self.mouse_over_widget_hash.len() {
            // we need `self.mouse_over_widget_hash[i] == 0` check here to prevent parent container to overwrite
            // mouse over child situation
            if widget.mouse_over[i] && self.mouse_over_widget_hash[i] == 0 {
                self.mouse_over_widget_hash[i] = widget.hash;
            }
        }
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl HmGui {
    /// Begin GUI declaration. Region is limited by [0, 0] - [sx, sy] rectangle.
    pub fn begin_gui(&mut self, sx: f32, sy: f32) {
        // TODO: [optimization idea] do not clear all layers but reuse unchanged widgets
        self.layers.clear();
        self.screen_size = Vec2::new(sx, sy);

        self.beginLayer();
    }

    /// Finish GUI declaration, calculate hierarchy widgets sizes and layout.
    // TODO: do not calculate layout for the widgets that go out of the screen. If possible.
    pub fn end_gui(&mut self, input: &Input) {
        unsafe { Profiler_Begin(c_str!("HmGui_End")) };

        self.endLayer();
        assert_eq!(
            self.layer_index, UNDEFINED_LAYER_INDEX,
            "At least one beginLayer scope was not closed"
        );

        let layers_root: Vec<_> = self
            .layers
            .iter()
            .map(|layer| (layer.root.clone(), layer.location))
            .collect();
        for (root_rf, location) in &layers_root {
            let mut root = root_rf.as_mut();

            if let HmGuiLayerLocation::Below(widget_hash) = location {
                // we assume here that widget_hash points to the widget on the previous layer,
                // so its position and size are already calculated
                let data = self.get_data(*widget_hash);

                root.pos = Vec2::new(data.pos.x, data.pos.y + data.size.y);
                root.inner_pos = root.pos;
                root.size = self.screen_size - root.pos; // rest of the screen
                root.inner_size = root.size;
            }

            root.compute_size(self);
            root.layout(self);
        }

        self.focus_pos = input.mouse().position();
        self.mouse_over_widget_hash.fill(0);

        let layers_root: Vec<_> = self
            .layers
            .iter()
            .rev()
            .map(|layer| layer.root.clone())
            .collect();
        for root in layers_root {
            self.check_mouse_over(root);
        }

        unsafe { Profiler_End() };
    }

    /// Pass information about widgets to the renderer and draw them.
    // TODO: optimize - do not pass to the renderer widgets that are outside of the rendering region
    pub fn draw(&mut self) {
        unsafe {
            Profiler_Begin(c_str!("HmGui_Draw"));

            RenderState_PushBlendMode(1);
        }

        self.renderer.begin();

        let layers_root: Vec<_> = self
            .layers
            .iter()
            .rev()
            .map(|layer| layer.root.clone())
            .collect();
        for root_rf in layers_root {
            let root = root_rf.as_ref();

            root.draw(self);
        }

        self.renderer.end();

        unsafe { RenderState_PopBlendMode() };

        self.renderer.draw();

        unsafe { Profiler_End() };
    }

    /// Begin a whole screen new layer on top of the current one.
    /// Position of the layer (top/left corner) will be [0, 0] and size will be a size of the screen set in [`HmGui::begin_gui`].
    /// All new elements will be added to this new layer.
    /// Each layer has its own separate layout system.
    pub fn beginLayer(&mut self) {
        let layer_index = self.layers.len();
        let layer = HmGuiLayer::new_fixed(self.layer_index, Vec2::ZERO, self.screen_size);

        self.layers.push(layer);
        self.layer_index = layer_index;
    }

    /// Begin a new layer on top of the current one at specified position.
    /// The size of new layer will bw up to the screen borders.
    /// All new elements will be added to this new layer.
    /// Each layer has its own separate layout system.
    pub fn beginLayerAtPos(&mut self, pos: Vec2) {
        let layer_index = self.layers.len();
        // TODO: process situation when position is outside of the screen.
        let layer = HmGuiLayer::new_fixed(self.layer_index, pos, self.screen_size - pos);

        self.layers.push(layer);
        self.layer_index = layer_index;
    }

    /// Begin a new layer below the latest element of the current layer.
    /// Position and size of the new layer will be calculated after layouting of the previous layer.
    /// All new elements will be added to this new layer.
    /// Each layer has its own separate layout system.
    pub fn beginLayerBelow(&mut self) {
        let hash = self.last().as_ref().hash;

        let layer_index = self.layers.len();
        let layer = HmGuiLayer::new_below(self.layer_index, hash);

        self.layers.push(layer);
        self.layer_index = layer_index;
    }

    /// Close current layer and return to the previous one.
    pub fn endLayer(&mut self) {
        assert_ne!(
            self.layer_index, UNDEFINED_LAYER_INDEX,
            "Unmatched endLayer scope"
        );

        self.layer_index = self.layers[self.layer_index].prev_index;
    }

    /// Start a new container with a specified layout.
    fn begin_container(&mut self, layout: LayoutType) {
        let container = HmGuiContainer {
            layout,
            ..Default::default()
        };

        let widget_rf = self.init_widget(WidgetItem::Container(container));

        self.set_container(widget_rf.clone());
    }

    /// Starts stack container.
    /// Equivalent to: Gui:beginContainer(GuiLayoutType.Stack)
    pub fn begin_stack_container(&mut self) {
        self.begin_container(LayoutType::Stack);
    }

    /// Starts horizontal container.
    /// Equivalent to: Gui:beginContainer(GuiLayoutType.Horizontal)
    pub fn begin_horizontal_container(&mut self) {
        self.begin_container(LayoutType::Horizontal);
    }

    /// Starts vertical container.
    /// Equivalent to: Gui:beginContainer(GuiLayoutType.Vertical)
    pub fn begin_vertical_container(&mut self) {
        self.begin_container(LayoutType::Vertical);
    }

    /// Closes container started with one of `Gui:beginContainer()` calls.
    pub fn end_container(&mut self) {
        self.set_last(self.container());

        // We always have a parent since since we don't call end_container for root
        let Some(parent) = self.container().as_ref().parent.clone() else {
            unreachable!()
        };

        self.set_container(parent);
    }

    /// Update current container offset.
    /// Return offset value.
    pub fn update_container_offset(&mut self, offset: Vec2) -> Vec2 {
        let widget_rf = self.container();
        let mut widget = widget_rf.as_mut();
        let data = self.get_data(widget.hash);

        data.offset = data.offset.clamp(Vec2::ZERO, offset);

        let container = widget.get_container_item_mut();
        container.offset = -data.offset;

        data.offset
    }

    /// Return last element size calculated in the previous frame.
    pub fn element_size(&mut self) -> Vec2 {
        let widget_rf = self.last();
        let widget = widget_rf.as_mut();
        let data = self.get_data(widget.hash);

        data.size
    }

    /// Return current container element size calculated in the previous frame.
    pub fn container_size(&mut self) -> Vec2 {
        let widget_rf = self.container();
        let widget = widget_rf.as_mut();
        let data = self.get_data(widget.hash);

        data.size
    }

    /// Return current container element size calculated in the previous frame.
    pub fn container_min_size(&mut self) -> Vec2 {
        let widget_rf = self.container();
        let widget = widget_rf.as_mut();
        let data = self.get_data(widget.hash);

        data.min_size
    }

    /// Update current element minimum size.
    pub fn update_element_offset(&mut self, offset: Vec2) {
        let last = self.last();
        let widget_rf = last.clone();
        let widget = widget_rf.as_mut();
        let data = self.get_data(widget.hash);

        data.offset -= offset;
    }

    pub fn image(&mut self, image: &mut Tex2D) {
        let image_item = HmGuiImage { image };

        let _ = self.init_widget(WidgetItem::Image(image_item));
    }

    pub fn rect(&mut self) {
        let _ = self.init_widget(WidgetItem::Rect);
    }

    pub fn text(&mut self, text: &str, font: &Font, color: &Color) {
        let lines: Vec<_> = text.lines().collect();

        if lines.len() > 1 {
            // TODO: this is a temporary solution for multiline text.
            // Problem with it is that all widget styling will be applied to the container instead of text.
            self.begin_vertical_container();
            self.set_spacing(5.0);

            for line in lines {
                let item = HmGuiText {
                    text: line.into(),
                    font: font.clone().into(),
                    color: color.clone(),
                };
                let size = item.font.get_size2(line);
                let widget_rf = self.init_widget(WidgetItem::Text(item));
                let mut widget = widget_rf.as_mut();

                widget.inner_min_size = Vec2::new(size.x as f32, size.y as f32);
            }

            self.end_container();
        } else {
            let item = HmGuiText {
                text: text.into(),
                font: font.clone().into(),
                color: color.clone(),
            };
            let size = item.font.get_size2(text);
            let widget_rf = self.init_widget(WidgetItem::Text(item));
            let mut widget = widget_rf.as_mut();

            widget.inner_min_size = Vec2::new(size.x as f32, size.y as f32);
        }
    }

    /// Add multiline styled text element.
    pub fn text_view(&mut self, text_data: &TextData) {
        let image_item = HmGuiImage {
            image: std::ptr::null_mut(),
        };

        let widget_rf = self.init_widget(WidgetItem::Image(image_item));
        let widget = widget_rf.as_mut();

        let data = self.get_data(widget.hash);

        if let Some(text_view) = &mut data.text_view {
            text_view.set_data(text_data);
        } else {
            data.text_view = Some(TextView::new(text_data));
        }
    }

    /// Makes current widget `focusable` and returns true if mouse is over it.
    /// Returns true if mouse is over the widget (was calculated in the previous frame).
    pub fn is_mouse_over(&self, ty: FocusType) -> bool {
        let last = self.last();
        let mut widget = last.as_mut();

        // Will be used in the check_mouse_over to set `mouse over` hash for current widget for the next frame.
        widget.mouse_over[ty as usize] = true;

        self.mouse_over_widget_hash[ty as usize] == widget.hash
    }

    pub fn set_min_width(&self, width: f32) {
        let last = self.last();
        let mut widget = last.as_mut();

        widget.inner_min_size.x = width;
    }

    pub fn set_min_height(&self, height: f32) {
        let last = self.last();
        let mut widget = last.as_mut();

        widget.inner_min_size.y = height;
    }

    pub fn set_min_size(&self, width: f32, height: f32) {
        let last = self.last();
        let mut widget = last.as_mut();

        widget.inner_min_size.x = width;
        widget.inner_min_size.y = height;
    }

    pub fn set_fixed_width(&self, width: f32) {
        let last = self.last();
        let mut widget = last.as_mut();

        widget.default_size[0] = Length::Fixed(width);
    }

    pub fn set_fixed_height(&self, height: f32) {
        let last = self.last();
        let mut widget = last.as_mut();

        widget.default_size[1] = Length::Fixed(height);
    }

    pub fn set_fixed_size(&self, width: f32, height: f32) {
        let last = self.last();
        let mut widget = last.as_mut();

        widget.default_size[0] = Length::Fixed(width);
        widget.default_size[1] = Length::Fixed(height);
    }

    pub fn set_percent_width(&self, width: f32) {
        let last = self.last();
        let mut widget = last.as_mut();

        widget.default_size[0] = Length::Percent(width);
    }

    pub fn set_percent_height(&self, height: f32) {
        let last = self.last();
        let mut widget = last.as_mut();

        widget.default_size[1] = Length::Percent(height);
    }

    pub fn set_percent_size(&self, width: f32, height: f32) {
        let last = self.last();
        let mut widget = last.as_mut();

        widget.default_size[0] = Length::Percent(width);
        widget.default_size[1] = Length::Percent(height);
    }

    pub fn set_margin(&self, px: f32, py: f32) {
        let last = self.last();
        let mut widget = last.as_mut();

        widget.margin_lower = Vec2::new(px, py);
        widget.margin_upper = Vec2::new(px, py);
    }

    pub fn set_margin_ex(&self, left: f32, top: f32, right: f32, bottom: f32) {
        let last = self.last();
        let mut widget = last.as_mut();

        widget.margin_lower = Vec2::new(left, top);
        widget.margin_upper = Vec2::new(right, bottom);
    }

    pub fn set_margin_left(&self, margin: f32) {
        let last = self.last();
        let mut widget = last.as_mut();

        widget.margin_lower.x = margin;
    }

    pub fn set_margin_top(&self, margin: f32) {
        let last = self.last();
        let mut widget = last.as_mut();

        widget.margin_lower.y = margin;
    }

    pub fn set_margin_right(&self, margin: f32) {
        let last = self.last();
        let mut widget = last.as_mut();

        widget.margin_upper.x = margin;
    }

    pub fn set_margin_bottom(&self, margin: f32) {
        let last = self.last();
        let mut widget = last.as_mut();

        widget.margin_upper.y = margin;
    }

    pub fn set_border_width(&self, width: f32) {
        let last = self.last();
        let mut widget = last.as_mut();

        widget.border_width = width;
    }

    pub fn set_alignment(&self, h: AlignHorizontal, v: AlignVertical) {
        let last = self.last();
        let mut widget = last.as_mut();

        widget.alignment = [h.into(), v.into()];
    }

    pub fn set_horizontal_alignment(&self, align: AlignHorizontal) {
        let last = self.last();
        let mut widget = last.as_mut();

        widget.alignment[0] = align.into();
    }

    pub fn set_vertical_alignment(&self, align: AlignVertical) {
        let last = self.last();
        let mut widget = last.as_mut();

        widget.alignment[1] = align.into();
    }

    pub fn set_border_color(&self, color: &Color) {
        let last = self.last();
        let mut widget = last.as_mut();

        widget.border_color = *color;
    }

    pub fn set_background_color(&self, color: &Color) {
        let last = self.last();
        let mut widget = last.as_mut();

        widget.background_color = *color;
    }

    pub fn set_opacity(&self, opacity: f32) {
        let last = self.last();
        let mut widget = last.as_mut();

        widget.opacity = opacity;
    }

    // Container --------------------------------------------------------------

    pub fn set_clipping(&self, clip: bool) {
        let container = self.container();
        let mut widget = container.as_mut();
        let container = widget.get_container_item_mut();

        container.clip = clip;
    }

    pub fn set_padding(&self, px: f32, py: f32) {
        let container = self.container();
        let mut widget = container.as_mut();
        let container = widget.get_container_item_mut();

        container.padding_lower = Vec2::new(px, py);
        container.padding_upper = Vec2::new(px, py);
    }

    pub fn set_padding_ex(&self, left: f32, top: f32, right: f32, bottom: f32) {
        let container = self.container();
        let mut widget = container.as_mut();
        let container = widget.get_container_item_mut();

        container.padding_lower = Vec2::new(left, top);
        container.padding_upper = Vec2::new(right, bottom);
    }

    pub fn set_padding_left(&self, padding: f32) {
        let container = self.container();
        let mut widget = container.as_mut();
        let container = widget.get_container_item_mut();

        container.padding_lower.x = padding;
    }

    pub fn set_padding_top(&self, padding: f32) {
        let container = self.container();
        let mut widget = container.as_mut();
        let container = widget.get_container_item_mut();

        container.padding_lower.y = padding;
    }

    pub fn set_padding_right(&self, padding: f32) {
        let container = self.container();
        let mut widget = container.as_mut();
        let container = widget.get_container_item_mut();

        container.padding_upper.x = padding;
    }

    pub fn set_padding_bottom(&self, padding: f32) {
        let container = self.container();
        let mut widget = container.as_mut();
        let container = widget.get_container_item_mut();

        container.padding_upper.y = padding;
    }

    pub fn set_spacing(&self, spacing: f32) {
        let container = self.container();
        let mut widget = container.as_mut();
        let container = widget.get_container_item_mut();

        container.spacing = spacing;
    }

    pub fn set_children_alignment(&self, h: AlignHorizontal, v: AlignVertical) {
        let container = self.container();
        let mut widget = container.as_mut();
        let container = widget.get_container_item_mut();

        container.children_alignment = [h.into(), v.into()];
    }

    pub fn set_children_horizontal_alignment(&self, align: AlignHorizontal) {
        let container = self.container();
        let mut widget = container.as_mut();
        let container = widget.get_container_item_mut();

        container.children_alignment[0] = align.into();
    }

    pub fn set_children_vertical_alignment(&self, align: AlignVertical) {
        let container = self.container();
        let mut widget = container.as_mut();
        let container = widget.get_container_item_mut();

        container.children_alignment[1] = align.into();
    }

    /// Prints widgets hierarchy to the console. For testing.
    pub fn dump_widgets(&self) {
        let root = self.root();
        let container = root.as_ref();

        container.dump("GUI widgets", 1);
    }
}
