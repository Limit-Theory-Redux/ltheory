use std::collections::HashMap;

use arboard::Clipboard;
use glam::*;
use tracing::warn;

use super::*;
use crate::input::*;
use crate::render::*;
use crate::rf::Rf;
use crate::system::*;

pub struct HmGui {
    pub(super) renderer: UIRenderer,

    screen_size: Vec2,
    scale_factor: f64,

    layers: Vec<HmGuiLayer>,
    /// Current layer index in layers vector
    layer_index: usize,

    data: HashMap<u64, HmGuiData>,
    mouse_over_widget_hash: [u64; 2],
    focus_pos: Vec2,
    active_widget: Option<u64>,

    /// Named stroke looks (`define_look`, `set_stroke_look`); they outlive
    /// frames.
    pub(super) looks: HmGuiLooks,

    /// Fonts at the sizes scaled texts are drawn at (`set_scale`), by name
    /// and pixels per em; they outlive frames.
    scaled_fonts: HashMap<(String, u32), Font>,

    /// `None` when no clipboard backend is available, e.g. on a headless Linux CI runner
    /// without an X11/Wayland connection.
    clipboard: Option<Clipboard>,
}

impl HmGui {
    pub fn new(r: &mut Renderer, scale_factor: f64) -> Self {
        Self {
            renderer: UIRenderer::new(r),
            screen_size: Default::default(),
            scale_factor,
            layers: vec![],
            layer_index: UNDEFINED_LAYER_INDEX,
            data: HashMap::with_capacity(128),
            mouse_over_widget_hash: [0; 2],
            focus_pos: Vec2::ZERO,
            active_widget: None,
            looks: HmGuiLooks::new(),
            scaled_fonts: HashMap::new(),
            clipboard: Clipboard::new()
                .inspect_err(|err| warn!("Cannot create clipboard. Error: {err}"))
                .ok(),
        }
    }

    pub fn clipboard_get_text(&mut self) -> String {
        self.clipboard
            .as_mut()
            .and_then(|clipboard| clipboard.get_text().ok())
            .unwrap_or_default()
    }

    pub fn clipboard_set_text(&mut self, text: String) {
        if let Some(clipboard) = self.clipboard.as_mut() {
            if let Err(err) = clipboard.set_text(text) {
                warn!("Cannot set clipboard text. Error: {err}");
            }
        }
    }

    pub fn screen_size(&self) -> Vec2 {
        self.screen_size
    }

    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    pub fn set_scale_factor(&mut self, scale_factor: f64) {
        self.scale_factor = scale_factor;
    }

    /// `font` at `scale` times its size, rounded to whole pixels per em:
    /// loaded the first time, then kept.
    pub(super) fn scaled_font(&mut self, r: &mut Renderer, font: &Font, scale: f32) -> Font {
        let size = (font.size() * scale).round().max(1.0) as u32;
        if size as f32 == font.size() {
            return font.clone();
        }

        self.scaled_fonts
            .entry((font.name(), size))
            .or_insert_with(|| Font::load(r, &font.name(), size))
            .clone()
    }

    /// The UI scale of a widget of the current layer: its own `set_scale`
    /// times its parents' and the layer's.
    fn scale_of(&self, widget_rf: &Rf<HmGuiWidget>) -> f32 {
        let mut scale = self.layers[self.layer_index].scale;
        let mut next = Some(widget_rf.clone());
        while let Some(widget_rf) = next {
            let widget = widget_rf.as_ref();
            scale *= widget.own_scale;
            next = widget.parent.clone();
        }
        scale
    }

    /// The base layer's root widget. Usable both mid-construction (where it
    /// matches the current layer while no nested layer has been begun) and
    /// after `end_gui` (where `layer_index` has been reset to
    /// [`UNDEFINED_LAYER_INDEX`] but the base layer at index 0 still holds
    /// the fully laid-out tree).
    #[inline]
    pub fn root(&self) -> Rf<HmGuiWidget> {
        let index = if self.layer_index == UNDEFINED_LAYER_INDEX {
            0
        } else {
            self.layer_index
        };
        self.layers[index].root.clone()
    }

    /// The root widget of a layer (0 = the base layer), for the layout tests
    /// (which only run on Linux, see `mod tests`).
    #[cfg(all(test, target_os = "linux"))]
    pub(super) fn layer_root(&self, index: usize) -> Rf<HmGuiWidget> {
        self.layers[index].root.clone()
    }

    #[inline]
    pub fn container(&self) -> Rf<HmGuiWidget> {
        self.layers[self.layer_index].container.clone()
    }

    /// The UI scale of the current container (1 before the first layer).
    fn current_scale(&self) -> f32 {
        if self.layer_index == UNDEFINED_LAYER_INDEX {
            1.0
        } else {
            self.scale_of(&self.container())
        }
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

        widget.hash = Hash::fnv64_incremental(
            parent_hash,
            parent_container.children_hash.to_le_bytes().as_slice(),
        );

        let widget_rf = Rf::new(widget);

        parent_container.children.push(widget_rf.clone());

        self.set_last(widget_rf.clone());

        widget_rf.clone()
    }

    /// Add a shape widget (`annulus`, `circle`, `line`, `point`, `ring`,
    /// `triangle`, `wedge`, and the strokes).
    fn shape(&mut self, kind: HmGuiShapeKind, color: &Color, glow: bool) {
        let item = HmGuiShape::new(kind, *color, glow);
        let _ = self.init_widget(WidgetItem::Shape(item));
    }

    /// Change the last widget if it is a stroke (`what` names the caller in
    /// the warning when it is not).
    fn with_last_stroke(&self, what: &str, f: impl FnOnce(&mut HmGuiShape)) {
        let last = self.last();
        let mut widget = last.as_mut();

        match &mut widget.item {
            WidgetItem::Shape(shape) if shape.kind.is_stroke() => f(shape),
            _ => warn!("{what}: the last widget is not a stroke"),
        }
    }

    /// Change the last widget if it is an image (`what` names the caller in
    /// the warning when it is not).
    fn with_last_image(&self, what: &str, f: impl FnOnce(&mut HmGuiImage)) {
        let last = self.last();
        let mut widget = last.as_mut();

        match &mut widget.item {
            WidgetItem::Image(image) => f(image),
            _ => warn!("{what}: the last widget is not an image"),
        }
    }

    /// Change the last widget if it is a text (`what` names the caller in the
    /// warning when it is not).
    fn with_last_text(&self, what: &str, f: impl FnOnce(&mut HmGuiText)) {
        let last = self.last();
        let mut widget = last.as_mut();

        match &mut widget.item {
            WidgetItem::Text(text) => f(text),
            _ => warn!("{what}: the last widget is not a text"),
        }
    }

    /// Get persistent readonly data of the widget by its hash if exists.
    pub fn data(&self, widget_hash: u64) -> Option<&HmGuiData> {
        self.data.get(&widget_hash)
    }

    /// Get persistent mutable data of the widget by its hash.
    /// Add new data entry if it doesn't exist.
    pub fn data_mut(&mut self, widget_hash: u64) -> &mut HmGuiData {
        self.data.entry(widget_hash).or_default()
    }

    /// Calculate if mouse is over the widget. Recursively iterate over container widgets.
    /// Setting mouse over hash at the end of the method guarantees that the last (top most) widget will get the mouse over flag set.
    // TODO: take in account container clipping
    fn check_mouse_over(&mut self, widget_rf: Rf<HmGuiWidget>) {
        let widget = widget_rf.as_ref();
        let is_mouse_over = widget.contains_point(self.focus_pos);

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

    pub fn in_focus(&self, widget: &HmGuiWidget) -> bool {
        if let Some(hash) = self.active_widget {
            widget.hash == hash
        } else {
            false
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

        self.begin_layer();
    }

    /// Finish GUI declaration, calculate hierarchy widgets sizes and layout.
    // TODO: do not calculate layout for the widgets that go out of the screen. If possible.
    pub fn end_gui(&mut self, r: &mut Renderer, input: &Input) {
        Profiler::begin("HmGui_End");

        self.end_layer();
        assert_eq!(
            self.layer_index, UNDEFINED_LAYER_INDEX,
            "At least one beginLayer scope was not closed"
        );

        let layers_root: Vec<_> = self
            .layers
            .iter()
            .map(|layer| (layer.root.clone(), layer.location.clone(), layer.scale))
            .collect();
        for (root_rf, location, scale) in &layers_root {
            let mut root = root_rf.as_mut();

            if let HmGuiLayerLocation::Below(widget_rf) = location {
                // The widget is on an earlier layer, so it's laid out already (this
                // frame). The layer starts under its border box: margins don't count.
                let widget = widget_rf.as_ref();
                let box_pos = widget.pos + widget.margin_lower;
                let box_size = widget.size - widget.margin_lower - widget.margin_upper;

                root.pos = Vec2::new(box_pos.x, box_pos.y + box_size.y);
                root.inner_pos = root.pos;
                root.size = self.screen_size - root.pos; // rest of the screen
                root.inner_size = root.size;
            }

            root.apply_scale(self, r, *scale);
            root.compute_size(self, r);
            root.layout(self, r, input);
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

        Profiler::end();
    }

    /// Pass information about widgets to the renderer and draw them.
    // TODO: optimize - do not pass to the renderer widgets that are outside of the rendering region
    pub fn draw(&mut self, r: &mut Renderer) {
        Profiler::begin("HmGui_Draw");

        self.renderer.begin(r);

        // In order: a later layer (a dropdown list, a tooltip) is drawn over the
        // earlier ones, as the mouse finds it first (`end_gui`).
        let layers_root: Vec<_> = self.layers.iter().map(|layer| layer.root.clone()).collect();
        for root_rf in layers_root {
            let root = root_rf.as_ref();

            root.draw(self);
        }

        self.renderer.end();

        self.renderer.draw(r);

        Profiler::end();
    }

    /// Begin a whole screen new layer on top of the current one.
    /// Position of the layer (top/left corner) will be [0, 0] and size will be a size of the screen set in [`HmGui::begin_gui`].
    /// All new elements will be added to this new layer.
    /// Each layer has its own separate layout system.
    pub fn begin_layer(&mut self) {
        let layer_index = self.layers.len();
        let scale = self.current_scale();
        let layer = HmGuiLayer::new_fixed(self.layer_index, Vec2::ZERO, self.screen_size, scale);

        self.layers.push(layer);
        self.layer_index = layer_index;
    }

    /// Begin a new layer on top of the current one at specified position.
    /// The size of new layer will bw up to the screen borders.
    /// All new elements will be added to this new layer.
    /// Each layer has its own separate layout system.
    pub fn begin_layer_at_pos(&mut self, pos: Vec2) {
        let layer_index = self.layers.len();
        // TODO: process situation when position is outside of the screen.
        let scale = self.current_scale();
        let layer = HmGuiLayer::new_fixed(self.layer_index, pos, self.screen_size - pos, scale);

        self.layers.push(layer);
        self.layer_index = layer_index;
    }

    /// Begin a new layer below the latest element of the current layer.
    /// Position and size of the new layer will be calculated after layouting of the previous layer.
    /// All new elements will be added to this new layer.
    /// Each layer has its own separate layout system.
    pub fn begin_layer_below(&mut self) {
        let widget = self.last();

        let layer_index = self.layers.len();
        let scale = self.scale_of(&widget);
        let layer = HmGuiLayer::new_below(self.layer_index, widget, scale);

        self.layers.push(layer);
        self.layer_index = layer_index;
    }

    /// Close current layer and return to the previous one.
    pub fn end_layer(&mut self) {
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
        let data = self.data_mut(widget.hash);

        data.offset = data.offset.clamp(Vec2::ZERO, offset);

        let container = widget.get_container_item_mut();
        container.offset = -data.offset;

        data.offset
    }

    /// Return last element size calculated in the previous frame.
    pub fn element_size(&mut self) -> Vec2 {
        let widget_rf = self.last();
        let widget = widget_rf.as_mut();
        let data = self.data_mut(widget.hash);

        data.size
    }

    /// Return current container element size calculated in the previous frame.
    pub fn container_size(&mut self) -> Vec2 {
        let widget_rf = self.container();
        let widget = widget_rf.as_mut();
        let data = self.data_mut(widget.hash);

        data.size
    }

    /// Return current container element size calculated in the previous frame.
    pub fn container_min_size(&mut self) -> Vec2 {
        let widget_rf = self.container();
        let widget = widget_rf.as_mut();
        let data = self.data_mut(widget.hash);

        data.min_size
    }

    /// Return current container position (top-left) calculated in the previous frame.
    pub fn container_pos(&mut self) -> Vec2 {
        let widget_rf = self.container();
        let widget = widget_rf.as_mut();
        let data = self.data_mut(widget.hash);

        data.pos
    }

    /// The UI scale of the current container (`set_scale`: its own times the
    /// scales of the containers around it), screen pixels per design pixel.
    /// Known while the GUI is built, unlike the sizes above. Divide what HmGui
    /// reports in screen pixels (`container_size`, `element_size`, the mouse)
    /// by it to size a widget inside the container from it.
    pub fn container_scale(&self) -> f32 {
        self.current_scale()
    }

    /// Update current element minimum size.
    pub fn update_element_offset(&mut self, offset: Vec2) {
        let last = self.last();
        let widget_rf = last.clone();
        let widget = widget_rf.as_mut();
        let data = self.data_mut(widget.hash);

        data.offset -= offset;
    }

    /// Add an image, stretched to fill the widget.
    pub fn image(&mut self, image: &mut Tex2D) {
        let image_item = HmGuiImage::new(image.clone(), HmGuiImageLayout::Fit);

        let _ = self.init_widget(WidgetItem::Image(image_item));
    }

    /// Multiply the last image's texels by `color`: a white image takes the
    /// colour as its own, and the colour's alpha fades it. White (the
    /// default) draws the image as it is.
    pub fn set_image_color(&self, color: &Color) {
        self.with_last_image("setImageColor", |image| image.color = *color);
    }

    /// Sample the last image bilinearly between mip levels instead of
    /// taking the nearest texel, for an image drawn smaller or larger than
    /// its texture (an icon). Smaller only looks smooth when the texture
    /// has mip levels (`Tex2D:genMipmap` on a texture created with them).
    pub fn set_image_smooth(&self, smooth: bool) {
        self.with_last_image("setImageSmooth", |image| image.smooth = smooth);
    }

    pub fn rect(&mut self) {
        let _ = self.init_widget(WidgetItem::Rect);
    }

    /// Add a ring band (annulus), centred in the widget: its outer diameter
    /// is the widget's smaller side and it is `thickness` pixels wide,
    /// inwards. Nothing is drawn at thickness 0.
    ///
    /// Shapes draw over rects and under texts. `glow` draws additive
    /// (brightens only, with a soft halo) instead of alpha blended; any glow
    /// reaches past the widget's edge.
    pub fn annulus(&mut self, color: &Color, thickness: f32, glow: bool) {
        self.shape(HmGuiShapeKind::Annulus { thickness }, color, glow);
    }

    /// Add a filled circle with a soft edge, centred in the widget, as wide
    /// as the widget's smaller side. `glow`: see `annulus`.
    pub fn circle(&mut self, color: &Color, glow: bool) {
        self.shape(HmGuiShapeKind::Circle, color, glow);
    }

    /// Add a point of light: a small bright core in the widget's centre
    /// whose soft glow fills the widget. `glow`: see `annulus`.
    pub fn point(&mut self, color: &Color, glow: bool) {
        self.shape(HmGuiShapeKind::Point, color, glow);
    }

    /// Add a thin circle line, centred in the widget, as wide as the
    /// widget's smaller side. `glow`: see `annulus`.
    pub fn ring(&mut self, color: &Color, glow: bool) {
        self.shape(HmGuiShapeKind::Ring, color, glow);
    }

    /// Add a piece of a ring band (wedge), centred in the widget: its outer
    /// diameter is the widget's smaller side, it is `thickness` pixels deep,
    /// inwards, and it runs from `start` to `start + sweep` degrees, where 0
    /// is up and angles grow clockwise (like CSS `conic-gradient`). Nothing
    /// is drawn at thickness or sweep 0. `glow`: see `annulus`.
    ///
    /// Wedges of one widget size stacked in one container share a centre,
    /// so each piece of an arc can be its own widget.
    pub fn wedge(&mut self, color: &Color, thickness: f32, start: f32, sweep: f32, glow: bool) {
        self.shape(
            HmGuiShapeKind::Wedge {
                thickness,
                start,
                sweep,
            },
            color,
            glow,
        );
    }

    /// Add a filled triangle with soft edges. Its corners are fractions of
    /// the widget: (0, 0) is the top left, (1, 1) the bottom right (like CSS
    /// `clip-path: polygon()`). A point at the left middle:
    /// `triangle(c, 1, 0, 0, 0.5, 1, 1, false)`. `glow`: see `annulus`.
    #[allow(clippy::too_many_arguments)]
    pub fn triangle(
        &mut self,
        color: &Color,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        glow: bool,
    ) {
        let points = [Vec2::new(x1, y1), Vec2::new(x2, y2), Vec2::new(x3, y3)];
        self.shape(HmGuiShapeKind::Triangle { points }, color, glow);
    }

    /// Add a thin line from (`x1`, `y1`) to (`x2`, `y2`), fractions of the
    /// widget as in `triangle`. Across the middle: `line(c, 0, 0.5, 1, 0.5,
    /// false)`. `glow`: see `annulus`.
    pub fn line(&mut self, color: &Color, x1: f32, y1: f32, x2: f32, y2: f32, glow: bool) {
        let (from, to) = (Vec2::new(x1, y1), Vec2::new(x2, y2));
        self.shape(HmGuiShapeKind::Line { from, to }, color, glow);
    }

    /// Fade the last shape (annulus, circle, point, ring or wedge) by
    /// direction: fully drawn within `visible` degrees of `centre`, gone
    /// beyond `gone` degrees, smooth in between (0 = up, clockwise, measured
    /// from the shape's centre). Shapes that share a centre and a fade fade
    /// as one, like the pieces of a curved tape.
    pub fn set_shape_fade(&self, centre: f32, visible: f32, gone: f32) {
        let last = self.last();
        let mut widget = last.as_mut();

        match &mut widget.item {
            WidgetItem::Shape(shape) if shape.kind.is_round() => {
                shape.fade = Some(HmGuiShapeFade {
                    centre,
                    visible,
                    gone,
                });
            }
            _ => warn!("setShapeFade: the last widget is not a round shape"),
        }
    }

    /// Add a bendable stroke from (`x1`, `y1`) to (`x2`, `y2`), fractions of
    /// the widget as in `triangle`, bent `bend` degrees on the way (positive
    /// curves clockwise; below a full turn, since at 360 degrees the ends
    /// meet and the circle's size is unknown), `thickness` pixels wide.
    /// A bend of 0 is a straight line.
    ///
    /// Strokes draw with the plain `stroke` look (a line in its colour)
    /// until `setStrokeLook` names another; `setStrokeDash`,
    /// `setStrokeRoundCaps` and `setStrokeFade` change the last one. Like
    /// the other shapes they draw over rects and under texts, and glow
    /// reaches past the widget's edge.
    #[allow(clippy::too_many_arguments)]
    pub fn stroke_between(
        &mut self,
        color: &Color,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        bend: f32,
        thickness: f32,
    ) {
        let form = HmGuiStrokeForm::Between {
            from: Vec2::new(x1, y1),
            to: Vec2::new(x2, y2),
            bend,
            thickness,
        };
        self.shape(HmGuiShapeKind::Stroke(form), color, false);
    }

    /// Add a bendable stroke that starts at (`x`, `y`), a fraction of the
    /// widget, leaves in `direction` degrees (0 = up, clockwise) and is
    /// `length` pixels long, turning `bend` degrees in all (positive =
    /// clockwise): bending curls it up with its length kept, and 360 closes
    /// it into a circle. `thickness` and the rest: see `strokeBetween`.
    #[allow(clippy::too_many_arguments)]
    pub fn stroke_bent(
        &mut self,
        color: &Color,
        x: f32,
        y: f32,
        direction: f32,
        length: f32,
        bend: f32,
        thickness: f32,
    ) {
        let form = HmGuiStrokeForm::Bent {
            start: Vec2::new(x, y),
            direction,
            length,
            bend,
            thickness,
        };
        self.shape(HmGuiShapeKind::Stroke(form), color, false);
    }

    /// Add a stroke around the widget's centre, its middle line on the circle
    /// as wide as the widget's smaller side, from `start` to `start + sweep`
    /// degrees (0 = up, clockwise; negative sweeps go counter-clockwise; 360
    /// is a full circle), `thickness` pixels wide. Arcs of one widget size
    /// stacked in one container share their middle line whatever their
    /// thickness. It is a round shape, so `setShapeFade` works on it too.
    /// The rest: see `strokeBetween`.
    pub fn arc(&mut self, color: &Color, thickness: f32, start: f32, sweep: f32) {
        let form = HmGuiStrokeForm::Arc {
            start,
            sweep,
            thickness,
        };
        self.shape(HmGuiShapeKind::Stroke(form), color, false);
    }

    /// Dash the last stroke (or annulus, ring, wedge, line): `dash` pixels
    /// drawn, then `gap` pixels left out, the pattern shifted back by `phase`
    /// pixels (like SVG's `stroke-dashoffset`; animate it to make the dashes
    /// run). Lengths are along the stroke's middle; round caps add half the
    /// thickness to both ends of each dash, so `dash` 0 with round caps
    /// makes dots. A `gap` of 0 is solid again.
    pub fn set_stroke_dash(&self, dash: f32, gap: f32, phase: f32) {
        self.with_last_stroke("setStrokeDash", |shape| {
            shape.stroke.dash = StrokeDash::new(dash, gap, phase);
        });
    }

    /// Round ends (half circles, also on every dash) on the last stroke
    /// instead of flat ones. Lines have round ends to begin with.
    pub fn set_stroke_round_caps(&self, round: bool) {
        self.with_last_stroke("setStrokeRoundCaps", |shape| {
            shape.stroke.round_caps = Some(round);
        });
    }

    /// Round the corners of the last stroke's flat ends (and of its dashes)
    /// by `radius` pixels, at most half its thickness. Wedges have 2 to
    /// begin with; 0 makes them sharp.
    pub fn set_stroke_corner_radius(&self, radius: f32) {
        self.with_last_stroke("setStrokeCornerRadius", |shape| {
            shape.stroke.corner = Some(radius.max(0.0));
        });
    }

    /// Fade the last stroke along its length: fully drawn within `visible`
    /// pixels of the point `centre` pixels from its start, gone beyond
    /// `gone`, smooth in between. Replaces a `setShapeFade` window.
    pub fn set_stroke_fade(&self, centre: f32, visible: f32, gone: f32) {
        self.with_last_stroke("setStrokeFade", |shape| {
            shape.stroke.fade = Some(StrokeFade::along(centre, visible, gone));
        });
    }

    /// Draw the last stroke with the look called `look` (see `defineLook`).
    /// A name not defined yet gets the default values until it is.
    pub fn set_stroke_look(&mut self, look: &str) {
        let (id, created) = self.looks.get_or_insert(look);
        if created {
            warn!("setStrokeLook: look '{look}' is not defined yet; it draws as a plain stroke");
        }
        self.with_last_stroke("setStrokeLook", |shape| shape.stroke.look = Some(id));
    }

    /// Set one look value for the last stroke only, on top of its look (for
    /// example a glow that pulses on one element). `param` is one of `core`,
    /// `coreWidth`, `glow`, `glowWidth`, `boost`, `shade`, `halo`, `haloWidth`
    /// (see `defineLook`).
    pub fn set_stroke_look_param(&self, param: &str, value: f32) {
        match look_param_slot(param) {
            Ok(slot) => self.with_last_stroke("setStrokeLookParam", |shape| {
                shape.stroke.overrides[slot] = Some(value);
            }),
            Err(e) => warn!("setStrokeLookParam: {e}"),
        }
    }

    /// (Re)define the look called `look` with the default values: set what
    /// differs with `setLookParam` (`UICore.Look.define` does both). Looks
    /// outlive frames, so define them once; every stroke that uses one
    /// follows when it changes. The built-in looks `annulus`, `ring`,
    /// `wedge`, `line` and their `*Glow` twins are what those calls draw
    /// with, and redefining one restyles them all.
    ///
    /// The values (defaults in brackets): `core` [1] and `coreWidth` [0.5]:
    /// strength of the solid core and how soft its edge is, in pixels;
    /// `glow` [0] and `glowWidth` [4]: strength and reach of the glow around
    /// it; `boost` [1]: multiplies the colour of the light; `shade` [1]: how
    /// much it covers what is behind it, 0 = only adds light (neon), 1 =
    /// normal alpha; `halo` [0] and `haloWidth` [8]: a dark halo darkening
    /// what is behind out to that many pixels, 1 = black.
    pub fn define_look(&mut self, look: &str) {
        let _ = self.looks.define(look);
    }

    /// Set one value of the look called `look` (created with the default
    /// values if new). The values: see `defineLook`.
    pub fn set_look_param(&mut self, look: &str, param: &str, value: f32) {
        if let Err(e) = self.looks.set_param(look, param, value) {
            warn!("setLookParam: {e}");
        }
    }

    /// Add a line of text. A text widget narrower than its text (a fixed or
    /// stretched width, or a parent that is too small) shows as much of the
    /// text as fits, ended with "…".
    pub fn text(&mut self, text: &str, font: &Font, color: &Color) {
        let item = HmGuiText::new(text, font, color);
        let widget_rf = self.init_widget(WidgetItem::Text(item));
        let _ = widget_rf.as_mut();
    }

    /// Add a text element like `text`, with a 1 px `outline` and a 3 px
    /// `halo` around the glyphs so it stays readable on any background (see
    /// `Font::draw_outlined`). An alpha of 0 leaves a part out: a transparent
    /// outline gives the halo alone. Takes the same room as `text`: outline
    /// and halo spill past the widget, so a clipping container cuts them off
    /// at its edge.
    pub fn text_outlined(
        &mut self,
        text: &str,
        font: &Font,
        color: &Color,
        outline: &Color,
        halo: &Color,
    ) {
        let mut item = HmGuiText::new(text, font, color);
        item.outline = Some(TextOutline {
            outline: *outline,
            halo: *halo,
        });
        let widget_rf = self.init_widget(WidgetItem::Text(item));
        let _ = widget_rf.as_mut();
    }

    /// Break the last text into lines where it is wider than its max width
    /// (`set_max_width`) or fixed width, between words, instead of cutting it
    /// with "…" (like CSS `white-space: normal`). A line break ("\n") in a
    /// text always starts a new line, wrapped or not. A percentage or
    /// stretched width doesn't wrap it: give it a max width.
    pub fn set_text_wrap(&self, wrap: bool) {
        self.with_last_text("setTextWrap", |text| text.wrap = wrap);
    }

    /// Where the lines of the last text sit in its widget: `Left`, `Right`, or
    /// centred (the default, as a single line is centred when its widget is
    /// wider than it), like CSS `text-align`.
    pub fn set_text_align(&self, align: AlignHorizontal) {
        self.with_last_text("setTextAlign", |text| text.align = align.into());
    }

    /// Add multiline styled text element.
    pub fn text_view(&mut self, text_data: &mut TextData, editable: bool) {
        let widget_rf = self.init_widget(WidgetItem::TextView(None));
        let widget = widget_rf.as_mut();

        let data = self.data_mut(widget.hash);

        // changes from Lua (or new text data) since the last frame
        let changed = text_data.take_needs_sync();

        if let Some(text_view) = &mut data.text_view {
            text_view.set_editable(editable);

            // a read-only view always shows the Lua data; an editable one takes
            // it when Lua changed it (what the player types goes the other way,
            // `get_text_view_changes`)
            if changed || !editable {
                text_view.set_data(text_data);
            }
        } else {
            data.text_view = Some(TextView::new(text_data, editable));
        }
    }

    /// Apply changes to the text data if any and return if text was changed.
    pub fn get_text_view_changes(&mut self, text_data: &mut TextData) -> bool {
        let last = self.last();
        let widget = last.as_mut();
        let data = self.data_mut(widget.hash);
        let text_view = data
            .text_view
            .as_mut()
            .expect("Widget doesn't have a text view");

        text_view.is_editable() && text_view.update_source(text_data)
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

    /// Sets or removes current widget focus.
    /// To be used in combination with some input check, i.e. mouse left click inside or outside the widget.
    pub fn set_focus(&mut self, focused: bool) {
        let last = self.last();
        let widget = last.as_ref();

        if focused {
            self.active_widget = Some(widget.hash);
        } else if let Some(hash) = self.active_widget {
            if widget.hash == hash {
                self.active_widget = None;
            }
        }
    }

    /// Returns true if current widget is in focus.
    pub fn has_focus(&self) -> bool {
        if let Some(hash) = self.active_widget {
            let last = self.last();
            let widget = last.as_ref();

            widget.hash == hash
        } else {
            false
        }
    }

    /// Returns true if there is an editable text view in focus.
    pub fn has_active_input(&self) -> bool {
        if let Some(hash) = self.active_widget {
            if let Some(data) = self.data(hash) {
                if let Some(text_view) = &data.text_view {
                    return text_view.is_editable();
                }
            }
        }

        false
    }

    /// The widget is at least this wide (excluding margin and border); a
    /// container or text is wider when its content needs it.
    pub fn set_min_width(&self, width: f32) {
        let last = self.last();
        let mut widget = last.as_mut();

        widget.inner_min_size.x = width;
        widget.min_size_request.x = width;
    }

    /// The widget is at least this high (excluding margin and border); a
    /// container or text is higher when its content needs it.
    pub fn set_min_height(&self, height: f32) {
        let last = self.last();
        let mut widget = last.as_mut();

        widget.inner_min_size.y = height;
        widget.min_size_request.y = height;
    }

    /// The widget is at least this big (excluding margin and border); a
    /// container or text is bigger when its content needs it.
    pub fn set_min_size(&self, width: f32, height: f32) {
        let last = self.last();
        let mut widget = last.as_mut();

        widget.inner_min_size = Vec2::new(width, height);
        widget.min_size_request = Vec2::new(width, height);
    }

    /// The widget is at most this wide (excluding margin and border), also when
    /// stretched or given a fixed size; a text that is wider is cut and ends
    /// with "…" (like CSS `max-width` with `text-overflow: ellipsis`).
    pub fn set_max_width(&self, width: f32) {
        let last = self.last();
        let mut widget = last.as_mut();

        widget.max_size.x = width;
    }

    /// The widget is at most this high (excluding margin and border), also when
    /// stretched or given a fixed size.
    pub fn set_max_height(&self, height: f32) {
        let last = self.last();
        let mut widget = last.as_mut();

        widget.max_size.y = height;
    }

    /// The widget is at most this big (excluding margin and border); see
    /// `set_max_width`.
    pub fn set_max_size(&self, width: f32, height: f32) {
        let last = self.last();
        let mut widget = last.as_mut();

        widget.max_size = Vec2::new(width, height);
    }

    /// Scale the widget and everything in it, on top of the scale of the
    /// containers around it (like CSS `zoom`): every size, position, margin,
    /// padding, spacing, border width, font size and shape length set from Lua
    /// for it and inside it is in design pixels, `scale` screen pixels each.
    /// Percentages need no scaling, and what HmGui reports back
    /// (`container_pos`, `container_size`, `element_size`) and the mouse stay
    /// in screen pixels. Layers begun inside it (dropdown lists, tooltips)
    /// take its scale, so set it right after beginning the container, before
    /// its children. A UI built for 1080p, scaled by the window height / 1080,
    /// looks the same at every resolution.
    pub fn set_scale(&self, scale: f32) {
        if !(scale > 0.0 && scale.is_finite()) {
            warn!("setScale: the scale must be a positive number, not {scale}");
            return;
        }

        let last = self.last();
        let mut widget = last.as_mut();

        widget.own_scale = scale;
    }

    /// Place the widget at (x, y) relative to its parent container's top left
    /// corner inside the border (the parent's padding doesn't apply; its scroll
    /// offset does). The widget leaves its parent's layout, like CSS
    /// `position: absolute`: it doesn't move its siblings or make its parent
    /// bigger, and its alignment is not used. Its size is its fixed, percent or
    /// min size; its margins still apply.
    pub fn set_position(&self, x: f32, y: f32) {
        let last = self.last();
        let mut widget = last.as_mut();

        widget.position = Some(Vec2::new(x, y));
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
