use glam::{Vec2, Vec4};

use super::UIRendererPanelId;

#[derive(Clone)]
pub struct UIRendererPanel {
    pub next: Option<UIRendererPanelId>,
    pub pos: Vec2,
    pub size: Vec2,
    pub color: Vec4,
    pub bevel: f32,
    pub inner_alpha: f32,
}
