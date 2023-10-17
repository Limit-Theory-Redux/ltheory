use crate::render::{Tex2D, UIRenderer_Image};

use super::{widget::HmGuiWidget, HmGui, HmGuiWidgetId};

#[derive(Clone)]
pub struct HmGuiImage {
    pub widget_id: HmGuiWidgetId,
    pub image: *mut Tex2D,
}

impl HmGuiImage {
    pub fn draw(&self, hmgui: &HmGui) {
        let widget = hmgui.get_widget(self.widget_id);

        unsafe {
            UIRenderer_Image(
                self.image,
                widget.pos.x,
                widget.pos.y,
                widget.size.x,
                widget.size.y,
            );
        }
    }
}
