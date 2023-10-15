use crate::render::{Tex2D, UIRenderer_Image};

use super::widget::HmGuiWidget;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGuiImage {
    pub widget: HmGuiWidget,
    pub image: *mut Tex2D,
}

impl HmGuiImage {
    pub fn draw(&self) {
        unsafe {
            UIRenderer_Image(
                self.image,
                self.widget.pos.x,
                self.widget.pos.y,
                self.widget.size.x,
                self.widget.size.y,
            );
        }
    }
}
