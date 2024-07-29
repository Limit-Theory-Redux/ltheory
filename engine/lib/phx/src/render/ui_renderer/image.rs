use glam::Vec2;

use super::UIRendererImageId;
use crate::render::Tex2D;

#[derive(Clone)]
pub struct UIRendererImage {
    pub next: Option<UIRendererImageId>,
    pub pos: Vec2,
    pub size: Vec2,
    pub image: *mut Tex2D,
}
