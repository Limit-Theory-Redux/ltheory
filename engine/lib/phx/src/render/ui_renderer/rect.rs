use glam::Vec2;

use crate::render::Color;

#[derive(Clone)]
pub struct UIRendererRect {
    pub pos: Vec2,
    pub size: Vec2,
    pub color: Color,
    pub outline: Option<f32>,
}
