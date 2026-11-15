use glam::Vec2;

use super::UIRendererShapeId;
use crate::render::{Color, Shape};

/// Where and how a UI shape is drawn, by the engine entry point that draws it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UIShapeGeometry {
    /// A shape centred in the quad `pos`, `size` (glow room included), with
    /// `params` and `fade` as `Renderer::imm_shape_faded` takes them.
    Quad {
        pos: Vec2,
        size: Vec2,
        params: [f32; 4],
        fade: [f32; 4],
    },
    /// A triangle with its corners in pixels (`Renderer::imm_tri_shape`).
    Triangle { points: [Vec2; 3], pad: f32 },
    /// A line between two points in pixels (`Renderer::imm_line_shape`).
    Line { from: Vec2, to: Vec2, pad: f32 },
}

/// One of the engine's `ui/*` shapes.
#[derive(Clone)]
pub struct UIRendererShape {
    pub next: Option<UIRendererShapeId>,
    pub shape: Shape,
    pub geometry: UIShapeGeometry,
    pub color: Color,
}
