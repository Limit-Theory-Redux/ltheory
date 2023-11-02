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

    use super::{HmGui, HmGuiContainer, HmGuiWidget};

    fn init_test() -> (HmGui, Input) {
        // let subscriber = FmtSubscriber::builder()
        //     .with_max_level(Level::DEBUG)
        //     .with_target(false)
        //     .with_ansi(true)
        //     .finish();
        // let _ = tracing::subscriber::set_global_default(subscriber);

        (HmGui::new(Default::default()), Default::default())
    }

    fn check_pos_size(
        widget: &Ref<'_, HmGuiWidget>,
        pos: (f32, f32),
        size: (f32, f32),
        name: &str,
    ) {
        assert_eq!(widget.pos, Vec2::new(pos.0, pos.1), "Root widget position");
        assert_eq!(widget.size, Vec2::new(size.0, size.1), "{name} widget size");
    }

    fn check_children<'a>(
        widget: &'a Ref<'_, HmGuiWidget>,
        count: usize,
        name: &str,
    ) -> &'a HmGuiContainer {
        let container = widget
            .get_container_item()
            .expect(&format!("Cannot get {name} container"));
        assert_eq!(
            container.children.len(),
            count,
            "Children count {name} container"
        );

        container
    }

    #[test]
    fn test_hmgui_stack_layout_basic() {
        let (mut gui, input) = init_test();

        gui.begin_gui(300.0, 200.0, &input);
        gui.begin_stack_container();
        // gui.text("Text");
        gui.rect(30.0, 20.0, 0.0, 1.0, 0.0, 1.0);
        gui.rect(20.0, 30.0, 0.0, 1.0, 0.0, 1.0);
        gui.end_container();
        gui.set_docking(DOCKING_STRETCH_ALL);
        gui.end_gui(&input);

        // gui.dump_widgets(None);

        let root_widget_rf = gui.root().expect("Cannot get gui root widget");
        let root_widget = root_widget_rf.as_ref();

        // Root widget should always keep it's position and size
        check_pos_size(&root_widget, (0.0, 0.0), (300.0, 200.0), "Root");

        let root_container = check_children(&root_widget, 1, "root");

        let stack_widget_rf = root_container.children[0].clone();
        let stack_widget = stack_widget_rf.as_ref();

        // Stack container expanded so has the same position and size as root one
        check_pos_size(&stack_widget, (0.0, 0.0), (300.0, 200.0), "Stack");

        let _stack_container = check_children(&stack_widget, 2, "stack");
    }

    // Test cases:
    // 1. Widget docking/stretch has priority over fixed size.
    // 2. Vertical and horizontal containers:
    //    - priority of the container's children docking for the container main dimension over widget's one
    //    - priority of the widget's docking for the container secondary dimension over container's one
    // 3. Sticking to the sides.
    // 4. Min size when not stretched.
    // 5. Oversize parent widget/container.
    // 6. Margin, border, padding, spacing.
    // 7. Text auto expand. (manual only)
}
