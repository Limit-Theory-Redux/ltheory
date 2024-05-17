use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use glam::*;

use crate::common::*;
use crate::input::*;
use crate::math::*;
use crate::render::*;
use crate::rf::Rf;
use crate::system::*;

use super::*;

pub struct HmGui {
    pub(super) renderer: UIRenderer,

    /// Top level container object with None layout. Used for recalculating sizes, layouts and drawing of the whole gui
    root: Rf<HmGuiWidget>,
    /// Current active container
    container: Rf<HmGuiWidget>,
    /// Either last created/initialized widget (container, image, text, rect) or the last widget of the ended container
    last: Rf<HmGuiWidget>,

    data: HashMap<u64, HmGuiData>,
    mouse_over_widget_hash: [u64; 2],
    focus_pos: Vec2,
    activate: bool,

    default_property_registry: HmGuiPropertyRegistry,
    property_registry: HmGuiPropertyRegistry,
    theme_registry: HmGuiStyleRegistry,
    style_registry: HmGuiStyleRegistry,
    element_style: HmGuiStyle,
}

impl HmGui {
    pub fn new() -> Self {
        let container = HmGuiContainer {
            layout: LayoutType::Stack,
            clip: true, // always clip elements out of the screen
            spacing: 0.0,
            ..Default::default()
        };

        let mut widget = HmGuiWidget::new(None, WidgetItem::Container(container));
        widget.hash = Hash_FNV64_Init();

        let root = Rf::new(widget);
        let container = root.clone();
        let last = root.clone();

        let property_registry = HmGuiPropertyRegistry::new();
        let default_property_registry = property_registry.clone();

        let f = |_: &str, name: &str| {
            property_registry
                .registry
                .get_full(name)
                .map(|(id, _, prop)| (id.into(), prop.value.get_type()))
        };

        let theme_folders = Resource::get_folders(ResourceType::Theme);
        let mut theme_registry = HmGuiStyleRegistry::default();
        for folder_path in theme_folders {
            let registry = HmGuiStyleRegistry::load(&folder_path, f);
            if registry.size() > 0 {
                theme_registry = registry;
                break;
            }
        }

        let style_folders = Resource::get_folders(ResourceType::Other);
        let mut style_registry = HmGuiStyleRegistry::default();
        for folder_path in style_folders {
            let file_path = folder_path.join("styles.yaml");
            if file_path.is_file() {
                style_registry = HmGuiStyleRegistry::load_map(&file_path, f);
                break;
            }
        }

        Self {
            renderer: Default::default(),
            root,
            container,
            last,
            data: HashMap::with_capacity(128),
            mouse_over_widget_hash: [0; 2],
            focus_pos: Vec2::ZERO,
            activate: false,
            default_property_registry,
            property_registry,
            theme_registry,
            style_registry,
            element_style: Default::default(),
        }
    }

    pub fn root(&self) -> Rf<HmGuiWidget> {
        self.root.clone()
    }

    pub fn mouse_over_widget_hash(&self) -> u64 {
        self.mouse_over_widget_hash[FocusType::Mouse as usize]
    }

    fn apply_widget_properties(&self, widget: &mut HmGuiWidget) {
        let color = self
            .get_property_value(HmGuiProperties::BorderColor.id())
            .get_color();
        widget.set_border_color(&color);

        let color = self
            .get_property_value(HmGuiProperties::BackgroundColor.id())
            .get_color();
        widget.set_background_color(&color);

        let color = self
            .get_property_value(HmGuiProperties::HighlightColor.id())
            .get_color();
        widget.set_highlight_color(&color);

        let opacity = self
            .get_property_value(HmGuiProperties::Opacity.id())
            .get_f32();
        widget.set_opacity(opacity);
    }

    /// Add a new widget into the current container.
    fn init_widget(&mut self, item: WidgetItem) -> Rf<HmGuiWidget> {
        let parent_rf = self.container.clone();
        let mut parent = parent_rf.as_mut();
        let parent_hash = parent.hash;
        let parent_container = parent.get_container_item_mut();

        parent_container.children_hash = (parent_container.children_hash).wrapping_add(1);

        let mut widget = HmGuiWidget::new(Some(parent_rf.clone()), item);

        self.apply_widget_properties(&mut widget);

        widget.hash = unsafe {
            Hash_FNV64_Incremental(
                parent_hash,
                &mut parent_container.children_hash as *mut u32 as *const _,
                std::mem::size_of::<u32>() as i32,
            )
        };

        let widget_rf = Rf::new(widget);

        parent_container.children.push(widget_rf.clone());

        self.last = widget_rf.clone();

        widget_rf.clone()
    }

    /// Get persistent data of the widget by its hash.
    pub fn get_data(&mut self, widget_hash: u64) -> &mut HmGuiData {
        self.data.entry(widget_hash).or_insert(HmGuiData::default())
    }

    pub fn set_property<T: Into<HmGuiPropertyValue>>(&mut self, id: HmGuiProperties, value: T) {
        self.set_property_value(id.id(), &value.into());
    }

    /// Calculate if mouse is over the widget. Recursively iterate over container widgets.
    /// Setting mouse over hash at the end of the method guarantees that the last (top most) widget will get the mouse over flag set.
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

    /// Sets widget's `mouse over` flag to true.
    /// Will be used in the check_mouse_over to set `mouse over` hash for current widget for the next frame.
    /// Returns true if mouse is over the widget (was calculated in the previous frame).
    fn is_mouse_over_intern(&self, widget: &mut HmGuiWidget, ty: FocusType) -> bool {
        widget.mouse_over[ty as usize] = true;

        self.mouse_over_widget_hash[ty as usize] == widget.hash
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl HmGui {
    /// Begin GUI declaration. Region is limited by [0, 0] - [sx, sy] rectangle.
    pub fn begin_gui(&mut self, sx: f32, sy: f32, input: &Input) {
        let root = &mut self.root.as_mut();

        root.inner_pos = Vec2::ZERO;
        root.pos = root.inner_pos;
        root.inner_size = Vec2::new(sx, sy);
        root.size = root.inner_size;

        self.apply_widget_properties(root.borrow_mut());

        let root_container = root.get_container_item_mut();
        root_container.children.clear();
        root_container.children_hash = 0;

        root_container.spacing = self
            .get_property_value(HmGuiProperties::ContainerSpacing.id())
            .get_f32();

        self.container = self.root.clone();
        self.last = self.root.clone();

        self.activate = input.mouse().is_pressed(MouseControl::Left);
    }

    /// Finish GUI declaration, calculate hierarchy widgets sizes and layout.
    // TODO: do not calculate layout for the widgets that go out of the screen. If possible.
    pub fn end_gui(&mut self, input: &Input) {
        unsafe { Profiler_Begin(c_str!("HmGui_End")) };

        // NOTE: Scope is needed to avoid borrow conflict with check_focus below
        {
            let root_rf = self.root.clone();
            let mut root = root_rf.as_mut();

            root.compute_size(self);
            root.layout(self);
        }

        self.focus_pos = input.mouse().position();
        self.mouse_over_widget_hash.fill(0);

        self.check_mouse_over(self.root.clone());

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

        let root_rf = self.root.clone();
        let root = root_rf.as_ref();

        root.draw(self);

        self.renderer.end();

        unsafe { RenderState_PopBlendMode() };

        self.renderer.draw();

        unsafe { Profiler_End() };
    }

    /// Start a new container with a specified layout.
    fn begin_container(&mut self, layout: LayoutType) {
        let spacing = self
            .get_property_value(HmGuiProperties::ContainerSpacing.id())
            .get_f32();
        let clip = self
            .get_property_value(HmGuiProperties::ContainerClip.id())
            .get_bool();

        let container = HmGuiContainer {
            layout,
            spacing,
            clip,
            ..Default::default()
        };

        let widget_rf = self.init_widget(WidgetItem::Container(container));

        self.container = widget_rf.clone();
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
        self.last = self.container.clone();

        // We always have a parent since since we don't call end_container for root
        let Some(parent) = self.container.as_ref().parent.clone() else {
            unreachable!()
        };
        self.container = parent;
    }

    /// Start scroll area.
    ///
    /// Internally scroll area represented by 2 nested stack containers for a area itself
    /// and 2 other containers for scroll bars. So it is possible to set layout parameters
    /// for both external and internal containers. For the former parameters should be
    /// specified after `Gui:end_scroll_area()` function call and for the latter after
    /// `Gui:beginScrollArea()`.
    ///
    /// Parameters:
    /// **dir** - define directions in which scrolling is enabled: All, Horizontal, Vertical.
    ///
    /// Example:
    /// ```lua
    /// Gui:setPropertyValue(GuiProperties.ScrollAreaHScrollShow, GuiPropertyValue.FromBool(false))
    /// Gui:beginScrollArea(ScrollDirection.All)
    ///
    /// Gui:beginVerticalContainer()
    /// Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Top)
    /// Gui:setChildrenAlignment(AlignHorizontal.Stretch, AlignVertical.Top)
    ///
    /// Gui:button("Button1")
    /// Gui:button("Button2")
    ///
    /// Gui:endContainer()
    /// Gui:endScrollArea(InputInstance)
    /// Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)
    /// Gui:setFixedSize(500, 500)
    /// ```
    pub fn begin_scroll_area(&mut self, dir: ScrollDirection) {
        self.begin_stack_container();

        self.begin_stack_container();
        self.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);

        let widget_rf = self.container.clone();
        let mut widget = widget_rf.as_mut();
        let container = widget.get_container_item_mut();

        container.scroll_dir = Some(dir);
    }

    /// End of the scroll area.
    ///
    /// See [`HmGui::begin_scroll_area`] for example.
    pub fn end_scroll_area(&mut self, input: &Input) {
        let (max_scroll_x, max_scroll_y, inner_widget_hash, allow_hscroll, allow_vscroll) = {
            let widget_rf = self.container.clone();
            let mut widget = widget_rf.as_mut();

            let data = self.get_data(widget.hash);

            let max_scroll_x = f32::max(0.0, data.min_size.x - data.size.x);
            let max_scroll_y = f32::max(0.0, data.min_size.y - data.size.y);

            data.offset.x = data.offset.x.clamp(0.0, max_scroll_x);
            data.offset.y = data.offset.y.clamp(0.0, max_scroll_y);

            let container = widget.get_container_item_mut();
            container.offset = -data.offset;

            let (allow_hscroll, allow_vscroll) = if let Some(dir) = container.scroll_dir {
                match dir {
                    ScrollDirection::All => (true, true),
                    ScrollDirection::Horizontal => (true, false),
                    ScrollDirection::Vertical => (false, true),
                }
            } else {
                (false, false)
            };

            (
                max_scroll_x,
                max_scroll_y,
                widget.hash,
                allow_hscroll,
                allow_vscroll,
            )
        };

        self.end_container();

        let hscroll = allow_hscroll
            && self
                .get_property_value(HmGuiProperties::ScrollAreaHScrollShow.id())
                .get_bool();
        let vscroll = allow_vscroll
            && self
                .get_property_value(HmGuiProperties::ScrollAreaVScrollShow.id())
                .get_bool();

        if hscroll || vscroll {
            let fading = self
                .get_property_value(HmGuiProperties::ScrollAreaScrollbarVisibilityFading.id())
                .get_bool();
            let fade_scale = {
                let scroll_scale = self
                    .get_property_value(HmGuiProperties::ScrollAreaScrollScale.id())
                    .get_f32();
                let is_mouse_over = self.is_mouse_over(FocusType::Scroll);
                let mut scroll = input.mouse().scroll();

                if input.keyboard().is_down(KeyboardButton::ShiftLeft) {
                    let scroll_x = scroll.y;
                    scroll = Vec2::new(scroll_x, 0.0);
                }

                let widget_rf = self.container.clone();
                let widget = widget_rf.as_ref();

                let data = self.get_data(widget.hash);

                let fade_scale = if !fading {
                    1.0
                } else if is_mouse_over
                    && (scroll.length() > 0.3 || input.mouse().delta().length() > 0.5)
                {
                    data.scrollbar_activation_time = Instant::now();
                    1.0
                } else {
                    let elapsed_time = Instant::now() - data.scrollbar_activation_time;
                    let stable_time = Duration::from_millis(
                        self.get_property_value(
                            HmGuiProperties::ScrollAreaScrollbarVisibilityStableTime.id(),
                        )
                        .get_u64(),
                    );
                    let fade_time = Duration::from_millis(
                        self.get_property_value(
                            HmGuiProperties::ScrollAreaScrollbarVisibilityFadeTime.id(),
                        )
                        .get_u64(),
                    );

                    if elapsed_time <= stable_time {
                        1.0
                    } else if elapsed_time <= stable_time + fade_time {
                        1.0 - (elapsed_time - stable_time).as_millis() as f32
                            / fade_time.as_millis() as f32
                    } else {
                        0.0
                    }
                };

                if is_mouse_over {
                    let data = self.get_data(inner_widget_hash);
                    data.offset -= scroll * scroll_scale;
                }

                fade_scale
            };

            if fade_scale > 0.0 {
                let sb_length = self
                    .get_property_value(HmGuiProperties::ScrollAreaScrollbarLength.id())
                    .get_f32();

                let mut sb_bg_color = *self
                    .get_property_value(HmGuiProperties::ScrollAreaScrollbarBackgroundColor.id())
                    .get_color();
                sb_bg_color.a *= fade_scale;

                let mut sb_knob_color = *self
                    .get_property_value(HmGuiProperties::ScrollAreaScrollbarKnobColor.id())
                    .get_color();

                sb_knob_color.a *= fade_scale;

                if hscroll && max_scroll_x > 0.0 {
                    self.begin_horizontal_container();
                    self.set_alignment(AlignHorizontal::Stretch, AlignVertical::Bottom);
                    self.set_spacing(0.0);

                    let (handle_size, handle_pos) = {
                        let data = self.get_data(inner_widget_hash);
                        let handle_size = data.size.x * (data.size.x / data.min_size.x);
                        let handle_pos = Lerp(
                            0.0f64,
                            (data.size.x - handle_size) as f64,
                            (data.offset.x / max_scroll_x) as f64,
                        ) as f32;

                        (handle_size, handle_pos)
                    };

                    self.set_property(HmGuiProperties::BackgroundColor, sb_bg_color);
                    self.rect();
                    self.set_fixed_size(handle_pos, sb_length);

                    self.set_property(HmGuiProperties::BackgroundColor, sb_knob_color);
                    self.rect();
                    self.set_fixed_size(handle_size, sb_length);

                    self.set_property(HmGuiProperties::BackgroundColor, sb_bg_color);
                    self.rect();
                    self.set_fixed_height(sb_length);
                    self.set_horizontal_alignment(AlignHorizontal::Stretch);

                    self.end_container();
                }

                if vscroll && max_scroll_y > 0.0 {
                    self.begin_vertical_container();
                    self.set_alignment(AlignHorizontal::Right, AlignVertical::Stretch);
                    self.set_spacing(0.0);

                    let (handle_size, handle_pos) = {
                        let data = self.get_data(inner_widget_hash);
                        let handle_size = data.size.y * (data.size.y / data.min_size.y);
                        let handle_pos = Lerp(
                            0.0f64,
                            (data.size.y - handle_size) as f64,
                            (data.offset.y / max_scroll_y) as f64,
                        ) as f32;

                        (handle_size, handle_pos)
                    };

                    self.set_property(HmGuiProperties::BackgroundColor, sb_bg_color);
                    self.rect();
                    self.set_fixed_size(sb_length, handle_pos);

                    self.set_property(HmGuiProperties::BackgroundColor, sb_knob_color);
                    self.rect();
                    self.set_fixed_size(sb_length, handle_size);

                    self.set_property(HmGuiProperties::BackgroundColor, sb_bg_color);
                    self.rect();
                    self.set_fixed_width(sb_length);
                    self.set_vertical_alignment(AlignVertical::Stretch);

                    self.end_container();
                }
            }
        }

        self.end_container();
    }

    /// Invisible element that stretches in all directions.
    /// Use for pushing neighbor elements to the sides. See [`Self::checkbox`] for example.
    pub fn spacer(&mut self) {
        self.set_property(HmGuiProperties::BackgroundColor, Color::TRANSPARENT);
        self.rect();
        self.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);
    }

    pub fn checkbox(&mut self, label: &str, mut value: bool) -> bool {
        self.map_property_group("checkbox.rect");
        self.begin_horizontal_container();
        self.set_padding(4.0, 4.0);
        self.set_spacing(8.0);
        self.set_children_vertical_alignment(AlignVertical::Center);

        let is_mouse_over = self.is_mouse_over(FocusType::Mouse);
        if is_mouse_over && self.activate {
            value = !value;
        }

        self.map_property_group("checkbox.text");
        self.text(label);

        // Push text and rect to the sides if outer container has horizontal stretch
        self.spacer();

        // checkbox itself
        let bg_color = if value {
            *self
                .get_property_value(HmGuiProperties::CheckboxClickAreaSelectedColor.id())
                .get_color()
        } else {
            Color::TRANSPARENT
        };

        self.map_property_group("checkbox.click-area");
        self.set_property(HmGuiProperties::BackgroundColor, bg_color);
        self.rect();
        self.set_fixed_size(10.0, 10.0);
        self.set_border_width(3.0);

        self.end_container();
        // TODO: workaround. fix it
        self.set_property(HmGuiProperties::BackgroundColor, Color::TRANSPARENT);

        value
    }

    pub fn image(&mut self, image: &mut Tex2D) {
        let image_item = HmGuiImage { image };

        let _ = self.init_widget(WidgetItem::Image(image_item));
    }

    pub fn rect(&mut self) {
        let _ = self.init_widget(WidgetItem::Rect);
    }

    pub fn text(&mut self, text: &str) {
        let font = self
            .get_property_value(HmGuiProperties::TextFont.id())
            .get_font();
        let color = self
            .get_property_value(HmGuiProperties::TextColor.id())
            .get_color();

        // NOTE: cannot call text_ex() here because of mutable/immutable borrow conflict
        let item = HmGuiText {
            text: text.into(),
            font: font.clone(),
            color: color.clone(),
        };
        let size = item.font.get_size2(text);
        let widget_rf = self.init_widget(WidgetItem::Text(item));
        let mut widget = widget_rf.as_mut();

        widget.inner_min_size = Vec2::new(size.x as f32, size.y as f32);
    }

    pub fn text_colored(&mut self, text: &str, color: &Color) {
        let font = self
            .get_property_value(HmGuiProperties::TextFont.id())
            .get_font();

        // NOTE: cannot call text_ex() here because of mutable/immutable borrow conflict
        let item = HmGuiText {
            font: font.clone(),
            text: text.into(),
            color: color.clone(),
        };
        let size = item.font.get_size2(text);
        let widget_rf = self.init_widget(WidgetItem::Text(item));
        let mut widget = widget_rf.as_mut();

        widget.inner_min_size = Vec2::new(size.x as f32, size.y as f32);
    }

    pub fn text_ex(&mut self, font: &Font, text: &str, color: &Color) {
        let item = HmGuiText {
            font: font.clone().into(),
            text: text.into(),
            color: color.clone(),
        };
        let size = item.font.get_size2(text);
        let widget_rf = self.init_widget(WidgetItem::Text(item));
        let mut widget = widget_rf.as_mut();

        widget.inner_min_size = Vec2::new(size.x as f32, size.y as f32);
    }

    /// Makes current widget `focusable` and returns true if mouse is over it.
    pub fn is_mouse_over(&self, ty: FocusType) -> bool {
        let mut widget = self.last.as_mut();

        self.is_mouse_over_intern(&mut widget, ty)
    }

    pub fn set_min_width(&self, width: f32) {
        let mut widget = self.last.as_mut();

        widget.inner_min_size.x = width;
    }

    pub fn set_min_height(&self, height: f32) {
        let mut widget = self.last.as_mut();

        widget.inner_min_size.y = height;
    }

    pub fn set_min_size(&self, width: f32, height: f32) {
        let mut widget = self.last.as_mut();

        widget.inner_min_size.x = width;
        widget.inner_min_size.y = height;
    }

    pub fn set_fixed_width(&self, width: f32) {
        let mut widget = self.last.as_mut();

        widget.default_size[0] = Length::Fixed(width);
    }

    pub fn set_fixed_height(&self, height: f32) {
        let mut widget = self.last.as_mut();

        widget.default_size[1] = Length::Fixed(height);
    }

    pub fn set_fixed_size(&self, width: f32, height: f32) {
        let mut widget = self.last.as_mut();

        widget.default_size[0] = Length::Fixed(width);
        widget.default_size[1] = Length::Fixed(height);
    }

    pub fn set_percent_width(&self, width: f32) {
        let mut widget = self.last.as_mut();

        widget.default_size[0] = Length::Percent(width);
    }

    pub fn set_percent_height(&self, height: f32) {
        let mut widget = self.last.as_mut();

        widget.default_size[1] = Length::Percent(height);
    }

    pub fn set_percent_size(&self, width: f32, height: f32) {
        let mut widget = self.last.as_mut();

        widget.default_size[0] = Length::Percent(width);
        widget.default_size[1] = Length::Percent(height);
    }

    pub fn set_margin(&self, px: f32, py: f32) {
        let mut widget = self.last.as_mut();

        widget.margin_lower = Vec2::new(px, py);
        widget.margin_upper = Vec2::new(px, py);
    }

    pub fn set_margin_ex(&self, left: f32, top: f32, right: f32, bottom: f32) {
        let mut widget = self.last.as_mut();

        widget.margin_lower = Vec2::new(left, top);
        widget.margin_upper = Vec2::new(right, bottom);
    }

    pub fn set_margin_left(&self, margin: f32) {
        let mut widget = self.last.as_mut();

        widget.margin_lower.x = margin;
    }

    pub fn set_margin_top(&self, margin: f32) {
        let mut widget = self.last.as_mut();

        widget.margin_lower.y = margin;
    }

    pub fn set_margin_right(&self, margin: f32) {
        let mut widget = self.last.as_mut();

        widget.margin_upper.x = margin;
    }

    pub fn set_margin_bottom(&self, margin: f32) {
        let mut widget = self.last.as_mut();

        widget.margin_upper.y = margin;
    }

    pub fn set_border_width(&self, width: f32) {
        let mut widget = self.last.as_mut();

        widget.border_width = width;
    }

    pub fn set_alignment(&self, h: AlignHorizontal, v: AlignVertical) {
        let mut widget = self.last.as_mut();

        widget.alignment = [h.into(), v.into()];
    }

    pub fn set_horizontal_alignment(&self, align: AlignHorizontal) {
        let mut widget = self.last.as_mut();

        widget.alignment[0] = align.into();
    }

    pub fn set_vertical_alignment(&self, align: AlignVertical) {
        let mut widget = self.last.as_mut();

        widget.alignment[1] = align.into();
    }

    pub fn set_padding(&self, px: f32, py: f32) {
        let mut widget = self.container.as_mut();
        let container = widget.get_container_item_mut();

        container.padding_lower = Vec2::new(px, py);
        container.padding_upper = Vec2::new(px, py);
    }

    pub fn set_padding_ex(&self, left: f32, top: f32, right: f32, bottom: f32) {
        let mut widget = self.container.as_mut();
        let container = widget.get_container_item_mut();

        container.padding_lower = Vec2::new(left, top);
        container.padding_upper = Vec2::new(right, bottom);
    }

    pub fn set_padding_left(&self, padding: f32) {
        let mut widget = self.container.as_mut();
        let container = widget.get_container_item_mut();

        container.padding_lower.x = padding;
    }

    pub fn set_padding_top(&self, padding: f32) {
        let mut widget = self.container.as_mut();
        let container = widget.get_container_item_mut();

        container.padding_lower.y = padding;
    }

    pub fn set_padding_right(&self, padding: f32) {
        let mut widget = self.container.as_mut();
        let container = widget.get_container_item_mut();

        container.padding_upper.x = padding;
    }

    pub fn set_padding_bottom(&self, padding: f32) {
        let mut widget = self.container.as_mut();
        let container = widget.get_container_item_mut();

        container.padding_upper.y = padding;
    }

    pub fn set_spacing(&self, spacing: f32) {
        let mut widget = self.container.as_mut();
        let container = widget.get_container_item_mut();

        container.spacing = spacing;
    }

    pub fn set_children_alignment(&self, h: AlignHorizontal, v: AlignVertical) {
        let mut widget = self.container.as_mut();
        let container = widget.get_container_item_mut();

        container.children_alignment = [h.into(), v.into()];
    }

    pub fn set_children_horizontal_alignment(&self, align: AlignHorizontal) {
        let mut widget = self.container.as_mut();
        let container = widget.get_container_item_mut();

        container.children_alignment[0] = align.into();
    }

    pub fn set_children_vertical_alignment(&self, align: AlignVertical) {
        let mut widget = self.container.as_mut();
        let container = widget.get_container_item_mut();

        container.children_alignment[1] = align.into();
    }

    // Theme methods ----------------------------------------------------------

    /// Set a theme by merging it into the default properties.
    pub fn set_theme(&mut self, name: &str) {
        let mut property_registry = self.default_property_registry.clone();

        self.theme_registry.merge_to(&mut property_registry, name);

        self.property_registry = property_registry;
    }

    /// Restore default properties.
    pub fn clear_theme(&mut self) {
        self.property_registry = self.default_property_registry.clone();
    }

    // Style methods ----------------------------------------------------------

    /// Create a new empty style.
    /// Returns style id or None/nil if style with the same name already exists.
    ///
    /// Example:
    /// ```lua
    /// local styleId = Gui:newStyle("MyStyle")
    /// Gui:setStyleProperty(GuiProperties.BackgroundColor, Color(1, 0, 0, 1))
    /// Gui:setStyleProperty(GuiProperties.Opacity, 0.5)
    ///
    /// -- Later in the code
    ///
    /// Gui:setStyle(styleId)
    /// Gui:beginStackContainer()
    ///
    /// Gui:endContainer()
    /// ```
    pub fn new_style(&mut self, name: &str) -> Option<usize> {
        self.style_registry.create_style(name).map(|id| *id)
    }

    /// Sets style property value.
    /// See example in `Gui:newStyle()` method description.
    pub fn set_style_property_value(
        &mut self,
        style_id: usize,
        prop_id: usize,
        value: &HmGuiPropertyValue,
    ) {
        let (_, prop) = self
            .default_property_registry
            .registry
            .get_index(prop_id)
            .unwrap_or_else(|| {
                panic!("Unknown property id {prop_id}");
            });
        assert_eq!(
            prop.value.get_type(),
            value.get_type(),
            "Wrong property type"
        );

        let style = self
            .style_registry
            .get_mut(style_id.into())
            .expect(&format!("Unknown style with id: {style_id:?}"));

        style.set_property_value(prop_id.into(), value);
    }

    /// Get style id by its name.
    pub fn get_style_id(&self, name: &str) -> usize {
        *self
            .style_registry
            .get_id(name)
            .expect(&format!("Unknown style: {name}"))
    }

    /// Set a style for the following element by its id.
    /// Completely replaces current style with a new one.
    pub fn set_style(&mut self, id: usize) {
        self.element_style = self
            .style_registry
            .get(id.into())
            .expect(&format!("Unknown style with id: {id:?}"))
            .clone();
    }

    /// Set a style for the following element by its name.
    /// Completely replaces current style with a new one.
    /// NOTE: this method is slower than 'id' version.
    pub fn set_style_by_name(&mut self, name: &str) {
        self.element_style = self
            .style_registry
            .get_by_name(name)
            .expect(&format!("Unknown style: {name:?}"))
            .clone();
    }

    /// Remove element style.
    pub fn clear_style(&mut self) {
        self.element_style.properties.clear();
    }

    // Property methods -------------------------------------------------------

    /// Get property type by its id.
    pub fn get_property_type(&self, id: usize) -> HmGuiPropertyType {
        self.default_property_registry.registry[id].value.get_type()
    }

    /// Write property value into the mapped properties in the active element style.
    pub fn map_property(&mut self, property_id: usize) {
        let map_ids = &self.property_registry.registry[property_id].map_ids;
        if map_ids.is_empty() {
            return;
        }

        let prop = self.get_property_value(property_id).clone();

        for map_id in map_ids {
            self.element_style.properties.insert(*map_id, prop.clone());
        }
    }

    /// Write all properties values of the group into their mapped properties in the active element style.
    /// Example: `gui.map_property_group("button")`
    ///   It will map all properties with prefix "button.".
    pub fn map_property_group(&mut self, group: &str) {
        let prefix = format!("{group}.");
        let property_ids: Vec<_> = self
            .property_registry
            .registry
            .iter()
            .enumerate()
            .filter_map(|(property_id, (name, _))| name.starts_with(&prefix).then(|| property_id))
            .collect();

        for property_id in property_ids {
            self.map_property(property_id);
        }
    }

    /// Remove property by id from the active element style.
    pub fn remove_property(&mut self, property_id: usize) {
        self.element_style.properties.remove(&property_id.into());
    }

    pub fn register_property(
        &mut self,
        name: &str,
        value: &HmGuiPropertyValue,
        map_id: Option<&str>,
    ) -> usize {
        let mut map_ids = vec![];

        if let Some(map_id_str) = map_id {
            let (map_id, _, _) = self
                .default_property_registry
                .registry
                .get_full(map_id_str)
                .unwrap_or_else(|| panic!("{name:?} has unknown map property: {map_id_str}"));

            map_ids.push(map_id.into());
        }

        let def_id = self
            .default_property_registry
            .register(name, value.clone(), &map_ids);
        let id = self
            .property_registry
            .register(name, value.clone(), &map_ids);
        debug_assert_eq!(def_id, id);

        *id
    }

    pub fn set_property_value(&mut self, id: usize, value: &HmGuiPropertyValue) {
        let (_, prop) = self
            .default_property_registry
            .registry
            .get_index(id)
            .unwrap_or_else(|| {
                panic!("Unknown property id {id}");
            });
        assert_eq!(
            prop.value.get_type(),
            value.get_type(),
            "Wrong property type"
        );

        self.element_style
            .properties
            .insert(id.into(), value.clone());
    }

    pub fn get_property_value(&self, id: usize) -> &HmGuiPropertyValue {
        let prop_id = id.into();
        if let Some(prop) = self.element_style.properties.get(&prop_id) {
            return prop;
        }

        if let Some((_, prop)) = self.property_registry.registry.get_index(id) {
            return &prop.value;
        }

        panic!("Unknown property id {id}");
    }

    /// Get number of registered properties.
    pub fn get_properties_count(&self) -> usize {
        self.default_property_registry.registry.len()
    }

    /// Prints widgets hierarchy to the console. For testing.
    pub fn dump_widgets(&self) {
        let container = self.root.as_ref();

        container.dump("GUI widgets", 1);
    }
}
