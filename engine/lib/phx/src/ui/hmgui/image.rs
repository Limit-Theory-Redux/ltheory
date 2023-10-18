use crate::render::{Tex2D, UIRenderer_Image};

use super::{HmGui, HmGuiWidget, Rf};

#[derive(Clone, PartialEq)]
pub struct HmGuiImage {
    pub widget: Rf<HmGuiWidget>,
    pub image: *mut Tex2D,
}

impl HmGuiImage {
    pub fn draw(&self) {
        let widget = self.widget.as_ref();

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
