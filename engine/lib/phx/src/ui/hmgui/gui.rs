use std::collections::HashMap;

use glam::*;

use crate::common::*;
use crate::input::*;
use crate::math::*;
use crate::render::*;
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
    focus: [u64; 2],
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
            layout: LayoutType::None,
            spacing: 0.0,
            clip: true,
            ..Default::default()
        };

        let mut widget = HmGuiWidget::new(None, WidgetItem::Container(container));
        widget.hash = Hash_FNV64_Init();

        let root = Rf::new(widget);
        let container = root.clone();
        let last = root.clone();

        let property_registry = HmGuiPropertyRegistry::new();
        let default_property_registry = property_registry.clone();

        let f = |name: &str| {
            property_registry
                .registry
                .get_full(name)
                .map(|(id, _, prop)| (id.into(), prop.property.get_type()))
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

        let style_folders = Resource::get_folders(ResourceType::Style);
        let mut style_registry = HmGuiStyleRegistry::default();
        for folder_path in style_folders {
            let registry = HmGuiStyleRegistry::load(&folder_path, f);
            if registry.size() > 0 {
                style_registry = registry;
                break;
            }
        }

        Self {
            renderer: Default::default(),
            root,
            container,
            last,
            data: HashMap::with_capacity(128),
            focus: [0; 2],
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

    pub fn mouse_focus_hash(&self) -> u64 {
        self.focus[FocusType::Mouse as usize]
    }

    /// Add a new widget into the current container.
    fn init_widget(&mut self, item: WidgetItem) -> Rf<HmGuiWidget> {
        let parent_rf = self.container.clone();
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

        self.last = widget_rf.clone();

        widget_rf.clone()
    }

    /// Start a new container with specified layout.
    fn begin_container(&mut self, layout: LayoutType) {
        let spacing = self.get_property_f32(HmGuiProperties::ContainerSpacingId.id());

        let container = HmGuiContainer {
            layout,
            spacing,
            ..Default::default()
        };

        let widget_rf = self.init_widget(WidgetItem::Container(container));

        self.container = widget_rf.clone();
    }

    /// Get persistent data of the widget by its hash.
    pub fn get_data(&mut self, widget_hash: u64) -> &mut HmGuiData {
        self.data.entry(widget_hash).or_insert(HmGuiData::default())
    }

    #[inline]
    fn is_clipped(&self, pos: Vec2, size: Vec2, p: Vec2) -> bool {
        p.x < pos.x || p.y < pos.y || pos.x + size.x < p.x || pos.y + size.y < p.y
    }

    /// Recursively iterate over container widgets and calculate if they are in a focus (mouse is over the container).
    fn check_focus(&mut self, widget_rf: Rf<HmGuiWidget>) {
        let widget = widget_rf.as_ref();
        let WidgetItem::Container(container) = &widget.item else {
            return;
        };

        if container.clip && self.is_clipped(widget.pos, widget.size, self.focus_pos) {
            return;
        }

        for widget_rf in container.children.iter().rev() {
            self.check_focus(widget_rf.clone());
        }

        for i in 0..self.focus.len() {
            if self.focus[i] == 0
                && container.focusable[i]
                && widget.pos.x <= self.focus_pos.x
                && widget.pos.y <= self.focus_pos.y
                && self.focus_pos.x <= widget.pos.x + widget.size.x
                && self.focus_pos.y <= widget.pos.y + widget.size.y
            {
                self.focus[i] = widget.hash;
            }
        }
    }

    /// Sets container `focusable` flag to true and returns if it's currently in focus.
    fn container_has_focus_intern(
        &self,
        container: &mut HmGuiContainer,
        ty: FocusType,
        hash: u64,
    ) -> bool {
        container.focusable[ty as usize] = true;

        self.focus[ty as usize] == hash
    }

    fn get_property(&self, property_id: usize) -> &HmGuiProperty {
        let id = property_id.into();
        if let Some(prop) = self.element_style.properties.get(&id) {
            return prop;
        }

        if let Some((_, prop)) = self.property_registry.registry.get_index(property_id) {
            return &prop.property;
        }

        panic!("Unknown property id {property_id}");
    }
}

macro_rules! set_property {
    ($self:ident, $id:ident, $val:expr) => {
        let Some((_, def_prop)) = $self.default_property_registry.registry.get_index($id) else {
            panic!("Unknown property id {}", $id);
        };
        let value: HmGuiProperty = $val.into();
        assert_eq!(
            def_prop.property.get_type(),
            value.get_type(),
            "Wrong property type"
        );

        $self.element_style.properties.insert($id.into(), value);
    };
}

macro_rules! get_property {
    ($self:ident, $id:ident, $v:ident) => {{
        let prop = $self.get_property($id);

        let HmGuiProperty::$v(value) = prop else {
            panic!(
                "Wrong property type. Expected {} but was {}",
                stringify!($v),
                prop.name()
            )
        };

        value
    }};
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

        let root_container = root.get_container_item_mut();
        root_container.children.clear();
        root_container.children_hash = 0;

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
        self.focus.fill(0);

        self.check_focus(self.root.clone());

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

    pub fn begin_horizontal_container(&mut self) {
        self.begin_container(LayoutType::Horizontal);
    }

    pub fn begin_vertical_container(&mut self) {
        self.begin_container(LayoutType::Vertical);
    }

    pub fn begin_stack_container(&mut self) {
        self.begin_container(LayoutType::Stack);
    }

    pub fn end_container(&mut self) {
        self.last = self.container.clone();

        // We always have a parent since since we don't call end_container for root
        let Some(parent) = self.container.as_ref().parent.clone() else {
            unreachable!()
        };
        self.container = parent;
    }

    pub fn begin_scroll(&mut self, _max_size: f32) {
        let widget_rf = self.container.clone();
        let mut widget = widget_rf.as_mut();
        let widget_hash = widget.hash;
        let container = widget.get_container_item_mut();

        self.begin_horizontal_container();
        self.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);
        self.set_spacing(2.0);

        container.clip = true;

        self.begin_vertical_container();
        self.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);
        self.set_padding(6.0, 6.0);

        container.store_size = true;

        let data = self.get_data(widget_hash);

        container.offset.x = -data.offset.x;
        container.offset.y = -data.offset.y;
    }

    pub fn end_scroll(&mut self, input: &Input) {
        let widget_rf = self.container.clone();
        let widget = widget_rf.as_ref();
        let has_focus = self.container_has_focus(FocusType::Scroll);

        let data = self.get_data(widget.hash);

        if has_focus {
            let scroll_x = input.mouse().value(MouseControl::ScrollX);
            let scroll_y = input.mouse().value(MouseControl::ScrollY);

            data.offset.x -= 10.0 * scroll_x as f32;
            data.offset.y -= 10.0 * scroll_y as f32;
        }

        let max_scroll_x = f32::max(0.0, data.min_size.x - data.size.x);
        let max_scroll_y = f32::max(0.0, data.min_size.y - data.size.y);

        data.offset.x = data.offset.x.clamp(0.0, max_scroll_x);
        data.offset.y = data.offset.y.clamp(0.0, max_scroll_y);

        self.end_container();

        self.begin_vertical_container();
        self.set_vertical_alignment(AlignVertical::Stretch);
        self.set_spacing(0.0);

        if max_scroll_x > 0.0 {
            let (handle_size, handle_pos) = {
                let data = self.get_data(widget.hash);
                let handle_size = data.size.x * (data.size.x / data.min_size.x);
                let handle_pos = Lerp(
                    0.0f64,
                    (data.size.x - handle_size) as f64,
                    (data.offset.x / max_scroll_x) as f64,
                ) as f32;

                (handle_size, handle_pos)
            };

            self.rect(0.0, 0.0, 0.0, 0.0);
            self.set_fixed_size(handle_pos, 4.0);

            let color_frame = self.get_property_vec4(HmGuiProperties::ContainerColorFrameId.id());

            self.rect(color_frame.x, color_frame.y, color_frame.z, color_frame.w);
            self.set_fixed_size(handle_size, 4.0);
        } else {
            self.rect(0.0, 0.0, 0.0, 0.0);
            self.set_fixed_size(16.0, 4.0);
        }

        if max_scroll_y > 0.0 {
            let (handle_size, handle_pos) = {
                let data = self.get_data(widget.hash);
                let handle_size = data.size.y * (data.size.y / data.min_size.y);
                let handle_pos = Lerp(
                    0.0f64,
                    (data.size.y - handle_size) as f64,
                    (data.offset.y / max_scroll_y) as f64,
                ) as f32;

                (handle_size, handle_pos)
            };

            self.rect(0.0, 0.0, 0.0, 0.0);
            self.set_fixed_size(4.0, handle_pos);

            let color_frame = self.get_property_vec4(HmGuiProperties::ContainerColorFrameId.id());

            self.rect(color_frame.x, color_frame.y, color_frame.z, color_frame.w);
            self.set_fixed_size(4.0, handle_size);
        } else {
            self.rect(0.0, 0.0, 0.0, 0.0);
            self.set_fixed_size(4.0, 16.0);
        }

        self.end_container();

        self.end_container();
    }

    /// Begins window element.
    // TODO: refactor to draw title properly
    pub fn begin_window(&mut self, _title: &str, input: &Input) {
        self.begin_stack_container();

        // A separate scope to prevent runtime borrow conflict with self.begin_vertical_container() below
        {
            let mouse = input.mouse();
            let has_focus = self.container_has_focus(FocusType::Mouse);

            let widget_rf = self.container.clone();
            let mut widget = widget_rf.as_mut();
            let data = self.get_data(widget.hash);

            if has_focus && mouse.is_down(MouseControl::Left) {
                data.offset.x += mouse.value(MouseControl::DeltaX);
                data.offset.y += mouse.value(MouseControl::DeltaY);
            }

            widget.pos.x += data.offset.x;
            widget.pos.y += data.offset.y;

            let container = widget.get_container_item_mut();
            container.focus_style = FocusStyle::None;
            container.frame_opacity = 0.95;
            container.clip = true;
        }

        self.begin_vertical_container();
        self.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);
        self.set_padding(8.0, 8.0);
    }

    /// Ends window element.
    pub fn end_window(&mut self) {
        self.end_container(); // Vertical container
        self.end_container(); // Stack container
    }

    /// Invisible element that stretches in all directions.
    /// Use for pushing neighbor elements to the sides. See [`Self::checkbox`] for example.
    pub fn spacer(&mut self) {
        self.rect(0.0, 0.0, 0.0, 0.0);
        self.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);
    }

    pub fn button(&mut self, label: &str) -> bool {
        self.begin_stack_container();
        self.set_padding(8.0, 8.0);

        // A separate scope to prevent runtime borrow panics - widget borrowing conflicts with self.text() below
        let focus = {
            let mut widget = self.container.as_mut();
            let hash = widget.hash;
            let container = widget.get_container_item_mut();

            container.focus_style = FocusStyle::Fill;
            container.frame_opacity = 0.5;

            self.container_has_focus_intern(container, FocusType::Mouse, hash)
        };

        self.text(label);
        self.set_alignment(AlignHorizontal::Center, AlignVertical::Center);

        let pressed = focus && self.activate;

        self.end_container();

        pressed
    }

    pub fn checkbox(&mut self, label: &str, mut value: bool) -> bool {
        self.begin_horizontal_container();
        self.set_padding(4.0, 4.0);
        self.set_spacing(8.0);
        self.set_children_vertical_alignment(AlignVertical::Center);

        // A separate scope to prevent runtime borrow conflict with self.text() below
        {
            let mut widget = self.container.as_mut();
            let hash = widget.hash;
            let container = widget.get_container_item_mut();

            container.focus_style = FocusStyle::Underline;

            let focus = self.container_has_focus_intern(container, FocusType::Mouse, hash);

            if focus && self.activate {
                value = !value;
            }
        }

        self.text(label);

        // Push text and rect to the sides if outer container has horizontal stretch
        self.spacer();

        // TODO: replace with rect with border
        self.begin_stack_container();
        self.set_children_alignment(AlignHorizontal::Center, AlignVertical::Center);

        let (color_frame, color_primary) = {
            let color_frame = self.get_property_vec4(HmGuiProperties::ContainerColorFrameId.id());
            let color_primary =
                self.get_property_vec4(HmGuiProperties::ContainerColorPrimaryId.id());

            (color_frame.clone(), color_primary.clone())
        };

        self.rect(color_frame.x, color_frame.y, color_frame.z, color_frame.w);
        self.set_fixed_size(16.0, 16.0);

        if value {
            self.rect(
                color_primary.x,
                color_primary.y,
                color_primary.z,
                color_primary.w,
            );
            self.set_fixed_size(10.0, 10.0);
        }

        self.end_container();
        self.end_container();

        value
    }

    pub fn slider(&mut self, _lower: f32, _upper: f32, _value: f32) -> f32 {
        self.begin_stack_container();
        self.set_horizontal_alignment(AlignHorizontal::Stretch);

        self.rect(0.5, 0.5, 0.5, 1.0);
        self.set_fixed_size(0.0, 2.0);

        self.end_container();

        0.0
    }

    pub fn horizontal_divider(&mut self, height: f32, r: f32, g: f32, b: f32, a: f32) {
        self.rect(r, g, b, a);
        self.set_fixed_height(height);
        self.set_horizontal_alignment(AlignHorizontal::Stretch);
    }

    pub fn vertical_divider(&mut self, width: f32, r: f32, g: f32, b: f32, a: f32) {
        self.rect(r, g, b, a);
        self.set_fixed_width(width);
        self.set_vertical_alignment(AlignVertical::Stretch);
    }

    pub fn image(&mut self, image: &mut Tex2D) {
        let image_item = HmGuiImage { image };

        let _widget_rf = self.init_widget(WidgetItem::Image(image_item));
    }

    pub fn rect(&mut self, r: f32, g: f32, b: f32, a: f32) {
        let rect_item = HmGuiRect {
            color: Vec4::new(r, g, b, a),
        };

        self.init_widget(WidgetItem::Rect(rect_item));
    }

    pub fn text(&mut self, text: &str) {
        let font = self.get_property_font(HmGuiProperties::TextFontId.id());
        let color = self.get_property_vec4(HmGuiProperties::TextColorId.id());

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

    pub fn text_colored(&mut self, text: &str, r: f32, g: f32, b: f32, a: f32) {
        let font = self.get_property_font(HmGuiProperties::TextFontId.id());

        // NOTE: cannot call text_ex() here because of mutable/immutable borrow conflict
        let item = HmGuiText {
            font: font.clone(),
            text: text.into(),
            color: Vec4::new(r, g, b, a),
        };
        let size = item.font.get_size2(text);
        let widget_rf = self.init_widget(WidgetItem::Text(item));
        let mut widget = widget_rf.as_mut();

        widget.inner_min_size = Vec2::new(size.x as f32, size.y as f32);
    }

    pub fn text_ex(&mut self, font: &Font, text: &str, r: f32, g: f32, b: f32, a: f32) {
        let item = HmGuiText {
            font: font.clone().into(),
            text: text.into(),
            color: Vec4::new(r, g, b, a),
        };
        let size = item.font.get_size2(text);
        let widget_rf = self.init_widget(WidgetItem::Text(item));
        let mut widget = widget_rf.as_mut();

        widget.inner_min_size = Vec2::new(size.x as f32, size.y as f32);
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

        widget.default_width = Some(Length::Fixed(width));
    }

    pub fn set_fixed_height(&self, height: f32) {
        let mut widget = self.last.as_mut();

        widget.default_height = Some(Length::Fixed(height));
    }

    pub fn set_fixed_size(&self, width: f32, height: f32) {
        let mut widget = self.last.as_mut();

        widget.default_width = Some(Length::Fixed(width));
        widget.default_height = Some(Length::Fixed(height));
    }

    pub fn set_percent_width(&self, width: f32) {
        let mut widget = self.last.as_mut();

        widget.default_width = Some(Length::Percent(width));
    }

    pub fn set_percent_height(&self, height: f32) {
        let mut widget = self.last.as_mut();

        widget.default_height = Some(Length::Percent(height));
    }

    pub fn set_percent_size(&self, width: f32, height: f32) {
        let mut widget = self.last.as_mut();

        widget.default_width = Some(Length::Percent(width));
        widget.default_height = Some(Length::Percent(height));
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

    pub fn set_border_color(&self, r: f32, g: f32, b: f32, a: f32) {
        let mut widget = self.last.as_mut();

        widget.border_color = Vec4::new(r, g, b, a);
    }

    pub fn set_border_color_v4(&self, color: &Vec4) {
        let mut widget = self.last.as_mut();

        widget.border_color = *color;
    }

    pub fn set_border(&self, width: f32, r: f32, g: f32, b: f32, a: f32) {
        let mut widget = self.last.as_mut();

        widget.border_width = width;
        widget.border_color = Vec4::new(r, g, b, a);
    }

    pub fn set_border_v4(&self, width: f32, color: &Vec4) {
        let mut widget = self.last.as_mut();

        widget.border_width = width;
        widget.border_color = *color;
    }

    pub fn set_bg_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        let mut widget = self.last.as_mut();

        widget.bg_color = Some(Vec4::new(r, g, b, a));
    }

    pub fn set_bg_color_v4(&mut self, color: &Vec4) {
        let mut widget = self.last.as_mut();

        widget.bg_color = Some(*color);
    }

    pub fn set_alignment(&self, h: AlignHorizontal, v: AlignVertical) {
        let mut widget = self.last.as_mut();

        widget.horizontal_alignment = h;
        widget.vertical_alignment = v;
    }

    pub fn set_horizontal_alignment(&self, align: AlignHorizontal) {
        let mut widget = self.last.as_mut();

        widget.horizontal_alignment = align;
    }

    pub fn set_vertical_alignment(&self, align: AlignVertical) {
        let mut widget = self.last.as_mut();

        widget.vertical_alignment = align;
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

    /// Makes current container `focusable` and returns if it's currently in focus.
    pub fn container_has_focus(&self, ty: FocusType) -> bool {
        let mut widget = self.container.as_mut();
        let hash = widget.hash;
        let container = widget.get_container_item_mut();

        self.container_has_focus_intern(container, ty, hash)
    }

    pub fn set_children_alignment(&self, h: AlignHorizontal, v: AlignVertical) {
        let mut widget = self.container.as_mut();
        let container = widget.get_container_item_mut();

        container.children_horizontal_alignment = h;
        container.children_vertical_alignment = v;
    }

    pub fn set_children_horizontal_alignment(&self, align: AlignHorizontal) {
        let mut widget = self.container.as_mut();
        let container = widget.get_container_item_mut();

        container.children_horizontal_alignment = align;
    }

    pub fn set_children_vertical_alignment(&self, align: AlignVertical) {
        let mut widget = self.container.as_mut();
        let container = widget.get_container_item_mut();

        container.children_vertical_alignment = align;
    }

    // Theme methods ----------------------------------------------------------

    pub fn set_theme(&mut self, name: &str) {
        let mut property_registry = self.default_property_registry.clone();

        self.theme_registry.merge_to(&mut property_registry, name);

        self.property_registry = property_registry;
    }

    pub fn clear_theme(&mut self) {
        self.property_registry = self.default_property_registry.clone();
    }

    // Style methods ----------------------------------------------------------

    pub fn get_style_id(&self, name: &str) -> usize {
        *self
            .style_registry
            .get_id(name)
            .expect(&format!("Unknown style: {name}"))
    }

    pub fn set_style(&mut self, id: usize) {
        self.element_style = self
            .style_registry
            .get(id.into())
            .expect(&format!("Unknown style with id: {id:?}"))
            .clone();
    }

    pub fn clear_style(&mut self) {
        self.element_style.properties.clear();
    }

    // Property methods -------------------------------------------------------

    // TODO: map_ids: Vec<usize>
    pub fn register_property_bool(
        &mut self,
        name: &str,
        value: bool,
        map_id: Option<&str>,
    ) -> usize {
        let mut map_ids = vec![];
        if let Some(map_id_str) = map_id {
            let (map_id, _, _) = self
                .default_property_registry
                .registry
                .get_full(map_id_str)
                .unwrap_or_else(|| panic!("Unknown property: {map_id_str}"));

            map_ids.push(map_id.into());
        }

        let def_id = self
            .default_property_registry
            .register(name, value.into(), &map_ids);
        let id = self
            .property_registry
            .register(name, value.into(), &map_ids);
        debug_assert_eq!(def_id, id);

        *id
    }

    pub fn get_property_type(&self, id: usize) -> HmGuiPropertyType {
        self.default_property_registry.registry[id]
            .property
            .get_type()
    }

    pub fn map_property(&mut self, property_id: usize) {
        if let Some(prop) = self
            .element_style
            .properties
            .get(&property_id.into())
            .cloned()
        {
            let map_ids = &self.property_registry.registry[property_id].map_ids;

            for map_id in map_ids {
                self.element_style.properties.insert(*map_id, prop.clone());
            }
        }
    }

    pub fn remove_property(&mut self, property_id: usize) {
        self.element_style.properties.remove(&property_id.into());
    }

    pub fn set_property_bool(&mut self, property_id: usize, value: bool) {
        set_property!(self, property_id, value);
    }

    pub fn set_property_i8(&mut self, property_id: usize, value: i8) {
        set_property!(self, property_id, value);
    }

    pub fn set_property_u8(&mut self, property_id: usize, value: u8) {
        set_property!(self, property_id, value);
    }

    pub fn set_property_i16(&mut self, property_id: usize, value: i16) {
        set_property!(self, property_id, value);
    }

    pub fn set_property_u16(&mut self, property_id: usize, value: u16) {
        set_property!(self, property_id, value);
    }

    pub fn set_property_i32(&mut self, property_id: usize, value: i32) {
        set_property!(self, property_id, value);
    }

    pub fn set_property_u32(&mut self, property_id: usize, value: u32) {
        set_property!(self, property_id, value);
    }

    pub fn set_property_i64(&mut self, property_id: usize, value: i64) {
        set_property!(self, property_id, value);
    }

    pub fn set_property_u64(&mut self, property_id: usize, value: u64) {
        set_property!(self, property_id, value);
    }

    pub fn set_property_f32(&mut self, property_id: usize, value: f32) {
        set_property!(self, property_id, value);
    }

    pub fn set_property_f64(&mut self, property_id: usize, value: f64) {
        set_property!(self, property_id, value);
    }

    pub fn set_property_vec2(&mut self, property_id: usize, value: Vec2) {
        set_property!(self, property_id, value);
    }

    pub fn set_property_vec3(&mut self, property_id: usize, value: &Vec3) {
        set_property!(self, property_id, value.clone());
    }

    pub fn set_property_vec4(&mut self, property_id: usize, value: &Vec4) {
        set_property!(self, property_id, value.clone());
    }

    #[bind(name = "SetPropertyIVec2")]
    pub fn set_property_ivec2(&mut self, property_id: usize, value: IVec2) {
        set_property!(self, property_id, value);
    }

    #[bind(name = "SetPropertyIVec3")]
    pub fn set_property_ivec3(&mut self, property_id: usize, value: &IVec3) {
        set_property!(self, property_id, value.clone());
    }

    #[bind(name = "SetPropertyIVec4")]
    pub fn set_property_ivec4(&mut self, property_id: usize, value: &IVec4) {
        set_property!(self, property_id, value.clone());
    }

    #[bind(name = "SetPropertyUVec2")]
    pub fn set_property_uvec2(&mut self, property_id: usize, value: UVec2) {
        set_property!(self, property_id, value);
    }

    #[bind(name = "SetPropertyUVec3")]
    pub fn set_property_uvec3(&mut self, property_id: usize, value: &UVec3) {
        set_property!(self, property_id, value.clone());
    }

    #[bind(name = "SetPropertyUVec4")]
    pub fn set_property_uvec4(&mut self, property_id: usize, value: &UVec4) {
        set_property!(self, property_id, value.clone());
    }

    #[bind(name = "SetPropertyDVec2")]
    pub fn set_property_dvec2(&mut self, property_id: usize, value: DVec2) {
        set_property!(self, property_id, value);
    }

    #[bind(name = "SetPropertyDVec3")]
    pub fn set_property_dvec3(&mut self, property_id: usize, value: &DVec3) {
        set_property!(self, property_id, value.clone());
    }

    #[bind(name = "SetPropertyDVec4")]
    pub fn set_property_dvec4(&mut self, property_id: usize, value: &DVec4) {
        set_property!(self, property_id, value.clone());
    }

    pub fn set_property_box3(&mut self, property_id: usize, value: &Box3) {
        set_property!(self, property_id, value.clone());
    }

    pub fn set_property_string(&mut self, property_id: usize, value: &str) {
        set_property!(self, property_id, value.to_string());
    }

    pub fn set_property_font(&mut self, property_id: usize, value: &Font) {
        set_property!(self, property_id, value.clone());
    }

    pub fn get_property_bool(&self, property_id: usize) -> bool {
        *get_property!(self, property_id, Bool)
    }

    pub fn get_property_i8(&self, property_id: usize) -> i8 {
        *get_property!(self, property_id, I8)
    }

    pub fn get_property_u8(&self, property_id: usize) -> u8 {
        *get_property!(self, property_id, U8)
    }

    pub fn get_property_i16(&self, property_id: usize) -> i16 {
        *get_property!(self, property_id, I16)
    }

    pub fn get_property_u16(&self, property_id: usize) -> u16 {
        *get_property!(self, property_id, U16)
    }

    pub fn get_property_i32(&self, property_id: usize) -> i32 {
        *get_property!(self, property_id, I32)
    }

    pub fn get_property_u32(&self, property_id: usize) -> u32 {
        *get_property!(self, property_id, U32)
    }

    pub fn get_property_i64(&self, property_id: usize) -> i64 {
        *get_property!(self, property_id, I64)
    }

    pub fn get_property_u64(&self, property_id: usize) -> u64 {
        *get_property!(self, property_id, U64)
    }

    pub fn get_property_f32(&self, property_id: usize) -> f32 {
        *get_property!(self, property_id, F32)
    }

    pub fn get_property_f64(&self, property_id: usize) -> f64 {
        *get_property!(self, property_id, F64)
    }

    pub fn get_property_vec2(&self, property_id: usize) -> Vec2 {
        *get_property!(self, property_id, Vec2)
    }

    pub fn get_property_vec3(&self, property_id: usize) -> &Vec3 {
        get_property!(self, property_id, Vec3)
    }

    pub fn get_property_vec4(&self, property_id: usize) -> &Vec4 {
        get_property!(self, property_id, Vec4)
    }

    #[bind(name = "GetPropertyIVec2")]
    pub fn get_property_ivec2(&self, property_id: usize) -> IVec2 {
        *get_property!(self, property_id, IVec2)
    }

    #[bind(name = "GetPropertyIVec3")]
    pub fn get_property_ivec3(&self, property_id: usize) -> &IVec3 {
        get_property!(self, property_id, IVec3)
    }

    #[bind(name = "GetPropertyIVec4")]
    pub fn get_property_ivec4(&self, property_id: usize) -> &IVec4 {
        get_property!(self, property_id, IVec4)
    }

    #[bind(name = "GetPropertyUVec2")]
    pub fn get_property_uvec2(&self, property_id: usize) -> UVec2 {
        *get_property!(self, property_id, UVec2)
    }

    #[bind(name = "GetPropertyUVec3")]
    pub fn get_property_uvec3(&self, property_id: usize) -> &UVec3 {
        get_property!(self, property_id, UVec3)
    }

    #[bind(name = "GetPropertyUVec4")]
    pub fn get_property_uvec4(&self, property_id: usize) -> &UVec4 {
        get_property!(self, property_id, UVec4)
    }

    #[bind(name = "GetPropertyDVec2")]
    pub fn get_property_dvec2(&self, property_id: usize) -> DVec2 {
        *get_property!(self, property_id, DVec2)
    }

    #[bind(name = "GetPropertyDVec3")]
    pub fn get_property_dvec3(&self, property_id: usize) -> &DVec3 {
        get_property!(self, property_id, DVec3)
    }

    #[bind(name = "GetPropertyDVec4")]
    pub fn get_property_dvec4(&self, property_id: usize) -> &DVec4 {
        get_property!(self, property_id, DVec4)
    }

    pub fn get_property_box3(&self, property_id: usize) -> &Box3 {
        get_property!(self, property_id, Box3)
    }

    pub fn get_property_string(&self, property_id: usize) -> &str {
        get_property!(self, property_id, String).as_str()
    }

    pub fn get_property_font(&self, property_id: usize) -> &Font {
        get_property!(self, property_id, Font)
    }

    /// Prints widgets hierarchy to the console. For testing.
    pub fn dump_widgets(&self) {
        println!("Widgets:");

        let container = self.root.as_ref();

        container.dump(1);
    }
}
