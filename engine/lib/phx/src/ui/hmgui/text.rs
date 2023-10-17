use std::ffi::CString;

use glam::Vec4;
use internal::*;

use crate::render::{Font, UIRenderer_Text};

use super::{HmGui, HmGuiWidgetId};

#[derive(Clone)]
pub struct HmGuiText {
    pub widget_id: HmGuiWidgetId,
    pub font: *mut Font,
    pub text: String,
    pub color: Vec4,
}

impl HmGuiText {
    pub fn draw(&self, hmgui: &HmGui) {
        // #if HMGUI_DRAW_GROUP_FRAMES
        //   Draw_Color(0.5f, 0.2f, 0.2f, 0.5f);
        //   Draw_Border(1.0f, e->pos.x, e->pos.y, e->size.x, e->size.y);
        //#endif

        let widget = hmgui.get_widget(self.widget_id);
        let text = CString::new(self.text.as_str()).expect("Cannot convert text");

        unsafe {
            UIRenderer_Text(
                self.font,
                text.as_ptr(),
                widget.pos.x,
                widget.pos.y + widget.minSize.y,
                self.color.x,
                self.color.y,
                self.color.z,
                self.color.w,
            );
        }
    }
}
