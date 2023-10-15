use glam::Vec4;

use crate::render::UIRenderer_Rect;

use super::widget::HmGuiWidget;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGuiRect {
    pub widget: HmGuiWidget,
    pub color: Vec4,
}

impl HmGuiRect {
    pub fn draw(&self) {
        unsafe {
            UIRenderer_Rect(
                self.widget.pos.x,
                self.widget.pos.y,
                self.widget.size.x,
                self.widget.size.y,
                self.color.x,
                self.color.y,
                self.color.z,
                self.color.w,
                false,
            );
        }
    }
}
