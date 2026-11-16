mod alignment;
mod container;
mod data;
mod focus_type;
mod gui;
mod image;
mod layer;
mod look;
mod shape;
mod text;
mod text_line;
mod widget;

pub use alignment::*;
pub use container::*;
use data::*;
pub use focus_type::*;
pub use gui::*;
pub use layer::*;
pub use look::*;
pub use shape::*;
pub use text::*;
pub use text_line::*;
pub use widget::*;

pub use self::image::*;

const IDENT: &str = "  ";

// TODO fix tests for win and mac
#[cfg(target_os = "linux")]
#[cfg(test)]
mod tests {
    use std::cell::Ref;
    use std::path::PathBuf;

    use glam::Vec2;

    use super::*;
    use crate::input::Input;
    use crate::render::Renderer;

    static mut RESOURCES_INITIALIZED: bool = false;

    struct WidgetCheck(
        &'static str,             // Widget name
        (f32, f32),               // Widget position
        (f32, f32),               // Widget outer size
        (f32, f32),               // Widget inner size
        Option<Vec<WidgetCheck>>, // Container widget children. None for non-container widget
    );

    fn init_test() -> (HmGui, Input, Renderer) {
        #[allow(unsafe_code)] // TODO: remove
        unsafe {
            if !RESOURCES_INITIALIZED {
                let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../");

                std::env::set_current_dir(&path).unwrap_or_else(|err| {
                    panic!(
                        "Cannot set current directory to: {}. Error: {err}",
                        path.display()
                    )
                });

                RESOURCES_INITIALIZED = true;
            }
        }

        let mut r = Renderer::new_headless();
        let hmgui = HmGui::new(&mut r, 1.0);
        (hmgui, Default::default(), r)
    }

    fn check_widget(widget: &Ref<'_, HmGuiWidget>, expected: &WidgetCheck) {
        assert_eq!(
            (widget.pos, widget.size, widget.inner_size),
            (
                Vec2::new(expected.1.0, expected.1.1),
                Vec2::new(expected.2.0, expected.2.1),
                Vec2::new(expected.3.0, expected.3.1)
            ),
            "[{}] - Wrong widget position, outer or inner sizes",
            expected.0
        );

        if let Some(expected_children) = &expected.4 {
            let container = widget.get_container_item();
            assert_eq!(
                container.children.len(),
                expected_children.len(),
                "[{}] - Wrong number of children",
                expected.0
            );

            for (i, expected_child) in expected_children.iter().enumerate() {
                let child_widget_rf = container.children[i].clone();
                let child_widget = child_widget_rf.as_ref();

                check_widget(&child_widget, expected_child);
            }
        } else {
            assert!(
                !matches!(widget.item, WidgetItem::Container(_)),
                "[{}] - Expected non-container item",
                expected.0
            );
        }
    }

    #[test]
    fn test_hmgui_stack_layout_basic() {
        let (mut gui, input, mut r) = init_test();

        gui.begin_gui(300.0, 200.0);
        gui.begin_stack_container();
        gui.set_alignment(AlignHorizontal::Center, AlignVertical::Center);
        gui.set_children_alignment(AlignHorizontal::Center, AlignVertical::Center);

        gui.rect();
        gui.set_fixed_size(30.0, 20.0);

        gui.rect();
        gui.set_fixed_size(20.0, 30.0);

        gui.end_container();
        gui.end_gui(&mut r, &input);

        let root_widget_rf = gui.root();
        let root_widget = root_widget_rf.as_ref();

        check_widget(
            &root_widget,
            &WidgetCheck(
                "Root",
                (0.0, 0.0),
                (300.0, 200.0), // Root widget should always keep it's position and size
                (300.0, 200.0),
                Some(vec![WidgetCheck(
                    "Stack",
                    (135.0, 85.0),
                    (30.0, 30.0),
                    (30.0, 30.0),
                    Some(vec![
                        WidgetCheck("Rect1", (135.0, 90.0), (30.0, 20.0), (30.0, 20.0), None),
                        WidgetCheck("Rect2", (140.0, 85.0), (20.0, 30.0), (20.0, 30.0), None),
                    ]),
                )]),
            ),
        );
    }

    #[test]
    fn test_hmgui_stack_layout_stretch() {
        let (mut gui, input, mut r) = init_test();

        gui.begin_gui(300.0, 200.0);
        gui.begin_stack_container();
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);
        gui.set_children_alignment(AlignHorizontal::Center, AlignVertical::Center);

        gui.rect();
        gui.set_fixed_size(30.0, 20.0);

        gui.rect();
        gui.set_fixed_size(20.0, 30.0);

        gui.end_container();
        gui.end_gui(&mut r, &input);

        let root_widget_rf = gui.root();
        let root_widget = root_widget_rf.as_ref();

        check_widget(
            &root_widget,
            &WidgetCheck(
                "Root",
                (0.0, 0.0),
                (300.0, 200.0), // Root widget should always keep it's position and size
                (300.0, 200.0),
                Some(vec![WidgetCheck(
                    "Stack",
                    (0.0, 0.0),
                    (300.0, 200.0), // Stack container expanded so has the same position and size as root one
                    (300.0, 200.0), // Stack container expanded so has the same position and size as root one
                    Some(vec![
                        WidgetCheck("Rect1", (135.0, 90.0), (30.0, 20.0), (30.0, 20.0), None),
                        WidgetCheck("Rect2", (140.0, 85.0), (20.0, 30.0), (20.0, 30.0), None),
                    ]),
                )]),
            ),
        );
    }

    // Widget docking/stretch has priority over fixed size.
    #[test]
    fn test_hmgui_stack_layout_docking_fixed_size_priority() {
        let (mut gui, input, mut r) = init_test();

        gui.begin_gui(300.0, 200.0);
        gui.begin_stack_container();
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);
        gui.set_fixed_size(50.0, 50.0);
        gui.set_children_alignment(AlignHorizontal::Center, AlignVertical::Center);

        gui.rect();
        gui.set_fixed_size(30.0, 20.0);

        gui.rect();
        gui.set_fixed_size(20.0, 30.0);

        gui.end_container();
        gui.end_gui(&mut r, &input);

        let root_widget_rf = gui.root();
        let root_widget = root_widget_rf.as_ref();

        check_widget(
            &root_widget,
            &WidgetCheck(
                "Root",
                (0.0, 0.0),
                (300.0, 200.0), // Root widget should always keep it's position and size
                (300.0, 200.0),
                Some(vec![WidgetCheck(
                    "Stack",
                    (0.0, 0.0),
                    (300.0, 200.0), // Stack container expanded so has the same position and size as root one. Ignoring fixed size
                    (300.0, 200.0),
                    Some(vec![
                        WidgetCheck("Rect1", (135.0, 90.0), (30.0, 20.0), (30.0, 20.0), None),
                        WidgetCheck("Rect2", (140.0, 85.0), (20.0, 30.0), (20.0, 30.0), None),
                    ]),
                )]),
            ),
        );
    }

    // Test horizontal and vertical stretching in combination with the fixed size
    #[test]
    fn test_hmgui_stack_layout_partial_stretch() {
        let (mut gui, input, mut r) = init_test();

        gui.begin_gui(300.0, 200.0);
        gui.begin_stack_container();
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);
        gui.set_children_alignment(AlignHorizontal::Center, AlignVertical::Center);

        gui.rect();
        gui.set_fixed_width(30.0);
        gui.set_vertical_alignment(AlignVertical::Stretch);

        gui.rect();
        gui.set_fixed_height(30.0);
        gui.set_horizontal_alignment(AlignHorizontal::Stretch);

        gui.end_container();
        gui.end_gui(&mut r, &input);

        let root_widget_rf = gui.root();
        let root_widget = root_widget_rf.as_ref();

        check_widget(
            &root_widget,
            &WidgetCheck(
                "Root",
                (0.0, 0.0),
                (300.0, 200.0), // Root widget should always keep it's position and size
                (300.0, 200.0),
                Some(vec![WidgetCheck(
                    "Stack",
                    (0.0, 0.0),
                    (300.0, 200.0), // Stack container expanded so has the same position and size as root one. Ignoring fixed size
                    (300.0, 200.0),
                    Some(vec![
                        WidgetCheck("Rect1", (135.0, 0.0), (30.0, 200.0), (30.0, 200.0), None), // Stretched vertically
                        WidgetCheck("Rect2", (0.0, 85.0), (300.0, 30.0), (300.0, 30.0), None), // Stretched horizontally
                    ]),
                )]),
            ),
        );
    }

    // Sticking to the sides.
    #[test]
    fn test_hmgui_stack_layout_stick_to_sides() {
        let (mut gui, input, mut r) = init_test();

        gui.begin_gui(300.0, 200.0);
        gui.begin_stack_container();
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);
        gui.set_children_alignment(AlignHorizontal::Center, AlignVertical::Center);

        gui.rect();
        gui.set_fixed_size(30.0, 20.0);
        gui.set_horizontal_alignment(AlignHorizontal::Left);

        gui.rect();
        gui.set_fixed_size(20.0, 30.0);
        gui.set_horizontal_alignment(AlignHorizontal::Right);

        gui.rect();
        gui.set_fixed_size(30.0, 20.0);
        gui.set_vertical_alignment(AlignVertical::Top);

        gui.rect();
        gui.set_fixed_size(20.0, 30.0);
        gui.set_vertical_alignment(AlignVertical::Bottom);

        gui.end_container();
        gui.end_gui(&mut r, &input);

        let root_widget_rf = gui.root();
        let root_widget = root_widget_rf.as_ref();

        check_widget(
            &root_widget,
            &WidgetCheck(
                "Root",
                (0.0, 0.0),
                (300.0, 200.0), // Root widget should always keep it's position and size
                (300.0, 200.0),
                Some(vec![WidgetCheck(
                    "Stack",
                    (0.0, 0.0),
                    (300.0, 200.0), // Stack container expanded so has the same position and size as root one
                    (300.0, 200.0),
                    Some(vec![
                        WidgetCheck("Rect1", (0.0, 90.0), (30.0, 20.0), (30.0, 20.0), None), // Stick to the left
                        WidgetCheck("Rect2", (280.0, 85.0), (20.0, 30.0), (20.0, 30.0), None), // Stick to the right
                        WidgetCheck("Rect3", (135.0, 0.0), (30.0, 20.0), (30.0, 20.0), None), // Stick to the top
                        WidgetCheck("Rect4", (140.0, 170.0), (20.0, 30.0), (20.0, 30.0), None), // Stick to the bottom
                    ]),
                )]),
            ),
        );
    }

    // Oversize parent widget/container.
    #[test]
    fn test_hmgui_stack_layout_oversize() {
        let (mut gui, input, mut r) = init_test();

        gui.begin_gui(300.0, 200.0);
        gui.begin_stack_container();
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);
        gui.set_children_alignment(AlignHorizontal::Center, AlignVertical::Center);

        gui.rect();
        gui.set_fixed_size(500.0, 20.0);

        gui.rect();
        gui.set_fixed_size(20.0, 400.0);

        gui.end_container();
        gui.end_gui(&mut r, &input);

        let root_widget_rf = gui.root();
        let root_widget = root_widget_rf.as_ref();

        check_widget(
            &root_widget,
            &WidgetCheck(
                "Root",
                (0.0, 0.0),
                (300.0, 200.0), // Root widget should always keep it's position and size
                (300.0, 200.0),
                Some(vec![WidgetCheck(
                    "Stack",
                    (0.0, 0.0),
                    (300.0, 200.0), // Stack container expanded so has the same position and size as root one
                    (300.0, 200.0),
                    Some(vec![
                        WidgetCheck("Rect1", (-100.0, 90.0), (500.0, 20.0), (500.0, 20.0), None), // Vertical oversize
                        WidgetCheck("Rect2", (140.0, -100.0), (20.0, 400.0), (20.0, 400.0), None), // Horizontal oversize
                    ]),
                )]),
            ),
        );
    }

    // Vertical layout
    #[test]
    fn test_hmgui_vertical_layout_stretch() {
        let (mut gui, input, mut r) = init_test();

        gui.begin_gui(300.0, 200.0);
        gui.begin_vertical_container();
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);
        gui.set_spacing(0.0);
        gui.set_children_alignment(AlignHorizontal::Center, AlignVertical::Center);

        gui.rect();
        gui.set_fixed_size(30.0, 20.0);

        gui.rect();
        gui.set_fixed_height(20.0);
        gui.set_horizontal_alignment(AlignHorizontal::Stretch);

        gui.rect();
        gui.set_fixed_width(20.0);
        gui.set_vertical_alignment(AlignVertical::Stretch);

        gui.rect();
        gui.set_fixed_size(20.0, 30.0);

        gui.rect();
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);

        gui.end_container();
        gui.end_gui(&mut r, &input);

        let root_widget_rf = gui.root();
        let root_widget = root_widget_rf.as_ref();

        check_widget(
            &root_widget,
            &WidgetCheck(
                "Root",
                (0.0, 0.0),
                (300.0, 200.0), // Root widget should always keep it's position and size
                (300.0, 200.0),
                Some(vec![WidgetCheck(
                    "Vertical",
                    (0.0, 0.0),
                    (300.0, 200.0), // Vertical container expanded so has the same position and size as root one
                    (300.0, 200.0),
                    Some(vec![
                        WidgetCheck("Rect1", (135.0, 0.0), (30.0, 20.0), (30.0, 20.0), None), // Fixed size
                        WidgetCheck("Rect2", (0.0, 20.0), (300.0, 20.0), (300.0, 20.0), None), // Fixed height, stretch horizontal
                        WidgetCheck("Rect3", (140.0, 40.0), (20.0, 65.0), (20.0, 65.0), None), // Fixed width, stretch vertical
                        WidgetCheck("Rect4", (140.0, 105.0), (20.0, 30.0), (20.0, 30.0), None), // Fixed size
                        WidgetCheck("Rect5", (0.0, 135.0), (300.0, 65.0), (300.0, 65.0), None), // Stretch all
                    ]),
                )]),
            ),
        );
    }

    #[test]
    fn test_hmgui_vertical_layout_stretch_secondary_dim() {
        let (mut gui, input, mut r) = init_test();

        gui.begin_gui(300.0, 200.0);
        gui.begin_vertical_container();
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);
        gui.set_spacing(0.0);
        gui.set_children_alignment(AlignHorizontal::Stretch, AlignVertical::Center);

        gui.rect();
        gui.set_fixed_size(30.0, 20.0);

        gui.rect();
        gui.set_fixed_size(20.0, 30.0);

        gui.rect();
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);

        gui.end_container();
        gui.end_gui(&mut r, &input);

        let root_widget_rf = gui.root();
        let root_widget = root_widget_rf.as_ref();

        check_widget(
            &root_widget,
            &WidgetCheck(
                "Root",
                (0.0, 0.0),
                (300.0, 200.0), // Root widget should always keep it's position and size
                (300.0, 200.0),
                Some(vec![WidgetCheck(
                    "Vertical",
                    (0.0, 0.0),
                    (300.0, 200.0), // Vertical container expanded so has the same position and size as root one
                    (300.0, 200.0),
                    Some(vec![
                        WidgetCheck("Rect1", (0.0, 0.0), (300.0, 20.0), (300.0, 20.0), None), // Fixed size
                        WidgetCheck("Rect2", (0.0, 20.0), (300.0, 30.0), (300.0, 30.0), None), // Fixed size
                        WidgetCheck("Rect3", (0.0, 50.0), (300.0, 150.0), (300.0, 150.0), None), // Stretch all
                    ]),
                )]),
            ),
        );
    }

    #[test]
    fn test_hmgui_vertical_layout_dock_children() {
        let (mut gui, input, mut r) = init_test();

        gui.begin_gui(300.0, 200.0);

        // Vertical 1: dock top
        gui.begin_vertical_container();
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);
        gui.set_spacing(0.0);
        gui.set_children_alignment(AlignHorizontal::Center, AlignVertical::Top);

        gui.rect();
        gui.set_fixed_size(30.0, 20.0);

        gui.rect();
        gui.set_fixed_size(20.0, 30.0);

        gui.end_container();

        // Vertical 2: dock bottom
        gui.begin_vertical_container();
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);
        gui.set_spacing(0.0);
        gui.set_children_alignment(AlignHorizontal::Center, AlignVertical::Bottom);

        gui.rect();
        gui.set_fixed_size(30.0, 20.0);

        gui.rect();
        gui.set_fixed_size(20.0, 30.0);

        gui.end_container();

        gui.end_gui(&mut r, &input);

        let root_widget_rf = gui.root();
        let root_widget = root_widget_rf.as_ref();

        check_widget(
            &root_widget,
            &WidgetCheck(
                "Root",
                (0.0, 0.0),
                (300.0, 200.0), // Root widget should always keep it's position and size
                (300.0, 200.0),
                Some(vec![
                    WidgetCheck(
                        "Vertical1",
                        (0.0, 0.0),
                        (300.0, 200.0), // Vertical container expanded so has the same position and size as root one
                        (300.0, 200.0),
                        Some(vec![
                            WidgetCheck("Rect1", (135.0, 0.0), (30.0, 20.0), (30.0, 20.0), None),
                            WidgetCheck("Rect2", (140.0, 20.0), (20.0, 30.0), (20.0, 30.0), None),
                        ]),
                    ),
                    WidgetCheck(
                        "Vertical2",
                        (0.0, 0.0),
                        (300.0, 200.0), // Vertical container expanded so has the same position and size as root one
                        (300.0, 200.0),
                        Some(vec![
                            WidgetCheck("Rect1", (135.0, 150.0), (30.0, 20.0), (30.0, 20.0), None),
                            WidgetCheck("Rect2", (140.0, 170.0), (20.0, 30.0), (20.0, 30.0), None),
                        ]),
                    ),
                ]),
            ),
        );
    }

    // Horizontal layout
    #[test]
    fn test_hmgui_horizontal_layout_stretch() {
        let (mut gui, input, mut r) = init_test();

        gui.begin_gui(300.0, 200.0);
        gui.begin_horizontal_container();
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);
        gui.set_spacing(0.0);
        gui.set_children_vertical_alignment(AlignVertical::Center);

        gui.rect();
        gui.set_fixed_size(30.0, 20.0);

        gui.rect();
        gui.set_fixed_width(20.0);
        gui.set_vertical_alignment(AlignVertical::Stretch);

        gui.rect();
        gui.set_fixed_height(20.0);
        gui.set_horizontal_alignment(AlignHorizontal::Stretch);

        gui.rect();
        gui.set_fixed_size(20.0, 30.0);

        gui.rect();
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);

        gui.end_container();
        gui.end_gui(&mut r, &input);

        let root_widget_rf = gui.root();
        let root_widget = root_widget_rf.as_ref();

        check_widget(
            &root_widget,
            &WidgetCheck(
                "Root",
                (0.0, 0.0),
                (300.0, 200.0), // Root widget should always keep it's position and size
                (300.0, 200.0),
                Some(vec![WidgetCheck(
                    "Horizontal",
                    (0.0, 0.0),
                    (300.0, 200.0), // Horizontal container expanded so has the same position and size as root one
                    (300.0, 200.0),
                    Some(vec![
                        WidgetCheck("Rect1", (0.0, 90.0), (30.0, 20.0), (30.0, 20.0), None), // Fixed size
                        WidgetCheck("Rect2", (30.0, 0.0), (20.0, 200.0), (20.0, 200.0), None), // Fixed width, stretch vertical
                        WidgetCheck("Rect3", (50.0, 90.0), (115.0, 20.0), (115.0, 20.0), None), // Fixed height, stretch horizontal
                        WidgetCheck("Rect4", (165.0, 85.0), (20.0, 30.0), (20.0, 30.0), None), // Fixed size
                        WidgetCheck("Rect5", (185.0, 0.0), (115.0, 200.0), (115.0, 200.0), None), // Stretch all
                    ]),
                )]),
            ),
        );
    }

    #[test]
    fn test_hmgui_horizontal_layout_stretch_secondary_dim() {
        let (mut gui, input, mut r) = init_test();

        gui.begin_gui(300.0, 200.0);
        gui.begin_horizontal_container();
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);
        gui.set_spacing(0.0);
        gui.set_children_alignment(AlignHorizontal::Center, AlignVertical::Stretch);

        gui.rect();
        gui.set_fixed_size(30.0, 20.0);

        gui.rect();
        gui.set_fixed_size(20.0, 30.0);

        gui.rect();
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);

        gui.end_container();
        gui.end_gui(&mut r, &input);

        let root_widget_rf = gui.root();
        let root_widget = root_widget_rf.as_ref();

        check_widget(
            &root_widget,
            &WidgetCheck(
                "Root",
                (0.0, 0.0),
                (300.0, 200.0), // Root widget should always keep it's position and size
                (300.0, 200.0),
                Some(vec![WidgetCheck(
                    "Horizontal",
                    (0.0, 0.0),
                    (300.0, 200.0), // Stack container expanded so has the same position and size as root one
                    (300.0, 200.0),
                    Some(vec![
                        WidgetCheck("Rect1", (0.0, 0.0), (30.0, 200.0), (30.0, 200.0), None), // Fixed size
                        WidgetCheck("Rect2", (30.0, 0.0), (20.0, 200.0), (20.0, 200.0), None), // Fixed size
                        WidgetCheck("Rect3", (50.0, 0.0), (250.0, 200.0), (250.0, 200.0), None), // Stretch all
                    ]),
                )]),
            ),
        );
    }

    #[test]
    fn test_hmgui_horizontal_layout_dock_children() {
        let (mut gui, input, mut r) = init_test();

        gui.begin_gui(300.0, 200.0);

        // Horizontal 1: dock left
        gui.begin_horizontal_container();
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);
        gui.set_spacing(0.0);
        gui.set_children_alignment(AlignHorizontal::Left, AlignVertical::Center);

        gui.rect();
        gui.set_fixed_size(30.0, 20.0);

        gui.rect();
        gui.set_fixed_size(20.0, 30.0);

        gui.end_container();

        // Horizontal 2: dock right
        gui.begin_horizontal_container();
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);
        gui.set_spacing(0.0);
        gui.set_children_alignment(AlignHorizontal::Right, AlignVertical::Center);

        gui.rect();
        gui.set_fixed_size(30.0, 20.0);

        gui.rect();
        gui.set_fixed_size(20.0, 30.0);

        gui.end_container();

        gui.end_gui(&mut r, &input);

        let root_widget_rf = gui.root();
        let root_widget = root_widget_rf.as_ref();

        check_widget(
            &root_widget,
            &WidgetCheck(
                "Root",
                (0.0, 0.0),
                (300.0, 200.0), // Root widget should always keep it's position and size
                (300.0, 200.0),
                Some(vec![
                    WidgetCheck(
                        "Horizontal1",
                        (0.0, 0.0),
                        (300.0, 200.0), // Horizontal container expanded so has the same position and size as root one
                        (300.0, 200.0),
                        Some(vec![
                            WidgetCheck("Rect1", (0.0, 90.0), (30.0, 20.0), (30.0, 20.0), None),
                            WidgetCheck("Rect2", (30.0, 85.0), (20.0, 30.0), (20.0, 30.0), None),
                        ]),
                    ),
                    WidgetCheck(
                        "Horizontal2",
                        (0.0, 0.0),
                        (300.0, 200.0), // Horizontal container expanded so has the same position and size as root one
                        (300.0, 200.0),
                        Some(vec![
                            WidgetCheck("Rect1", (250.0, 90.0), (30.0, 20.0), (30.0, 20.0), None),
                            WidgetCheck("Rect2", (280.0, 85.0), (20.0, 30.0), (20.0, 30.0), None),
                        ]),
                    ),
                ]),
            ),
        );
    }

    #[test]
    fn test_hmgui_evenly_stretching() {
        let (mut gui, input, mut r) = init_test();

        gui.begin_gui(300.0, 200.0);

        // Horizontal
        gui.begin_horizontal_container();
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);
        gui.set_spacing(0.0);
        gui.set_children_alignment(AlignHorizontal::Stretch, AlignVertical::Center);

        gui.rect();
        gui.set_fixed_height(20.0);

        gui.rect();
        gui.set_fixed_height(30.0);

        gui.rect();
        gui.set_fixed_height(40.0);

        gui.end_container();

        // Vertical
        gui.begin_vertical_container();
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);
        gui.set_spacing(0.0);
        gui.set_children_alignment(AlignHorizontal::Center, AlignVertical::Stretch);

        gui.rect();
        gui.set_fixed_width(20.0);

        gui.rect();
        gui.set_fixed_width(30.0);

        gui.rect();
        gui.set_fixed_width(40.0);

        gui.rect();
        gui.set_fixed_width(50.0);

        gui.end_container();

        gui.end_gui(&mut r, &input);

        let root_widget_rf = gui.root();
        let root_widget = root_widget_rf.as_ref();

        check_widget(
            &root_widget,
            &WidgetCheck(
                "Root",
                (0.0, 0.0),
                (300.0, 200.0), // Root widget should always keep it's position and size
                (300.0, 200.0),
                Some(vec![
                    WidgetCheck(
                        "Horizontal",
                        (0.0, 0.0),
                        (300.0, 200.0), // Horizontal container expanded so has the same position and size as root one
                        (300.0, 200.0),
                        Some(vec![
                            WidgetCheck("Rect1", (0.0, 90.0), (100.0, 20.0), (100.0, 20.0), None),
                            WidgetCheck("Rect2", (100.0, 85.0), (100.0, 30.0), (100.0, 30.0), None),
                            WidgetCheck("Rect3", (200.0, 80.0), (100.0, 40.0), (100.0, 40.0), None),
                        ]),
                    ),
                    WidgetCheck(
                        "Vertical",
                        (0.0, 0.0),
                        (300.0, 200.0), // Vertical container expanded so has the same position and size as root one
                        (300.0, 200.0),
                        Some(vec![
                            WidgetCheck("Rect1", (140.0, 0.0), (20.0, 50.0), (20.0, 50.0), None),
                            WidgetCheck("Rect2", (135.0, 50.0), (30.0, 50.0), (30.0, 50.0), None),
                            WidgetCheck("Rect3", (130.0, 100.0), (40.0, 50.0), (40.0, 50.0), None),
                            WidgetCheck("Rect4", (125.0, 150.0), (50.0, 50.0), (50.0, 50.0), None),
                        ]),
                    ),
                ]),
            ),
        );
    }

    // Vertical: Margin, border, padding, spacing.
    #[test]
    fn test_hmgui_vertical_layout_margin_border_padding_spacing() {
        let (mut gui, input, mut r) = init_test();

        gui.begin_gui(300.0, 200.0);

        gui.begin_vertical_container();
        gui.set_margin(5.0, 5.0);
        gui.set_border_width(5.0);
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);
        gui.set_padding(20.0, 20.0);
        gui.set_spacing(10.0);
        gui.set_children_horizontal_alignment(AlignHorizontal::Center);

        gui.rect();
        gui.set_fixed_size(30.0, 20.0);

        gui.rect();
        gui.set_fixed_size(20.0, 30.0);
        gui.set_margin(13.0, 13.0);
        gui.set_border_width(2.0);

        gui.rect();
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);

        gui.end_container();

        gui.end_gui(&mut r, &input);

        let root_widget_rf = gui.root();
        let root_widget = root_widget_rf.as_ref();

        check_widget(
            &root_widget,
            &WidgetCheck(
                "Root",
                (0.0, 0.0),
                (300.0, 200.0), // Root widget should always keep it's position and size
                (300.0, 200.0),
                Some(vec![WidgetCheck(
                    "Vertical",
                    (0.0, 0.0),
                    (300.0, 200.0), // Vertical container expanded so has the same position and size as root one
                    (280.0, 180.0),
                    Some(vec![
                        WidgetCheck("Rect1", (135.0, 30.0), (30.0, 20.0), (30.0, 20.0), None), // Fixed size
                        WidgetCheck("Rect2", (125.0, 60.0), (50.0, 60.0), (20.0, 30.0), None), // Fixed size
                        WidgetCheck("Rect3", (30.0, 130.0), (240.0, 40.0), (240.0, 40.0), None), // Stretch all
                    ]),
                )]),
            ),
        );
    }

    // Horizontal: Margin, border, padding, spacing.
    #[test]
    fn test_hmgui_horizontal_layout_margin_border_padding_spacing() {
        let (mut gui, input, mut r) = init_test();

        gui.begin_gui(300.0, 200.0);

        gui.begin_horizontal_container();
        gui.set_margin(5.0, 5.0);
        gui.set_border_width(5.0);
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);
        gui.set_padding(20.0, 20.0);
        gui.set_spacing(10.0);
        gui.set_children_vertical_alignment(AlignVertical::Center);

        gui.rect();
        gui.set_fixed_size(30.0, 20.0);

        gui.rect();
        gui.set_fixed_size(20.0, 30.0);
        gui.set_margin(13.0, 13.0);
        gui.set_border_width(2.0);

        gui.rect();
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);

        gui.end_container();

        gui.end_gui(&mut r, &input);

        let root_widget_rf = gui.root();
        let root_widget = root_widget_rf.as_ref();

        check_widget(
            &root_widget,
            &WidgetCheck(
                "Root",
                (0.0, 0.0),
                (300.0, 200.0), // Root widget should always keep it's position and size
                (300.0, 200.0),
                Some(vec![WidgetCheck(
                    "Horizontal",
                    (0.0, 0.0),
                    (300.0, 200.0), // Horizontal container expanded so has the same position and size as root one
                    (280.0, 180.0),
                    Some(vec![
                        WidgetCheck("Rect1", (30.0, 90.0), (30.0, 20.0), (30.0, 20.0), None), // Fixed size
                        WidgetCheck("Rect2", (70.0, 70.0), (50.0, 60.0), (20.0, 30.0), None), // Fixed size
                        WidgetCheck("Rect3", (130.0, 30.0), (140.0, 140.0), (140.0, 140.0), None), // Stretch all
                    ]),
                )]),
            ),
        );
    }

    // One-sided margins: the left/top margin moves the widget's inner rect,
    // the right/bottom margin only adds space after it.
    #[test]
    fn test_hmgui_one_sided_margins() {
        let (mut gui, input, mut r) = init_test();

        gui.begin_gui(300.0, 200.0);

        gui.begin_vertical_container();
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);
        gui.set_children_horizontal_alignment(AlignHorizontal::Left);

        gui.rect();
        gui.set_fixed_size(30.0, 20.0);
        gui.set_margin_ex(4.0, 6.0, 0.0, 0.0); // left and top only

        gui.rect();
        gui.set_fixed_size(30.0, 20.0);
        gui.set_margin_ex(0.0, 0.0, 4.0, 6.0); // right and bottom only

        gui.rect();
        gui.set_fixed_size(30.0, 20.0);
        gui.set_margin_left(3.0);
        gui.set_margin_top(5.0);
        gui.set_margin_right(7.0);
        gui.set_margin_bottom(9.0);
        gui.set_border_width(2.0);

        gui.end_container();

        gui.end_gui(&mut r, &input);

        let root_widget_rf = gui.root();
        let root_widget = root_widget_rf.as_ref();

        check_widget(
            &root_widget,
            &WidgetCheck(
                "Root",
                (0.0, 0.0),
                (300.0, 200.0), // Root widget should always keep it's position and size
                (300.0, 200.0),
                Some(vec![WidgetCheck(
                    "Vertical",
                    (0.0, 0.0),
                    (300.0, 200.0), // Vertical container expanded so has the same position and size as root one
                    (300.0, 200.0),
                    Some(vec![
                        WidgetCheck("Rect1", (0.0, 0.0), (34.0, 26.0), (30.0, 20.0), None),
                        WidgetCheck("Rect2", (0.0, 26.0), (34.0, 26.0), (30.0, 20.0), None),
                        WidgetCheck("Rect3", (0.0, 52.0), (44.0, 38.0), (30.0, 20.0), None),
                    ]),
                )]),
            ),
        );

        // check_widget compares outer positions; which side a margin is on shows in inner_pos.
        let vertical_rf = root_widget.get_container_item().children[0].clone();
        let vertical = vertical_rf.as_ref();
        let inner_pos: Vec<Vec2> = vertical
            .get_container_item()
            .children
            .iter()
            .map(|child| child.as_ref().inner_pos)
            .collect();

        assert_eq!(
            inner_pos,
            vec![
                Vec2::new(4.0, 6.0),  // Moved by its left/top margin
                Vec2::new(0.0, 26.0), // A right/bottom margin doesn't move it
                Vec2::new(5.0, 59.0), // Left/top margin plus border
            ],
            "Inner rects must be offset by the left/top margin"
        );
    }

    #[test]
    fn test_hmgui_stack_percentage_size() {
        let (mut gui, input, mut r) = init_test();

        gui.begin_gui(300.0, 200.0);

        // Stack
        gui.begin_stack_container();
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);
        gui.set_children_alignment(AlignHorizontal::Center, AlignVertical::Center);

        // Inside container

        gui.rect();
        gui.set_percent_width(70.0);
        gui.set_fixed_height(20.0);

        gui.rect();
        gui.set_fixed_width(30.0);
        gui.set_percent_height(60.0);

        // Outside container

        gui.rect();
        gui.set_percent_width(120.0);
        gui.set_fixed_height(20.0);

        gui.rect();
        gui.set_fixed_width(30.0);
        gui.set_percent_height(130.0);

        gui.end_container();

        gui.end_gui(&mut r, &input);

        let root_widget_rf = gui.root();
        let root_widget = root_widget_rf.as_ref();

        check_widget(
            &root_widget,
            &WidgetCheck(
                "Root",
                (0.0, 0.0),
                (300.0, 200.0), // Root widget should always keep it's position and size
                (300.0, 200.0),
                Some(vec![WidgetCheck(
                    "Stack",
                    (0.0, 0.0),
                    (300.0, 200.0), // Stack container expanded so has the same position and size as root one
                    (300.0, 200.0),
                    Some(vec![
                        WidgetCheck("Rect1", (45.0, 90.0), (210.0, 20.0), (210.0, 20.0), None),
                        WidgetCheck("Rect2", (135.0, 40.0), (30.0, 120.0), (30.0, 120.0), None),
                        WidgetCheck("Rect3", (-30.0, 90.0), (360.0, 20.0), (360.0, 20.0), None),
                        WidgetCheck("Rect4", (135.0, -30.0), (30.0, 260.0), (30.0, 260.0), None),
                    ]),
                )]),
            ),
        );
    }

    #[test]
    fn test_hmgui_horizontal_percentage_size() {
        let (mut gui, input, mut r) = init_test();

        gui.begin_gui(300.0, 200.0);

        // Horizontal: first widget - fixed size - 20, second - 50% = 140, last - expands to non remaining width - 140
        gui.begin_horizontal_container();
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);
        gui.set_spacing(0.0);
        gui.set_children_alignment(AlignHorizontal::Center, AlignVertical::Center);

        gui.rect();
        gui.set_fixed_size(20.0, 30.0);

        gui.rect();
        gui.set_percent_width(50.0);
        gui.set_min_width(0.0);
        gui.set_fixed_height(30.0);

        gui.rect();
        gui.set_fixed_height(40.0);
        gui.set_min_width(0.0);
        gui.set_horizontal_alignment(AlignHorizontal::Stretch);

        gui.end_container();

        // Horizontal: first widget - fixed size - 20, second - 150% = 420, last widget shrinks to 0 width
        gui.begin_horizontal_container();
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);
        gui.set_spacing(0.0);
        gui.set_children_alignment(AlignHorizontal::Center, AlignVertical::Center);

        gui.rect();
        gui.set_fixed_size(20.0, 30.0);

        gui.rect();
        gui.set_percent_width(150.0);
        gui.set_min_width(0.0);
        gui.set_fixed_height(30.0);

        gui.rect();
        gui.set_fixed_height(40.0);
        gui.set_min_width(0.0);
        gui.set_horizontal_alignment(AlignHorizontal::Stretch);

        gui.end_container();

        gui.end_gui(&mut r, &input);

        let root_widget_rf = gui.root();
        let root_widget = root_widget_rf.as_ref();

        check_widget(
            &root_widget,
            &WidgetCheck(
                "Root",
                (0.0, 0.0),
                (300.0, 200.0), // Root widget should always keep it's position and size
                (300.0, 200.0),
                Some(vec![
                    WidgetCheck(
                        "Horizontal1",
                        (0.0, 0.0),
                        (300.0, 200.0), // Horizontal container expanded so has the same position and size as root one
                        (300.0, 200.0),
                        Some(vec![
                            WidgetCheck("Rect1", (0.0, 85.0), (20.0, 30.0), (20.0, 30.0), None),
                            WidgetCheck("Rect2", (20.0, 85.0), (150.0, 30.0), (150.0, 30.0), None),
                            WidgetCheck("Rect3", (170.0, 80.0), (130.0, 40.0), (130.0, 40.0), None),
                        ]),
                    ),
                    WidgetCheck(
                        "Horizontal2",
                        (0.0, 0.0),
                        (300.0, 200.0), // Horizontal container expanded so has the same position and size as root one
                        (300.0, 200.0),
                        Some(vec![
                            WidgetCheck("Rect4", (-85.0, 85.0), (20.0, 30.0), (20.0, 30.0), None),
                            WidgetCheck("Rect5", (-65.0, 85.0), (450.0, 30.0), (450.0, 30.0), None),
                            WidgetCheck("Rect6", (385.0, 80.0), (0.0, 40.0), (0.0, 40.0), None),
                        ]),
                    ),
                ]),
            ),
        );
    }

    #[test]
    fn test_hmgui_vertical_percentage_size() {
        let (mut gui, input, mut r) = init_test();

        gui.begin_gui(300.0, 200.0);

        // Vertical: first widget - fixed size - 30, second - 50% = 85, last - expands to non remaining width - 115
        gui.begin_vertical_container();
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);
        gui.set_spacing(0.0);
        gui.set_children_alignment(AlignHorizontal::Center, AlignVertical::Center);

        gui.rect();
        gui.set_fixed_size(20.0, 30.0);

        gui.rect();
        gui.set_fixed_width(30.0);
        gui.set_min_height(0.0);
        gui.set_percent_height(50.0);

        gui.rect();
        gui.set_fixed_width(40.0);
        gui.set_min_height(0.0);
        gui.set_vertical_alignment(AlignVertical::Stretch);

        gui.end_container();

        // Vertical: first widget - fixed size - 30, second - 120% = 204, last widget shrinks to 0 width
        gui.begin_vertical_container();
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);
        gui.set_spacing(0.0);
        gui.set_children_alignment(AlignHorizontal::Center, AlignVertical::Center);

        gui.rect();
        gui.set_fixed_size(20.0, 30.0);

        gui.rect();
        gui.set_fixed_width(30.0);
        gui.set_min_height(0.0);
        gui.set_percent_height(120.0);

        gui.rect();
        gui.set_fixed_width(40.0);
        gui.set_min_height(0.0);
        gui.set_vertical_alignment(AlignVertical::Stretch);

        gui.end_container();

        gui.end_gui(&mut r, &input);

        let root_widget_rf = gui.root();
        let root_widget = root_widget_rf.as_ref();

        check_widget(
            &root_widget,
            &WidgetCheck(
                "Root",
                (0.0, 0.0),
                (300.0, 200.0), // Root widget should always keep it's position and size
                (300.0, 200.0),
                Some(vec![
                    WidgetCheck(
                        "Horizontal1",
                        (0.0, 0.0),
                        (300.0, 200.0), // Horizontal container expanded so has the same position and size as root one
                        (300.0, 200.0),
                        Some(vec![
                            WidgetCheck("Rect1", (140.0, 0.0), (20.0, 30.0), (20.0, 30.0), None),
                            WidgetCheck("Rect2", (135.0, 30.0), (30.0, 100.0), (30.0, 100.0), None),
                            WidgetCheck("Rect3", (130.0, 130.0), (40.0, 70.0), (40.0, 70.0), None),
                        ]),
                    ),
                    WidgetCheck(
                        "Horizontal2",
                        (0.0, 0.0),
                        (300.0, 200.0), // Horizontal container expanded so has the same position and size as root one
                        (300.0, 200.0),
                        Some(vec![
                            WidgetCheck("Rect1", (140.0, -35.0), (20.0, 30.0), (20.0, 30.0), None),
                            WidgetCheck("Rect2", (135.0, -5.0), (30.0, 240.0), (30.0, 240.0), None),
                            WidgetCheck("Rect3", (130.0, 235.0), (40.0, 0.0), (40.0, 0.0), None),
                        ]),
                    ),
                ]),
            ),
        );
    }

    #[test]
    fn test_hmgui_alignment_combination() {
        let (mut gui, input, mut r) = init_test();

        gui.begin_gui(300.0, 200.0);

        gui.begin_horizontal_container();
        gui.set_horizontal_alignment(AlignHorizontal::Stretch);
        gui.set_spacing(0.0);
        gui.set_children_vertical_alignment(AlignVertical::Stretch);

        gui.begin_vertical_container();
        gui.set_children_alignment(AlignHorizontal::Center, AlignVertical::Center);
        gui.set_percent_width(10.0);

        gui.rect();
        gui.set_fixed_size(20.0, 30.0);
        gui.end_container();

        gui.begin_horizontal_container();
        gui.set_horizontal_alignment(AlignHorizontal::Stretch);
        gui.set_children_horizontal_alignment(AlignHorizontal::Center);

        gui.rect();
        gui.set_fixed_size(20.0, 30.0);
        gui.set_vertical_alignment(AlignVertical::Center);
        gui.end_container();

        gui.begin_stack_container();
        gui.set_percent_width(10.0);

        gui.rect();
        gui.set_fixed_size(20.0, 30.0);
        gui.set_alignment(AlignHorizontal::Center, AlignVertical::Center);
        gui.end_container();

        gui.end_container();

        gui.end_gui(&mut r, &input);

        let root_widget_rf = gui.root();
        let root_widget = root_widget_rf.as_ref();

        check_widget(
            &root_widget,
            &WidgetCheck(
                "Root",
                (0.0, 0.0),
                (300.0, 200.0), // Root widget should always keep it's position and size
                (300.0, 200.0),
                Some(vec![WidgetCheck(
                    "Horizontal",
                    (0.0, 0.0),
                    (300.0, 30.0), // Horizontal container expanded so has the same position and size as root one
                    (300.0, 30.0),
                    Some(vec![
                        WidgetCheck(
                            "Horizontal1",
                            (0.0, 0.0),
                            (30.0, 30.0),
                            (30.0, 30.0),
                            Some(vec![WidgetCheck(
                                "Rect1",
                                (5.0, 0.0),
                                (20.0, 30.0),
                                (20.0, 30.0),
                                None,
                            )]),
                        ),
                        WidgetCheck(
                            "Horizontal2",
                            (30.0, 0.0),
                            (240.0, 30.0),
                            (240.0, 30.0),
                            Some(vec![WidgetCheck(
                                "Rect2",
                                (140.0, 0.0),
                                (20.0, 30.0),
                                (20.0, 30.0),
                                None,
                            )]),
                        ),
                        WidgetCheck(
                            "Horizontal3",
                            (270.0, 0.0),
                            (30.0, 30.0),
                            (30.0, 30.0),
                            Some(vec![WidgetCheck(
                                "Rect3",
                                (275.0, 0.0),
                                (20.0, 30.0),
                                (20.0, 30.0),
                                None,
                            )]),
                        ),
                    ]),
                )]),
            ),
        );
    }

    #[test]
    fn test_hmgui_shape_widgets() {
        let (mut gui, input, mut r) = init_test();

        gui.begin_gui(300.0, 200.0);
        gui.begin_stack_container();
        gui.set_alignment(AlignHorizontal::Center, AlignVertical::Center);
        gui.set_children_alignment(AlignHorizontal::Center, AlignVertical::Center);

        // Shapes lay out like rects: they take the size the layout gives them.
        gui.annulus(&crate::render::Color::WHITE, 10.0, false);
        gui.set_fixed_size(120.0, 80.0);

        gui.wedge(&crate::render::Color::WHITE, 4.0, 270.0, 45.0, true);
        gui.set_fixed_size(60.0, 60.0);

        gui.end_container();
        gui.end_gui(&mut r, &input);

        let root_widget_rf = gui.root();
        let root_widget = root_widget_rf.as_ref();

        check_widget(
            &root_widget,
            &WidgetCheck(
                "Root",
                (0.0, 0.0),
                (300.0, 200.0),
                (300.0, 200.0),
                Some(vec![WidgetCheck(
                    "Stack",
                    (90.0, 60.0),
                    (120.0, 80.0),
                    (120.0, 80.0),
                    Some(vec![
                        WidgetCheck("Annulus", (90.0, 60.0), (120.0, 80.0), (120.0, 80.0), None),
                        WidgetCheck("Wedge", (120.0, 70.0), (60.0, 60.0), (60.0, 60.0), None),
                    ]),
                )]),
            ),
        );

        // Each shape is centred in its widget, so both share the stack's
        // centre (150, 100), whatever their widget sizes.
        let stack_rf = root_widget.get_container_item().children[0].clone();
        let stack = stack_rf.as_ref();
        // Both are drawn as strokes on a circle: (centre, radius, thickness).
        let looks = HmGuiLooks::new();
        let circles: Vec<_> = stack
            .get_container_item()
            .children
            .iter()
            .map(|widget_rf| {
                let widget = widget_rf.as_ref();
                let WidgetItem::Shape(shape) = &widget.item else {
                    panic!("Expected a shape widget");
                };
                match shape.geometry(widget.inner_pos, widget.inner_size, &looks) {
                    Some((
                        crate::render::Shape::Stroke,
                        crate::render::UIShapeGeometry::Stroke { stroke, .. },
                    )) => (
                        stroke.start + stroke.normal() / stroke.curvature,
                        1.0 / stroke.curvature.abs(),
                        stroke.thickness,
                    ),
                    other => panic!("Expected a stroke, got {other:?}"),
                }
            })
            .collect();

        let close = |a: Vec2, b: Vec2| a.distance(b) < 1e-3;
        // Radius 40 (half the widget's smaller side), 10 px thick: the
        // stroke runs along its middle, 35.
        assert!(close(circles[0].0, Vec2::new(150.0, 100.0)), "{circles:?}");
        assert!((circles[0].1 - 35.0).abs() < 1e-3, "{circles:?}");
        // Radius 30, 4 px deep.
        assert!(close(circles[1].0, Vec2::new(150.0, 100.0)), "{circles:?}");
        assert!((circles[1].1 - 28.0).abs() < 1e-3, "{circles:?}");
        assert_eq!(circles[1].2, 4.0);
    }

    // A fixed or percent size is kept when a child is bigger: the child
    // overflows (and is clipped), the container doesn't grow around it.
    #[test]
    fn test_hmgui_fixed_and_percent_size_containers_keep_their_size() {
        let (mut gui, input, mut r) = init_test();

        gui.begin_gui(300.0, 200.0);

        gui.begin_stack_container();
        gui.set_alignment(AlignHorizontal::Left, AlignVertical::Top);
        gui.set_fixed_size(100.0, 50.0);
        gui.rect();
        gui.set_alignment(AlignHorizontal::Left, AlignVertical::Top);
        gui.set_fixed_size(250.0, 20.0);
        gui.end_container();

        gui.begin_vertical_container();
        gui.set_alignment(AlignHorizontal::Right, AlignVertical::Top);
        gui.set_percent_size(20.0, 25.0); // 60 x 50
        gui.rect();
        gui.set_fixed_size(40.0, 80.0);
        gui.end_container();

        gui.end_gui(&mut r, &input);

        let root_widget_rf = gui.root();
        let root_widget = root_widget_rf.as_ref();

        check_widget(
            &root_widget,
            &WidgetCheck(
                "Root",
                (0.0, 0.0),
                (300.0, 200.0),
                (300.0, 200.0),
                Some(vec![
                    WidgetCheck(
                        "FixedStack",
                        (0.0, 0.0),
                        (100.0, 50.0), // not 250 wide
                        (100.0, 50.0),
                        Some(vec![WidgetCheck(
                            "WideRect",
                            (0.0, 0.0),
                            (250.0, 20.0),
                            (250.0, 20.0),
                            None,
                        )]),
                    ),
                    WidgetCheck(
                        "PercentVertical",
                        (240.0, 0.0),
                        (60.0, 50.0), // not 80 high
                        (60.0, 50.0),
                        Some(vec![WidgetCheck(
                            "TallRect",
                            (240.0, 0.0),
                            (40.0, 80.0),
                            (40.0, 80.0),
                            None,
                        )]),
                    ),
                ]),
            ),
        );
    }

    // set_min_width/size work on containers: at least that big, bigger if
    // the content needs it.
    #[test]
    fn test_hmgui_min_size_on_containers() {
        let (mut gui, input, mut r) = init_test();

        gui.begin_gui(300.0, 200.0);

        gui.begin_vertical_container();
        gui.set_alignment(AlignHorizontal::Left, AlignVertical::Top);
        gui.set_min_width(120.0);
        gui.rect();
        gui.set_fixed_size(40.0, 10.0);
        gui.end_container();

        gui.begin_vertical_container();
        gui.set_alignment(AlignHorizontal::Right, AlignVertical::Top);
        gui.set_min_size(30.0, 5.0);
        gui.rect();
        gui.set_fixed_size(80.0, 10.0);
        gui.end_container();

        gui.end_gui(&mut r, &input);

        let root_widget_rf = gui.root();
        let root_widget = root_widget_rf.as_ref();

        check_widget(
            &root_widget,
            &WidgetCheck(
                "Root",
                (0.0, 0.0),
                (300.0, 200.0),
                (300.0, 200.0),
                Some(vec![
                    WidgetCheck(
                        "MinWidth",
                        (0.0, 0.0),
                        (120.0, 10.0), // wider than its content
                        (120.0, 10.0),
                        Some(vec![WidgetCheck(
                            "Rect",
                            (0.0, 0.0),
                            (40.0, 10.0),
                            (40.0, 10.0),
                            None,
                        )]),
                    ),
                    WidgetCheck(
                        "ContentWins",
                        (220.0, 0.0),
                        (80.0, 10.0), // the content is bigger than the min size
                        (80.0, 10.0),
                        Some(vec![WidgetCheck(
                            "Rect",
                            (220.0, 0.0),
                            (80.0, 10.0),
                            (80.0, 10.0),
                            None,
                        )]),
                    ),
                ]),
            ),
        );
    }

    // set_position: placed from the parent's top left inside its border
    // (padding doesn't apply), out of the layout.
    #[test]
    fn test_hmgui_positioned_widgets() {
        let (mut gui, input, mut r) = init_test();

        gui.begin_gui(300.0, 200.0);

        gui.begin_vertical_container();
        gui.set_alignment(AlignHorizontal::Left, AlignVertical::Top);
        gui.set_padding(10.0, 10.0);
        gui.set_border_width(2.0);

        gui.rect();
        gui.set_fixed_size(50.0, 20.0);

        gui.rect();
        gui.set_fixed_size(30.0, 30.0);
        gui.set_position(100.0, 40.0);

        gui.rect();
        gui.set_fixed_size(50.0, 20.0);

        gui.end_container();

        gui.begin_stack_container();
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);
        gui.rect();
        gui.set_percent_size(50.0, 10.0); // of the area it's placed in
        gui.set_position(150.0, 0.0);
        gui.end_container();

        gui.end_gui(&mut r, &input);

        let root_widget_rf = gui.root();
        let root_widget = root_widget_rf.as_ref();

        check_widget(
            &root_widget,
            &WidgetCheck(
                "Root",
                (0.0, 0.0),
                (300.0, 200.0),
                (300.0, 200.0),
                Some(vec![
                    WidgetCheck(
                        "Vertical",
                        (0.0, 0.0),
                        (74.0, 64.0), // sized by the two flowing rects only
                        (70.0, 60.0),
                        Some(vec![
                            WidgetCheck("Flow1", (12.0, 12.0), (50.0, 20.0), (50.0, 20.0), None),
                            WidgetCheck(
                                "Positioned",
                                (102.0, 42.0), // border (2) + position, no padding
                                (30.0, 30.0),
                                (30.0, 30.0),
                                None,
                            ),
                            WidgetCheck("Flow2", (12.0, 32.0), (50.0, 20.0), (50.0, 20.0), None), // right after Flow1
                        ]),
                    ),
                    WidgetCheck(
                        "Stack",
                        (0.0, 0.0),
                        (300.0, 200.0),
                        (300.0, 200.0),
                        Some(vec![WidgetCheck(
                            "PositionedPercent",
                            (150.0, 0.0),
                            (150.0, 20.0),
                            (150.0, 20.0),
                            None,
                        )]),
                    ),
                ]),
            ),
        );
    }

    // set_max_size caps a stretched and a fixed size.
    #[test]
    fn test_hmgui_max_size() {
        let (mut gui, input, mut r) = init_test();

        gui.begin_gui(300.0, 200.0);

        gui.rect();
        gui.set_alignment(AlignHorizontal::Stretch, AlignVertical::Stretch);
        gui.set_max_size(100.0, 50.0);

        gui.rect();
        gui.set_alignment(AlignHorizontal::Left, AlignVertical::Top);
        gui.set_fixed_size(200.0, 80.0);
        gui.set_max_width(120.0);

        // a capped container doesn't grow around a wider child either
        gui.begin_stack_container();
        gui.set_alignment(AlignHorizontal::Left, AlignVertical::Top);
        gui.set_max_width(80.0);
        gui.rect();
        gui.set_alignment(AlignHorizontal::Left, AlignVertical::Top);
        gui.set_fixed_size(200.0, 10.0);
        gui.end_container();

        gui.end_gui(&mut r, &input);

        let root_widget_rf = gui.root();
        let root_widget = root_widget_rf.as_ref();

        check_widget(
            &root_widget,
            &WidgetCheck(
                "Root",
                (0.0, 0.0),
                (300.0, 200.0),
                (300.0, 200.0),
                Some(vec![
                    WidgetCheck("Stretched", (0.0, 0.0), (100.0, 50.0), (100.0, 50.0), None),
                    WidgetCheck("Fixed", (0.0, 0.0), (120.0, 80.0), (120.0, 80.0), None),
                    WidgetCheck(
                        "CappedStack",
                        (0.0, 0.0),
                        (80.0, 10.0),
                        (80.0, 10.0),
                        Some(vec![WidgetCheck(
                            "Wide",
                            (0.0, 0.0),
                            (200.0, 10.0),
                            (200.0, 10.0),
                            None,
                        )]),
                    ),
                ]),
            ),
        );
    }

    // A layer below a widget starts under its border box: its margins don't count.
    #[test]
    fn test_hmgui_layer_below_starts_under_the_border_box() {
        let (mut gui, input, mut r) = init_test();

        gui.begin_gui(300.0, 200.0);

        gui.rect();
        gui.set_alignment(AlignHorizontal::Left, AlignVertical::Top);
        gui.set_fixed_size(50.0, 30.0);
        gui.set_margin_ex(10.0, 20.0, 5.0, 7.0);
        gui.set_border_width(1.0);

        gui.begin_layer_below();
        gui.rect();
        gui.set_fixed_size(40.0, 10.0);
        gui.end_layer();

        gui.end_gui(&mut r, &input);

        // the rect's border box: (10, 20), 52 x 32
        let layer_root_rf = gui.layer_root(1);
        assert_eq!(layer_root_rf.as_ref().pos, Vec2::new(10.0, 52.0));
    }

    // A container that sizes to its content makes room for its percent-sized
    // children's content: two rows of 50 % are as tall as twice the taller row,
    // so both rows fit (before, the column was 40 high and the second row cut).
    #[test]
    fn test_hmgui_percent_children_size_their_parent() {
        let (mut gui, input, mut r) = init_test();

        gui.begin_gui(300.0, 200.0);

        gui.begin_vertical_container();
        gui.set_alignment(AlignHorizontal::Left, AlignVertical::Top);
        gui.set_spacing(0.0);

        gui.begin_stack_container();
        gui.set_percent_height(50.0);
        gui.rect();
        gui.set_fixed_size(40.0, 10.0);
        gui.end_container();

        gui.begin_stack_container();
        gui.set_percent_height(50.0);
        gui.rect();
        gui.set_fixed_size(20.0, 30.0);
        gui.end_container();

        gui.end_container();

        // Across the layout direction too: 25 % wide, 30 px of content -> 120.
        gui.begin_vertical_container();
        gui.set_alignment(AlignHorizontal::Right, AlignVertical::Top);
        gui.begin_stack_container();
        gui.set_percent_width(25.0);
        gui.rect();
        gui.set_fixed_size(30.0, 10.0);
        gui.end_container();
        gui.end_container();

        gui.end_gui(&mut r, &input);

        let root_widget_rf = gui.root();
        let root_widget = root_widget_rf.as_ref();

        check_widget(
            &root_widget,
            &WidgetCheck(
                "Root",
                (0.0, 0.0),
                (300.0, 200.0),
                (300.0, 200.0),
                Some(vec![
                    WidgetCheck(
                        "Rows",
                        (0.0, 0.0),
                        (40.0, 60.0),
                        (40.0, 60.0),
                        Some(vec![
                            WidgetCheck(
                                "Row1",
                                (0.0, 0.0),
                                (40.0, 30.0),
                                (40.0, 30.0),
                                Some(vec![WidgetCheck(
                                    "Rect1",
                                    (0.0, 0.0),
                                    (40.0, 10.0),
                                    (40.0, 10.0),
                                    None,
                                )]),
                            ),
                            WidgetCheck(
                                "Row2",
                                (0.0, 30.0),
                                (20.0, 30.0),
                                (20.0, 30.0),
                                Some(vec![WidgetCheck(
                                    "Rect2",
                                    (0.0, 30.0),
                                    (20.0, 30.0),
                                    (20.0, 30.0),
                                    None,
                                )]),
                            ),
                        ]),
                    ),
                    WidgetCheck(
                        "Column",
                        (180.0, 0.0),
                        (120.0, 10.0),
                        (120.0, 10.0),
                        Some(vec![WidgetCheck(
                            "Quarter",
                            (180.0, 0.0),
                            (30.0, 10.0),
                            (30.0, 10.0),
                            Some(vec![WidgetCheck(
                                "Rect3",
                                (180.0, 0.0),
                                (30.0, 10.0),
                                (30.0, 10.0),
                                None,
                            )]),
                        )]),
                    ),
                ]),
            ),
        );
    }

    // A percent-sized rect has no content to make room for: a 1 % bar doesn't
    // make its parent bigger than it was (its default 20 px min size still
    // counts, as before).
    #[test]
    fn test_hmgui_percent_rect_does_not_grow_its_parent() {
        let (mut gui, input, mut r) = init_test();

        gui.begin_gui(300.0, 200.0);

        gui.begin_horizontal_container();
        gui.set_alignment(AlignHorizontal::Left, AlignVertical::Top);
        gui.set_spacing(0.0);
        gui.rect();
        gui.set_fixed_size(50.0, 10.0);
        gui.rect();
        gui.set_percent_width(1.0);
        gui.set_fixed_height(10.0);
        gui.end_container();

        gui.end_gui(&mut r, &input);

        let root_widget_rf = gui.root();
        let root_widget = root_widget_rf.as_ref();

        check_widget(
            &root_widget,
            &WidgetCheck(
                "Root",
                (0.0, 0.0),
                (300.0, 200.0),
                (300.0, 200.0),
                Some(vec![WidgetCheck(
                    "Horizontal",
                    (0.0, 0.0),
                    (70.0, 10.0), // 50 + the bar's default 20, not 50 / 1 %
                    (70.0, 10.0),
                    Some(vec![
                        WidgetCheck("Fixed", (0.0, 0.0), (50.0, 10.0), (50.0, 10.0), None),
                        WidgetCheck("Bar", (50.0, 0.0), (0.7, 10.0), (0.7, 10.0), None),
                    ]),
                )]),
            ),
        );
    }

    // The space left along the layout direction comes from the children's
    // resolved sizes, so a percent-sized container (and one wider than its
    // content because of its min width) centres its children. Before, both
    // saw no space left and kept them at the start.
    #[test]
    fn test_hmgui_percent_and_min_size_containers_align_their_children() {
        let (mut gui, input, mut r) = init_test();

        gui.begin_gui(300.0, 200.0);

        gui.begin_horizontal_container();
        gui.set_alignment(AlignHorizontal::Left, AlignVertical::Top);
        gui.set_percent_width(50.0); // 150
        gui.set_children_alignment(AlignHorizontal::Center, AlignVertical::Top);
        gui.rect();
        gui.set_fixed_size(40.0, 10.0);
        gui.end_container();

        gui.begin_horizontal_container();
        gui.set_alignment(AlignHorizontal::Right, AlignVertical::Top);
        gui.set_min_width(100.0);
        gui.set_children_alignment(AlignHorizontal::Center, AlignVertical::Top);
        gui.rect();
        gui.set_fixed_size(40.0, 10.0);
        gui.end_container();

        gui.end_gui(&mut r, &input);

        let root_widget_rf = gui.root();
        let root_widget = root_widget_rf.as_ref();

        check_widget(
            &root_widget,
            &WidgetCheck(
                "Root",
                (0.0, 0.0),
                (300.0, 200.0),
                (300.0, 200.0),
                Some(vec![
                    WidgetCheck(
                        "Percent",
                        (0.0, 0.0),
                        (150.0, 10.0),
                        (150.0, 10.0),
                        Some(vec![WidgetCheck(
                            "Rect1",
                            (55.0, 0.0),
                            (40.0, 10.0),
                            (40.0, 10.0),
                            None,
                        )]),
                    ),
                    WidgetCheck(
                        "MinWidth",
                        (200.0, 0.0),
                        (100.0, 10.0),
                        (100.0, 10.0),
                        Some(vec![WidgetCheck(
                            "Rect2",
                            (230.0, 0.0),
                            (40.0, 10.0),
                            (40.0, 10.0),
                            None,
                        )]),
                    ),
                ]),
            ),
        );
    }
}
