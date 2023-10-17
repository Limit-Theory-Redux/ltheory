use glam::Vec4;

use crate::render::UIRenderer_Rect;

use super::{HmGui, HmGuiWidgetId};

#[derive(Clone)]
pub struct HmGuiRect {
    pub widget_id: HmGuiWidgetId,
    pub color: Vec4,
}

impl HmGuiRect {
    pub fn draw(&self, hmgui: &HmGui) {
        let widget = hmgui.get_widget(self.widget_id);

        unsafe {
            UIRenderer_Rect(
                widget.pos.x,
                widget.pos.y,
                widget.size.x,
                widget.size.y,
                self.color.x,
                self.color.y,
                self.color.z,
                self.color.w,
                false,
            );
        }
    }
}
