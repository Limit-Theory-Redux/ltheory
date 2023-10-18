use glam::Vec4;

use crate::render::UIRenderer_Rect;

use super::{HmGui, HmGuiWidget, Rf};

#[derive(Clone, PartialEq)]
pub struct HmGuiRect {
    pub widget: Rf<HmGuiWidget>,
    pub color: Vec4,
}

impl HmGuiRect {
    pub fn draw(&self) {
        let widget = self.widget.as_ref();

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
