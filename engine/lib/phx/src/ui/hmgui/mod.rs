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
        &'static str,
        (f32, f32),
        (f32, f32),
        Option<Vec<WidgetCheck>>,
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
            "{} widget size",
            expected.0
        );

        if let Some(expected_children) = &expected.3 {
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
                Some(vec![WidgetCheck(
                    "Stack",
                    (135.0, 85.0),
                    (30.0, 30.0),
                    Some(vec![
                        WidgetCheck("Rect1", (135.0, 90.0), (30.0, 20.0), None),
                        WidgetCheck("Rect2", (140.0, 85.0), (20.0, 30.0), None),
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
                Some(vec![WidgetCheck(
                    "Stack",
                    (0.0, 0.0),
                    (300.0, 200.0), // Stack container expanded so has the same position and size as root one
                    Some(vec![
                        WidgetCheck("Rect1", (135.0, 90.0), (30.0, 20.0), None),
                        WidgetCheck("Rect2", (140.0, 85.0), (20.0, 30.0), None),
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
                Some(vec![WidgetCheck(
                    "Stack",
                    (0.0, 0.0),
                    (300.0, 200.0), // Stack container expanded so has the same position and size as root one. Ignoring fixed size
                    Some(vec![
                        WidgetCheck("Rect1", (135.0, 90.0), (30.0, 20.0), None),
                        WidgetCheck("Rect2", (140.0, 85.0), (20.0, 30.0), None),
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
                Some(vec![WidgetCheck(
                    "Stack",
                    (0.0, 0.0),
                    (300.0, 200.0), // Stack container expanded so has the same position and size as root one. Ignoring fixed size
                    Some(vec![
                        WidgetCheck("Rect1", (135.0, 0.0), (30.0, 200.0), None), // Stretched vertically
                        WidgetCheck("Rect2", (0.0, 85.0), (300.0, 30.0), None), // Stretched horizontally
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
                Some(vec![WidgetCheck(
                    "Stack",
                    (0.0, 0.0),
                    (300.0, 200.0), // Stack container expanded so has the same position and size as root one
                    Some(vec![
                        WidgetCheck("RectLeft", (0.0, 90.0), (30.0, 20.0), None), // Stick to the left
                        WidgetCheck("RectRight", (280.0, 85.0), (20.0, 30.0), None), // Stick to the right
                        WidgetCheck("RectTop", (135.0, 0.0), (30.0, 20.0), None), // Stick to the top
                        WidgetCheck("RectBottom", (140.0, 170.0), (20.0, 30.0), None), // Stick to the bottom
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
                Some(vec![WidgetCheck(
                    "Stack",
                    (-100.0, -100.0), // TODO: is this correct?
                    (500.0, 400.0), // Stack container expanded so has the same position and size as root one
                    Some(vec![
                        WidgetCheck("Rect1", (-100.0, 90.0), (500.0, 20.0), None), // Vertical oversize
                        WidgetCheck("Rect2", (140.0, -100.0), (20.0, 400.0), None), // Horizontal oversize
                    ]),
                )]),
            ),
        );
    }

    // Test cases:
    // 2. Vertical and horizontal containers:
    //    - priority of the container's children docking for the container main dimension over widget's one
    //    - priority of the widget's docking for the container secondary dimension over container's one
    // 4. Min size when not stretched.
    // 6. Margin, border, padding, spacing.
    // 7. Text auto expand. (manual only)
}
