use glam::Vec2;

use crate::render::Color;

use super::UIRendererPanelId;

#[derive(Clone)]
pub struct UIRendererPanel {
    pub next: Option<UIRendererPanelId>,
    pub pos: Vec2,
    pub size: Vec2,
    pub color: Color,
    pub bevel: f32,
    pub inner_alpha: f32,
}
