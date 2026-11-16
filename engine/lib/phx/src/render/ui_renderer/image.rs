use glam::Vec2;

use crate::render::Tex2D;

#[derive(Clone)]
pub struct UIRendererImage {
    pub pos: Vec2,
    pub size: Vec2,
    pub image: Tex2D,
}
