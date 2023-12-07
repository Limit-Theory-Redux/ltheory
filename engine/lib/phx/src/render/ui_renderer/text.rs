use glam::Vec2;

use crate::render::{Color, Font};

use super::UIRendererTextId;

#[derive(Clone)]
pub struct UIRendererText {
    pub next: Option<UIRendererTextId>,
    pub pos: Vec2,
    pub font: *const Font,
    pub text: String,
    pub color: Color,
}
