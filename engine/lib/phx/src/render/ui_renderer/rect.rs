use glam::Vec2;

use crate::render::Color;

use super::UIRendererRectId;

#[derive(Clone)]
pub struct UIRendererRect {
    pub next: Option<UIRendererRectId>,
    pub pos: Vec2,
    pub size: Vec2,
    pub color: Color,
    pub outline: Option<f32>,
}
