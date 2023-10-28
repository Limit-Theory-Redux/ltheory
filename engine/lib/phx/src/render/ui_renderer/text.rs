use glam::{Vec2, Vec4};

use crate::render::Font;

use super::UIRendererTextId;

#[derive(Clone)]
pub struct UIRendererText {
    pub next: Option<UIRendererTextId>,
    pub pos: Vec2,
    pub font: Font,
    pub text: String,
    pub color: Vec4,
}
