use glam::Vec2;

use super::image::UIRendererImage;
use super::panel::UIRendererPanel;
use super::rect::UIRendererRect;
use super::shape::{UIRendererShape, UIShapeGeometry};
use super::text::UIRendererText;
use super::{
    UIRendererImageId, UIRendererLayerId, UIRendererPanelId, UIRendererRectId, UIRendererShapeId,
    UIRendererTextId,
};
use crate::render::{ClipRect, Color, Renderer, Samplers, Shape};

/// One thing a layer draws, in the order it was added: what is added later is
/// drawn over what came before (painter's order, as in the widget tree).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UIRendererItem {
    Image(UIRendererImageId),
    Panel(UIRendererPanelId),
    Rect(UIRendererRectId),
    Shape(UIRendererShapeId),
    Text(UIRendererTextId),
    /// A nested layer (a container's children), drawn at this point.
    Layer(UIRendererLayerId),
}

/// The items of all layers, which the layers refer to by id.
pub struct UIRendererItems<'a> {
    pub layers: &'a [UIRendererLayer],
    pub images: &'a [UIRendererImage],
    pub panels: &'a [UIRendererPanel],
    pub rects: &'a [UIRendererRect],
    pub shapes: &'a [UIRendererShape],
    pub texts: &'a [UIRendererText],
}

#[derive(Default)]
pub struct UIRendererLayer {
    pub parent: Option<UIRendererLayerId>,
    /// What this layer draws, in drawing order.
    pub items: Vec<UIRendererItem>,

    pub pos: Vec2,
    pub size: Vec2,
    pub clip: bool,
}

impl UIRendererLayer {
    pub fn draw(&self, r: &mut Renderer, all: &UIRendererItems<'_>) {
        if self.clip {
            // extend clip area by 1 pixel to avoid border overlapping
            ClipRect::push_combined(
                r,
                self.pos.x - 1.0,
                self.pos.y - 1.0,
                self.size.x + 2.0,
                self.size.y + 2.0,
            );
        }

        for item in &self.items {
            match *item {
                UIRendererItem::Image(id) => Self::draw_image(r, &all.images[*id]),
                UIRendererItem::Panel(id) => Self::draw_panel(r, &all.panels[*id]),
                UIRendererItem::Rect(id) => Self::draw_rect(r, &all.rects[*id]),
                UIRendererItem::Shape(id) => Self::draw_shape(r, &all.shapes[*id]),
                UIRendererItem::Text(id) => Self::draw_text(r, &all.texts[*id]),
                UIRendererItem::Layer(id) => all.layers[*id].draw(r, all),
            }
        }

        if self.clip {
            ClipRect::pop(r);
        }
    }

    fn draw_panel(r: &mut Renderer, panel: &UIRendererPanel) {
        let pad: f32 = 64.0;
        let x = panel.pos.x - pad;
        let y = panel.pos.y - pad;
        let sx = panel.size.x + 2.0 * pad;
        let sy = panel.size.y + 2.0 * pad;

        r.imm_shape(
            Shape::Panel,
            x,
            y,
            sx,
            sy,
            &panel.color,
            [panel.inner_alpha, panel.bevel, 0.0, 0.0],
        );
    }

    fn draw_image(r: &mut Renderer, image: &UIRendererImage) {
        // UI images were sampled with the texture's own state: nearest
        // filtering, clamped (`Tex2D::load`).
        r.imm_textured(
            Shape::Image,
            image.image.view(),
            Samplers::Point.id(),
            [image.pos.x, image.pos.y, image.size.x, image.size.y],
            [0.0, 0.0, 1.0, 1.0],
            &Color::WHITE,
        );
    }

    fn draw_rect(r: &mut Renderer, rect: &UIRendererRect) {
        if let Some(s) = rect.outline {
            let (x, y, w, h) = (rect.pos.x, rect.pos.y, rect.size.x, rect.size.y);
            r.imm_rect(x, y, w, s, &rect.color);
            r.imm_rect(x, y + h - s, w, s, &rect.color);
            r.imm_rect(x, y + s, s, h - 2.0 * s, &rect.color);
            r.imm_rect(x + w - s, y + s, s, h - 2.0 * s, &rect.color);
        } else {
            r.imm_rect(
                rect.pos.x,
                rect.pos.y,
                rect.size.x,
                rect.size.y,
                &rect.color,
            );
        }
    }

    fn draw_shape(r: &mut Renderer, shape: &UIRendererShape) {
        match shape.geometry {
            UIShapeGeometry::Quad {
                pos,
                size,
                params,
                fade,
            } => r.imm_shape_faded(
                shape.shape,
                pos.x,
                pos.y,
                size.x,
                size.y,
                &shape.color,
                params,
                fade,
            ),
            UIShapeGeometry::Triangle { points, pad } => {
                r.imm_tri_shape(shape.shape, points.map(|p| p.to_array()), &shape.color, pad)
            }
            UIShapeGeometry::Stroke { stroke, look } => r.imm_stroke(&stroke, &shape.color, &look),
        }
    }

    fn draw_text(r: &mut Renderer, text: &UIRendererText) {
        #[allow(unsafe_code)] // TODO: remove
        unsafe {
            (*text.font).draw(r, &text.text, text.pos.x, text.pos.y, &text.color);
        }
    }
}
