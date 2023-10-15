use glam::Vec4;

use crate::render::Font;

#[derive(Clone)]
pub struct HmGuiStyle {
    pub font: *mut Font,
    pub spacing: f32,
    pub colorPrimary: Vec4,
    pub colorFrame: Vec4,
    pub colorText: Vec4,
}

impl Default for HmGuiStyle {
    fn default() -> Self {
        Self {
            font: std::ptr::null_mut(),
            spacing: Default::default(),
            colorPrimary: Default::default(),
            colorFrame: Default::default(),
            colorText: Default::default(),
        }
    }
}
