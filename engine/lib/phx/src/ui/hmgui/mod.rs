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
        // let subscriber = FmtSubscriber::builder()
        //     .with_max_level(Level::DEBUG)
        //     .with_target(false)
        //     .with_ansi(true)
        //     .finish();
        // let _ = tracing::subscriber::set_global_default(subscriber);

        (HmGui::new(Default::default()), Default::default())
    }

    fn check_widget(widget: &Ref<'_, HmGuiWidget>, expected: &WidgetCheck) {
        assert_eq!(
            widget.pos,
            Vec2::new(expected.1 .0, expected.1 .1),
            "{} widget position",
            expected.0
        );
        assert_eq!(
            widget.size,
            Vec2::new(expected.2 .0, expected.2 .1),
            "{} widget outer size",
            expected.0
        );
        assert_eq!(
            widget.inner_size,
            Vec2::new(expected.3 .0, expected.3 .1),
            "{} widget inner size",
            expected.0
        );

        if let Some(expected_children) = &expected.4 {
            let container = widget
                .get_container_item()
                .expect(&format!("Cannot get {} container", expected.0));
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
                widget.get_container_item().is_none(),
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

        let root_widget_rf = gui.root().expect("Cannot get gui root widget");
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

        let root_widget_rf = gui.root().expect("Cannot get gui root widget");
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

        let root_widget_rf = gui.root().expect("Cannot get gui root widget");
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

        let root_widget_rf = gui.root().expect("Cannot get gui root widget");
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

        let root_widget_rf = gui.root().expect("Cannot get gui root widget");
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

        let root_widget_rf = gui.root().expect("Cannot get gui root widget");
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

        let root_widget_rf = gui.root().expect("Cannot get gui root widget");
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

        let root_widget_rf = gui.root().expect("Cannot get gui root widget");
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

        let root_widget_rf = gui.root().expect("Cannot get gui root widget");
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
                        "Stack1",
                        (0.0, 0.0),
                        (300.0, 200.0), // Stack container expanded so has the same position and size as root one
                        (300.0, 200.0),
                        Some(vec![
                            WidgetCheck("Rect1", (135.0, 0.0), (30.0, 20.0), (30.0, 20.0), None),
                            WidgetCheck("Rect2", (140.0, 20.0), (20.0, 30.0), (20.0, 30.0), None),
                        ]),
                    ),
                    WidgetCheck(
                        "Stack2",
                        (0.0, 0.0),
                        (300.0, 200.0), // Stack container expanded so has the same position and size as root one
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

        let root_widget_rf = gui.root().expect("Cannot get gui root widget");
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

        let root_widget_rf = gui.root().expect("Cannot get gui root widget");
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

        let root_widget_rf = gui.root().expect("Cannot get gui root widget");
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
                        "Stack1",
                        (0.0, 0.0),
                        (300.0, 200.0), // Stack container expanded so has the same position and size as root one
                        (300.0, 200.0),
                        Some(vec![
                            WidgetCheck("Rect1", (0.0, 90.0), (30.0, 20.0), (30.0, 20.0), None),
                            WidgetCheck("Rect2", (30.0, 85.0), (20.0, 30.0), (20.0, 30.0), None),
                        ]),
                    ),
                    WidgetCheck(
                        "Stack2",
                        (0.0, 0.0),
                        (300.0, 200.0), // Stack container expanded so has the same position and size as root one
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

        let root_widget_rf = gui.root().expect("Cannot get gui root widget");
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

        let root_widget_rf = gui.root().expect("Cannot get gui root widget");
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
}
