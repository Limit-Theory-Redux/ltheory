use glam::Vec2;

use crate::render::{Color, Tex2D};

#[derive(Clone)]
pub struct UIRendererImage {
    pub pos: Vec2,
    pub size: Vec2,
    pub image: Tex2D,
    /// Multiplies every texel: white draws the image as it is.
    pub color: Color,
    /// Bilinear between mip levels instead of the nearest texel.
    pub smooth: bool,
}
