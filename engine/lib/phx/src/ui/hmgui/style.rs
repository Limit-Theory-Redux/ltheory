use glam::Vec4;

use crate::render::Font;

#[derive(Clone, Default)]
pub struct HmGuiStyle {
    pub font: Font,
    pub spacing: f32,
    pub colorPrimary: Vec4,
    pub colorFrame: Vec4,
    pub colorText: Vec4,
}
