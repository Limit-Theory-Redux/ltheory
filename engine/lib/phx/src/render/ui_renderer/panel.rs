use glam::Vec2;

use super::UIRendererPanelId;
use crate::render::Color;

#[derive(Clone)]
pub struct UIRendererPanel {
    pub next: Option<UIRendererPanelId>,
    pub pos: Vec2,
    pub size: Vec2,
    pub color: Color,
    pub bevel: f32,
    pub inner_alpha: f32,
}
