use glam::Vec2;

use crate::render::Tex2D;

#[derive(Clone)]
pub struct UIRendererImage {
    pub next: *mut UIRendererImage,
    pub image: *mut Tex2D,
    pub pos: Vec2,
    pub size: Vec2,
}
