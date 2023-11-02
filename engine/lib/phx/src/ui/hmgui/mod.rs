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
    use glam::Vec2;
    use tracing::Level;
    use tracing_subscriber::FmtSubscriber;

    use crate::{
        input::Input,
        ui::hmgui::{DockingFlag, DOCKING_STRETCH_ALL},
    };

    use super::HmGui;

    fn init_test() -> (HmGui, Input) {
        // let subscriber = FmtSubscriber::builder()
        //     .with_max_level(Level::DEBUG)
        //     .with_target(false)
        //     .with_ansi(true)
        //     .finish();
        // let _ = tracing::subscriber::set_global_default(subscriber);

        (HmGui::new(Default::default()), Default::default())
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
        assert_eq!(root_widget.pos, Vec2::new(0.0, 0.0), "Root widget position");
        assert_eq!(
            root_widget.size,
            Vec2::new(300.0, 200.0),
            "Root widget size"
        );

        let root_container = root_widget
            .get_container_item()
            .expect("Cannot get root container");
        assert_eq!(
            root_container.children.len(),
            1,
            "Root container children count"
        );

        let stack_widget_rf = root_container.children[0].clone();
        let stack_widget = stack_widget_rf.as_ref();
        assert_eq!(
            stack_widget.pos,
            Vec2::new(0.0, 0.0),
            "Stack widget position"
        );
        assert_eq!(
            stack_widget.size,
            Vec2::new(300.0, 200.0),
            "Stack widget size"
        );

        let stack_container = stack_widget
            .get_container_item()
            .expect("Cannot get stack container");
        assert_eq!(
            stack_container.children.len(),
            2,
            "Stack container children count"
        );
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
