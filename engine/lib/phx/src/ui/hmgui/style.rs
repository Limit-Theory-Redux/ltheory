use glam::Vec4;

use crate::render::Font;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGuiStyle {
    pub prev: *mut HmGuiStyle,
    pub font: *mut Font,
    pub spacing: f32,
    pub colorPrimary: Vec4,
    pub colorFrame: Vec4,
    pub colorText: Vec4,
}
