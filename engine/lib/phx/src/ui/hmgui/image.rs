use glam::Vec2;

use crate::render::{Tex2D, UIRenderer};

#[derive(Clone, PartialEq)]
pub struct HmGuiImage {
    pub image: *mut Tex2D,
}

impl HmGuiImage {
    pub fn draw(&self, renderer: &mut UIRenderer, pos: Vec2, size: Vec2) {
        debug_assert_ne!(self.image, std::ptr::null_mut(), "Image pointer is null");
        renderer.image(self.image, pos, size);
    }

    // For testing.
    #[allow(dead_code)]
    pub(crate) fn dump(&self, ident: usize) {
        let ident_str = format!("{}", crate::ui::hmgui::IDENT.repeat(ident));

        println!("{ident_str}- image: {:?}", self.image);
    }
}
