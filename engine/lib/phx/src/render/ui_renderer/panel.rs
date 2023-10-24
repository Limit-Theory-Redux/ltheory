use glam::{Vec2, Vec4};

#[derive(Clone)]
pub struct UIRendererPanel {
    pub next: *mut UIRendererPanel,
    pub pos: Vec2,
    pub size: Vec2,
    pub color: Vec4,
    pub bevel: f32,
    pub inner_alpha: f32,
}
