use glam::Vec2;

use crate::render::Tex2D;

use super::UIRendererImageId;

#[derive(Clone)]
pub struct UIRendererImage {
    pub next: Option<UIRendererImageId>,
    pub pos: Vec2,
    pub size: Vec2,
    pub image: *mut Tex2D,
}
