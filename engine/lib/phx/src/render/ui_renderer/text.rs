use glam::{Vec2, Vec4};

use crate::render::Font;

#[derive(Clone)]
pub struct UIRendererText {
    pub next: *mut UIRendererText,
    pub font: *const Font,
    pub text: String,
    pub pos: Vec2,
    pub color: Vec4,
}
