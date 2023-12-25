use glam::Vec4;

use crate::render::Font;

#[derive(Clone, Default)]
pub struct HmGuiStyle {
    pub font: Font,
    pub spacing: f32,
    pub color_primary: Vec4,
    pub color_frame: Vec4,
    pub color_text: Vec4,
}
