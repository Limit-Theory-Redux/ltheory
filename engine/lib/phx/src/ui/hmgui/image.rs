use glam::Vec2;

use crate::render::{Tex2D, UIRenderer_Image};

use super::{HmGui, HmGuiWidget, Rf};

#[derive(Clone, PartialEq)]
pub struct HmGuiImage {
    pub image: *mut Tex2D,
}

impl HmGuiImage {
    pub fn draw(&self, pos: Vec2, size: Vec2) {
        unsafe {
            UIRenderer_Image(self.image, pos.x, pos.y, size.x, size.y);
        }
    }

    // For testing.
    #[allow(dead_code)]
    pub(crate) fn dump(&self, ident: usize) {
        let ident_str = format!("{}", crate::ui::hmgui::IDENT.repeat(ident));

        println!("{ident_str}- image: {:?}", self.image);
    }
}
