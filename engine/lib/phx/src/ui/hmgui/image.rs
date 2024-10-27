use glam::Vec2;

use super::HmGui;
use crate::render::Tex2D;

#[derive(Default, Clone, Copy, PartialEq)]
pub enum HmGuiImageLayout {
    #[default]
    Fit,
    Center,
    TopLeft,
}

#[derive(Clone)]
pub struct HmGuiImage {
    pub image: Tex2D,
    pub layout: HmGuiImageLayout,
}

impl HmGuiImage {
    pub fn draw(&self, hmgui: &mut HmGui, mut pos: Vec2, size: Vec2) {
        let image = self.image.clone();

        if self.layout == HmGuiImageLayout::Fit {
            hmgui.renderer.image(image, pos, size);
        } else {
            let tex_size = image.get_size().as_vec2();

            hmgui.renderer.begin_layer(pos, size, true);

            if self.layout == HmGuiImageLayout::Center {
                pos += (size - tex_size) / 2.0;
            }

            hmgui.renderer.image(image, pos, tex_size);

            hmgui.renderer.end_layer();
        }
    }

    // For testing.
    #[allow(dead_code)]
    pub(crate) fn dump(&self, ident: usize) {
        let ident_str = crate::ui::hmgui::IDENT.repeat(ident).to_string();

        println!("{ident_str}- image: {:?}", self.image);
    }
}
