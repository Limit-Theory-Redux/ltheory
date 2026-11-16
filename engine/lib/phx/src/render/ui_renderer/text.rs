use glam::Vec2;

use crate::render::{Color, Font};

#[derive(Clone)]
pub struct UIRendererText {
    pub pos: Vec2,
    pub font: *const Font,
    pub text: String,
    pub color: Color,
}
