use glam::{Vec2, Vec4};

#[derive(Clone)]
pub struct UIRendererRect {
    pub next: *mut UIRendererRect,
    pub pos: Vec2,
    pub size: Vec2,
    pub color: Vec4,
    pub outline: bool,
}
