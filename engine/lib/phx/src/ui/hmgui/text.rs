use glam::Vec4;
use internal::*;

use crate::render::{Font, UIRenderer_Text};

use super::widget::HmGuiWidget;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGuiText {
    pub widget: HmGuiWidget,
    pub font: *mut Font,
    pub text: *const libc::c_char,
    pub color: Vec4,
}

pub unsafe extern "C" fn HmGui_FreeText(e: *mut HmGuiText) {
    StrFree((*e).text);
    MemFree(e as *const _);
}

impl HmGuiText {
    pub fn draw(&self) {
        // #if HMGUI_DRAW_GROUP_FRAMES
        //   Draw_Color(0.5f, 0.2f, 0.2f, 0.5f);
        //   Draw_Border(1.0f, e->pos.x, e->pos.y, e->size.x, e->size.y);
        //#endif

        unsafe {
            UIRenderer_Text(
                self.font,
                self.text,
                self.widget.pos.x,
                self.widget.pos.y + self.widget.minSize.y,
                self.color.x,
                self.color.y,
                self.color.z,
                self.color.w,
            );
        }
    }
}
