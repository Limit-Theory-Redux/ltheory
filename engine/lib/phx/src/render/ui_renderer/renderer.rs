use glam::Vec2;

use super::UIRendererLayerId;
use super::image::UIRendererImage;
use super::layer::{UIRendererItem, UIRendererItems, UIRendererLayer};
use super::panel::UIRendererPanel;
use super::rect::UIRendererRect;
use super::shape::{UIRendererShape, UIShapeGeometry};
use super::text::UIRendererText;
use crate::render::{Color, Font, Renderer, Shape, Tex2D, TextOutline};

/// Collects what the UI draws in a frame and draws it in the order it was
/// added: each layer draws its items in that order, and a nested layer is
/// drawn where it began. So what comes later in the widget tree is on top.
pub struct UIRenderer {
    current_layer_id: Option<UIRendererLayerId>,

    layers: Vec<UIRendererLayer>,
    images: Vec<UIRendererImage>,
    panels: Vec<UIRendererPanel>,
    rects: Vec<UIRendererRect>,
    shapes: Vec<UIRendererShape>,
    texts: Vec<UIRendererText>,
}

impl UIRenderer {
    pub fn new(_r: &mut Renderer) -> UIRenderer {
        UIRenderer {
            current_layer_id: None,
            layers: Vec::new(),
            images: Vec::new(),
            panels: Vec::new(),
            rects: Vec::new(),
            shapes: Vec::new(),
            texts: Vec::new(),
        }
    }

    pub fn begin(&mut self, r: &Renderer) {
        self.current_layer_id = None;

        self.layers.clear();
        self.images.clear();
        self.panels.clear();
        self.rects.clear();
        self.shapes.clear();
        self.texts.clear();

        let vp = r.target_size();

        self.begin_layer(Vec2::ZERO, Vec2::new(vp.x as f32, vp.y as f32), true);
    }

    pub fn end(&mut self) {
        self.end_layer();
    }

    pub fn draw(&mut self, r: &mut Renderer) {
        let Some(root) = self.layers.first() else {
            unreachable!("No layers defined");
        };

        let all = UIRendererItems {
            layers: &self.layers,
            images: &self.images,
            panels: &self.panels,
            rects: &self.rects,
            shapes: &self.shapes,
            texts: &self.texts,
        };

        root.draw(r, &all);
    }

    /// Begin a nested layer (clipped to `pos`, `size` if `clip`). It is drawn
    /// at this point of its parent layer, over what the parent drew before.
    pub fn begin_layer(&mut self, pos: Vec2, size: Vec2, clip: bool) {
        let layer_id: UIRendererLayerId = self.layers.len().into();

        self.layers.push(UIRendererLayer {
            parent: self.current_layer_id,
            pos,
            size,
            clip,
            ..Default::default()
        });

        if let Some(parent_id) = self.current_layer_id {
            self.layers[*parent_id]
                .items
                .push(UIRendererItem::Layer(layer_id));
        }

        self.current_layer_id = Some(layer_id);
    }

    pub fn end_layer(&mut self) {
        let Some(current_layer_id) = self.current_layer_id else {
            unreachable!();
        };

        self.current_layer_id = self.layers[*current_layer_id].parent;
    }

    /// Add an item to the current layer.
    fn push(&mut self, item: UIRendererItem) {
        let Some(current_layer_id) = self.current_layer_id else {
            unreachable!();
        };

        self.layers[*current_layer_id].items.push(item);
    }

    pub fn image(&mut self, image: Tex2D, pos: Vec2, size: Vec2) {
        let id = self.images.len().into();
        self.images.push(UIRendererImage { pos, size, image });
        self.push(UIRendererItem::Image(id));
    }

    pub fn panel(&mut self, pos: Vec2, size: Vec2, color: Color, bevel: f32, inner_alpha: f32) {
        let id = self.panels.len().into();
        self.panels.push(UIRendererPanel {
            pos,
            size,
            color,
            bevel,
            inner_alpha,
        });
        self.push(UIRendererItem::Panel(id));
    }

    pub fn rect(&mut self, pos: Vec2, size: Vec2, color: Color, outline: Option<f32>) {
        let id = self.rects.len().into();
        self.rects.push(UIRendererRect {
            pos,
            size,
            color,
            outline,
        });
        self.push(UIRendererItem::Rect(id));
    }

    /// One of the engine's `ui/*` shapes.
    pub fn shape(&mut self, shape: Shape, geometry: UIShapeGeometry, color: Color) {
        let id = self.shapes.len().into();
        self.shapes.push(UIRendererShape {
            shape,
            geometry,
            color,
        });
        self.push(UIRendererItem::Shape(id));
    }

    pub fn text(
        &mut self,
        font: &Font,
        text: &str,
        pos: Vec2,
        color: Color,
        outline: Option<TextOutline>,
    ) {
        let id = self.texts.len().into();
        self.texts.push(UIRendererText {
            pos,
            font: font as _,
            text: text.into(),
            color,
            outline,
        });
        self.push(UIRendererItem::Text(id));
    }

    /// The items of the current layer, in drawing order.
    #[cfg(test)]
    pub(crate) fn current_layer_items(&self) -> &[UIRendererItem] {
        let Some(current_layer_id) = self.current_layer_id else {
            unreachable!();
        };

        &self.layers[*current_layer_id].items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty() -> UIRenderer {
        UIRenderer {
            current_layer_id: None,
            layers: Vec::new(),
            images: Vec::new(),
            panels: Vec::new(),
            rects: Vec::new(),
            shapes: Vec::new(),
            texts: Vec::new(),
        }
    }

    #[test]
    fn items_keep_their_order_and_nested_layers_their_place() {
        let mut ui = empty();
        ui.begin_layer(Vec2::ZERO, Vec2::new(100.0, 100.0), true);

        // a container's background, then its children's layer, then a later sibling
        ui.panel(Vec2::ZERO, Vec2::splat(10.0), Color::WHITE, 0.0, 1.0);
        ui.begin_layer(Vec2::ZERO, Vec2::splat(10.0), true);
        ui.rect(Vec2::ZERO, Vec2::splat(5.0), Color::WHITE, None);
        ui.end_layer();
        ui.rect(Vec2::ZERO, Vec2::splat(10.0), Color::WHITE, Some(1.0));
        ui.panel(Vec2::ZERO, Vec2::splat(10.0), Color::WHITE, 0.0, 1.0);

        assert_eq!(
            ui.current_layer_items(),
            &[
                UIRendererItem::Panel(0.into()),
                UIRendererItem::Layer(1.into()),
                UIRendererItem::Rect(1.into()),
                UIRendererItem::Panel(1.into()),
            ]
        );
        assert_eq!(ui.layers[1].items, vec![UIRendererItem::Rect(0.into())]);
        assert_eq!(ui.layers[1].parent, Some(0.into()));

        ui.end_layer();
        assert_eq!(ui.current_layer_id, None);
    }
}
