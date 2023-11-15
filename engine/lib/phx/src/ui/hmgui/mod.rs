mod container;
mod data;
mod docking;
mod focus;
mod gui;
mod image;
mod rect;
mod rf;
mod style;
mod text;
mod widget;

use internal::*;

pub use container::*;
pub(self) use data::*;
pub use docking::*;
pub use focus::*;
pub use gui::*;
pub use image::*;
pub use rect::*;
pub use rf::*;
pub use style::*;
pub use text::*;
pub use widget::*;

pub(self) const IDENT: &str = "  ";

#[cfg(test)]
mod tests {
    use std::cell::Ref;

    use glam::Vec2;
    use tracing::Level;
    use tracing_subscriber::FmtSubscriber;

    use crate::{
        input::Input,
        ui::hmgui::{Docking, DOCKING_STRETCH_ALL},
    };

    use super::*;

    struct WidgetCheck(
        &'static str,             // Widget name
        (f32, f32),               // Widget position
        (f32, f32),               // Widget outer size
        (f32, f32),               // Widget inner size
        Option<Vec<WidgetCheck>>, // Container widget children. None for non-container widget
    );

    fn init_test() -> (HmGui, Input) {
        (HmGui::new(Default::default()), Default::default())
    }

    fn check_widget(widget: &Ref<'_, HmGuiWidget>, expected: &WidgetCheck) {
        assert_eq!(
            (widget.pos, widget.size, widget.inner_size),
            (
                Vec2::new(expected.1 .0, expected.1 .1),
                Vec2::new(expected.2 .0, expected.2 .1),
                Vec2::new(expected.3 .0, expected.3 .1)
            ),
            "{} widget position, outer and inner sizes",
            expected.0
        );

        if let Some(expected_children) = &expected.4 {
            let container = widget.get_container_item();
            assert_eq!(
                container.children.len(),
                expected_children.len(),
                "Children count {} container",
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
                "Expected non-container item for: {}",
                expected.0
            );
        }
    }

    #[test]
    fn test_hmgui_stack_layout_basic() {
        let (mut gui, input) = init_test();

        gui.begin_gui(300.0, 200.0, &input);
        gui.begin_stack_container();

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(30.0, 20.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(20.0, 30.0);

        gui.end_container();
        gui.end_gui(&input);

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
        let (mut gui, input) = init_test();

        gui.begin_gui(300.0, 200.0, &input);
        gui.begin_stack_container();

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(30.0, 20.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(20.0, 30.0);

        gui.end_container();
        gui.set_docking(DOCKING_STRETCH_ALL);
        gui.end_gui(&input);

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
        let (mut gui, input) = init_test();

        gui.begin_gui(300.0, 200.0, &input);
        gui.begin_stack_container();

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(30.0, 20.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(20.0, 30.0);

        gui.end_container();
        gui.set_fixed_size(50.0, 50.0);
        gui.set_docking(DOCKING_STRETCH_ALL);
        gui.end_gui(&input);

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
        let (mut gui, input) = init_test();

        gui.begin_gui(300.0, 200.0, &input);
        gui.begin_stack_container();

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_width(30.0);
        gui.set_docking(DOCKING_STRETCH_VERTICAL);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_height(30.0);
        gui.set_docking(DOCKING_STRETCH_HORIZONTAL);

        gui.end_container();
        gui.set_docking(DOCKING_STRETCH_ALL);
        gui.end_gui(&input);

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
        let (mut gui, input) = init_test();

        gui.begin_gui(300.0, 200.0, &input);
        gui.begin_stack_container();

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(30.0, 20.0);
        gui.set_docking(DOCKING_LEFT);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(20.0, 30.0);
        gui.set_docking(DOCKING_RIGHT);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(30.0, 20.0);
        gui.set_docking(DOCKING_TOP);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(20.0, 30.0);
        gui.set_docking(DOCKING_BOTTOM);

        gui.end_container();
        gui.set_docking(DOCKING_STRETCH_ALL);
        gui.end_gui(&input);

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
        let (mut gui, input) = init_test();

        gui.begin_gui(300.0, 200.0, &input);
        gui.begin_stack_container();

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(500.0, 20.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(20.0, 400.0);

        gui.end_container();
        gui.set_docking(DOCKING_STRETCH_ALL);
        gui.end_gui(&input);

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
        let (mut gui, input) = init_test();

        gui.begin_gui(300.0, 200.0, &input);
        gui.begin_vertical_container();
        gui.set_spacing(0.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(30.0, 20.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_height(20.0);
        gui.set_docking(DOCKING_STRETCH_HORIZONTAL);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_width(20.0);
        gui.set_docking(DOCKING_STRETCH_VERTICAL);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(20.0, 30.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_docking(DOCKING_STRETCH_ALL);

        gui.end_container();
        gui.set_docking(DOCKING_STRETCH_ALL);
        gui.end_gui(&input);

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
        let (mut gui, input) = init_test();

        gui.begin_gui(300.0, 200.0, &input);
        gui.begin_vertical_container();
        gui.set_spacing(0.0);
        gui.set_children_docking(DOCKING_STRETCH_HORIZONTAL);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(30.0, 20.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(20.0, 30.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_docking(DOCKING_STRETCH_ALL);

        gui.end_container();
        gui.set_docking(DOCKING_STRETCH_ALL);
        gui.end_gui(&input);

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
        let (mut gui, input) = init_test();

        gui.begin_gui(300.0, 200.0, &input);

        // Vertical 1: dock top
        gui.begin_vertical_container();
        gui.set_spacing(0.0);
        gui.set_children_docking(DOCKING_TOP);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(30.0, 20.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(20.0, 30.0);

        gui.end_container();
        gui.set_docking(DOCKING_STRETCH_ALL);

        // Vertical 2: dock bottom
        gui.begin_vertical_container();
        gui.set_spacing(0.0);
        gui.set_children_docking(DOCKING_BOTTOM);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(30.0, 20.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(20.0, 30.0);

        gui.end_container();
        gui.set_docking(DOCKING_STRETCH_ALL);

        gui.end_gui(&input);

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
        let (mut gui, input) = init_test();

        gui.begin_gui(300.0, 200.0, &input);
        gui.begin_horizontal_container();
        gui.set_spacing(0.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(30.0, 20.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_width(20.0);
        gui.set_docking(DOCKING_STRETCH_VERTICAL);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_height(20.0);
        gui.set_docking(DOCKING_STRETCH_HORIZONTAL);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(20.0, 30.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_docking(DOCKING_STRETCH_ALL);

        gui.end_container();
        gui.set_docking(DOCKING_STRETCH_ALL);
        gui.end_gui(&input);

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
        let (mut gui, input) = init_test();

        gui.begin_gui(300.0, 200.0, &input);
        gui.begin_horizontal_container();
        gui.set_spacing(0.0);
        gui.set_children_docking(DOCKING_STRETCH_VERTICAL);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(30.0, 20.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(20.0, 30.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_docking(DOCKING_STRETCH_ALL);

        gui.end_container();
        gui.set_docking(DOCKING_STRETCH_ALL);
        gui.end_gui(&input);

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
        let (mut gui, input) = init_test();

        gui.begin_gui(300.0, 200.0, &input);

        // Horizontal 1: dock left
        gui.begin_horizontal_container();
        gui.set_spacing(0.0);
        gui.set_children_docking(DOCKING_LEFT);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(30.0, 20.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(20.0, 30.0);

        gui.end_container();
        gui.set_docking(DOCKING_STRETCH_ALL);

        // Horizontal 2: dock right
        gui.begin_horizontal_container();
        gui.set_spacing(0.0);
        gui.set_children_docking(DOCKING_RIGHT);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(30.0, 20.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(20.0, 30.0);

        gui.end_container();
        gui.set_docking(DOCKING_STRETCH_ALL);

        gui.end_gui(&input);

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
        let (mut gui, input) = init_test();

        gui.begin_gui(300.0, 200.0, &input);

        // Horizontal
        gui.begin_horizontal_container();
        gui.set_spacing(0.0);
        gui.set_children_docking(DOCKING_STRETCH_HORIZONTAL);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_height(20.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_height(30.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_height(40.0);

        gui.end_container();
        gui.set_docking(DOCKING_STRETCH_ALL);

        // Vertical
        gui.begin_vertical_container();
        gui.set_spacing(0.0);
        gui.set_children_docking(DOCKING_STRETCH_VERTICAL);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_width(20.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_width(30.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_width(40.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_width(50.0);

        gui.end_container();
        gui.set_docking(DOCKING_STRETCH_ALL);

        gui.end_gui(&input);

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
        let (mut gui, input) = init_test();

        gui.begin_gui(300.0, 200.0, &input);
        gui.begin_vertical_container();
        gui.set_padding(20.0, 20.0);
        gui.set_spacing(10.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(30.0, 20.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(20.0, 30.0);
        gui.set_margin(13.0, 13.0);
        gui.set_border_width(2.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_docking(DOCKING_STRETCH_ALL);

        gui.end_container();
        gui.set_margin(5.0, 5.0);
        gui.set_border_width(5.0);
        gui.set_docking(DOCKING_STRETCH_ALL);
        gui.end_gui(&input);

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
        let (mut gui, input) = init_test();

        gui.begin_gui(300.0, 200.0, &input);
        gui.begin_horizontal_container();
        gui.set_padding(20.0, 20.0);
        gui.set_spacing(10.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(30.0, 20.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(20.0, 30.0);
        gui.set_margin(13.0, 13.0);
        gui.set_border_width(2.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_docking(DOCKING_STRETCH_ALL);

        gui.end_container();
        gui.set_margin(5.0, 5.0);
        gui.set_border_width(5.0);
        gui.set_docking(DOCKING_STRETCH_ALL);
        gui.end_gui(&input);

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

    #[test]
    fn test_hmgui_stack_percentage_size() {
        let (mut gui, input) = init_test();

        gui.begin_gui(300.0, 200.0, &input);

        // Stack
        gui.begin_stack_container();

        // Inside container
        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_percent_width(70.0);
        gui.set_fixed_height(20.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_width(30.0);
        gui.set_percent_height(60.0);

        // Outside container
        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_percent_width(120.0);
        gui.set_fixed_height(20.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_width(30.0);
        gui.set_percent_height(130.0);

        gui.end_container();
        gui.set_docking(DOCKING_STRETCH_ALL);

        gui.end_gui(&input);

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
                        WidgetCheck("Rect1", (55.5, 90.0), (189.0, 20.0), (189.0, 20.0), None),
                        WidgetCheck("Rect2", (135.0, 46.0), (30.0, 108.0), (30.0, 108.0), None),
                        WidgetCheck("Rect3", (-12.0, 90.0), (324.0, 20.0), (324.0, 20.0), None),
                        WidgetCheck("Rect4", (135.0, -17.0), (30.0, 234.0), (30.0, 234.0), None),
                    ]),
                )]),
            ),
        );
    }

    #[test]
    fn test_hmgui_horizontal_percentage_size() {
        let (mut gui, input) = init_test();

        gui.begin_gui(300.0, 200.0, &input);

        // Horizontal: first widget - fixed size - 20, second - 50% = 140, last - expands to non remaining width - 140
        gui.begin_horizontal_container();
        gui.set_spacing(0.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(20.0, 30.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_percent_width(50.0);
        gui.set_fixed_height(30.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_height(40.0);
        gui.set_docking(DOCKING_STRETCH_HORIZONTAL);

        gui.end_container();
        gui.set_docking(DOCKING_STRETCH_ALL);

        // Horizontal: first widget - fixed size - 20, second - 150% = 420, last widget shrinks to 0 width
        gui.begin_horizontal_container();
        gui.set_spacing(0.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(20.0, 30.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_percent_width(150.0);
        gui.set_fixed_height(30.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_height(40.0);
        gui.set_docking(DOCKING_STRETCH_HORIZONTAL);

        gui.end_container();
        gui.set_docking(DOCKING_STRETCH_ALL);

        gui.end_gui(&input);

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
                            WidgetCheck("Rect2", (20.0, 85.0), (140.0, 30.0), (140.0, 30.0), None),
                            WidgetCheck("Rect3", (160.0, 80.0), (140.0, 40.0), (140.0, 40.0), None),
                        ]),
                    ),
                    WidgetCheck(
                        "Horizontal2",
                        (0.0, 0.0),
                        (300.0, 200.0), // Horizontal container expanded so has the same position and size as root one
                        (300.0, 200.0),
                        Some(vec![
                            WidgetCheck("Rect1", (-70.0, 85.0), (20.0, 30.0), (20.0, 30.0), None),
                            WidgetCheck("Rect2", (-50.0, 85.0), (420.0, 30.0), (420.0, 30.0), None),
                            WidgetCheck("Rect3", (370.0, 80.0), (0.0, 40.0), (0.0, 40.0), None),
                        ]),
                    ),
                ]),
            ),
        );
    }

    #[test]
    fn test_hmgui_vertical_percentage_size() {
        let (mut gui, input) = init_test();

        gui.begin_gui(300.0, 200.0, &input);

        // Vertical: first widget - fixed size - 30, second - 50% = 85, last - expands to non remaining width - 115
        gui.begin_vertical_container();
        gui.set_spacing(0.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(20.0, 30.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_width(30.0);
        gui.set_percent_height(50.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_width(40.0);
        gui.set_docking(DOCKING_STRETCH_VERTICAL);

        gui.end_container();
        gui.set_docking(DOCKING_STRETCH_ALL);

        // Vertical: first widget - fixed size - 30, second - 120% = 204, last widget shrinks to 0 width
        gui.begin_vertical_container();
        gui.set_spacing(0.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_size(20.0, 30.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_width(30.0);
        gui.set_percent_height(120.0);

        gui.rect(0.0, 1.0, 0.0, 1.0);
        gui.set_fixed_width(40.0);
        gui.set_docking(DOCKING_STRETCH_VERTICAL);

        gui.end_container();
        gui.set_docking(DOCKING_STRETCH_ALL);

        gui.end_gui(&input);

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
                            WidgetCheck("Rect2", (135.0, 30.0), (30.0, 85.0), (30.0, 85.0), None),
                            WidgetCheck("Rect3", (130.0, 115.0), (40.0, 85.0), (40.0, 85.0), None),
                        ]),
                    ),
                    WidgetCheck(
                        "Horizontal2",
                        (0.0, 0.0),
                        (300.0, 200.0), // Horizontal container expanded so has the same position and size as root one
                        (300.0, 200.0),
                        Some(vec![
                            WidgetCheck("Rect1", (140.0, -17.0), (20.0, 30.0), (20.0, 30.0), None),
                            WidgetCheck("Rect2", (135.0, 13.0), (30.0, 204.0), (30.0, 204.0), None),
                            WidgetCheck("Rect3", (130.0, 217.0), (40.0, 0.0), (40.0, 0.0), None),
                        ]),
                    ),
                ]),
            ),
        );
    }
}
