use glam::Vec2;

use crate::render::Tex2D;

use super::HmGui;

#[derive(Default, Clone, Copy, PartialEq)]
pub enum HmGuiImageLayout {
    #[default]
    Fit,
    Center,
    TopLeft,
}

#[derive(Clone, PartialEq)]
pub struct HmGuiImage {
    pub image: *mut Tex2D,
    pub layout: HmGuiImageLayout,
}

impl HmGuiImage {
    pub fn draw(&self, hmgui: &mut HmGui, mut pos: Vec2, size: Vec2) {
        debug_assert_ne!(self.image, std::ptr::null_mut(), "Image pointer is null");

        if self.layout == HmGuiImageLayout::Fit {
            hmgui.renderer.image(self.image, pos, size);
        } else {
            let tex = unsafe { &*self.image };
            let tex_size = Vec2::new(tex.size.x as f32, tex.size.y as f32);

            hmgui.renderer.begin_layer(pos, size, true);

            if self.layout == HmGuiImageLayout::Center {
                pos += (size - tex_size) / 2.0;
            }

            hmgui.renderer.image(self.image, pos, tex_size);

            hmgui.renderer.end_layer();
        }
    }

    // For testing.
    #[allow(dead_code)]
    pub(crate) fn dump(&self, ident: usize) {
        let ident_str = format!("{}", crate::ui::hmgui::IDENT.repeat(ident));

        println!("{ident_str}- image: {:?}", self.image);
    }
}
