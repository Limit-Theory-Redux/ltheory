use glam::Vec2;

use super::UIRendererTextId;
use crate::render::{Color, Font, TextOutline};

#[derive(Clone)]
pub struct UIRendererText {
    pub next: Option<UIRendererTextId>,
    pub pos: Vec2,
    pub font: *const Font,
    pub text: String,
    pub color: Color,
    pub outline: Option<TextOutline>,
}
