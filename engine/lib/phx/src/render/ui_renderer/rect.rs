use glam::{Vec2, Vec4};

use super::UIRendererRectId;

#[derive(Clone)]
pub struct UIRendererRect {
    pub next: Option<UIRendererRectId>,
    pub pos: Vec2,
    pub size: Vec2,
    pub color: Vec4,
    pub outline: bool,
}
