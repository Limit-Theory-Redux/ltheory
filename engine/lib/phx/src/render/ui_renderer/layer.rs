use glam::Vec2;

use super::image::UIRendererImage;
use super::panel::UIRendererPanel;
use super::rect::UIRendererRect;
use super::text::UIRendererText;
use super::{
    UIRendererImageId, UIRendererLayerId, UIRendererPanelId, UIRendererRectId, UIRendererTextId,
};
use crate::render::{ClipRect, Color, Renderer, Samplers, Shape};

#[derive(Default)]
pub struct UIRendererLayer {
    pub parent: Option<UIRendererLayerId>,
    pub next: Option<UIRendererLayerId>,
    pub children: Option<UIRendererLayerId>,

    pub image_id: Option<UIRendererImageId>,
    pub panel_id: Option<UIRendererPanelId>,
    pub rect_id: Option<UIRendererRectId>,
    pub text_id: Option<UIRendererTextId>,

    pub pos: Vec2,
    pub size: Vec2,
    pub clip: bool,
}

impl UIRendererLayer {
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &self,
        r: &mut Renderer,
        layers: &Vec<UIRendererLayer>,
        images: &Vec<UIRendererImage>,
        panels: &Vec<UIRendererPanel>,
        rects: &Vec<UIRendererRect>,
        texts: &Vec<UIRendererText>,
    ) {
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

        let mut panel_id_opt = self.panel_id;
        while let Some(panel_id) = panel_id_opt {
            let panel = &panels[*panel_id];

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

            panel_id_opt = panel.next;
        }

        let mut image_id_opt = self.image_id;
        while let Some(image_id) = image_id_opt {
            let image = &images[*image_id];

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
            image_id_opt = image.next;
        }

        let mut rect_id_opt = self.rect_id;
        while let Some(rect_id) = rect_id_opt {
            let rect = &rects[*rect_id];

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

            rect_id_opt = rect.next;
        }

        let mut text_id_opt = self.text_id;
        while let Some(text_id) = text_id_opt {
            let text = &texts[*text_id];

            #[allow(unsafe_code)] // TODO: remove
            unsafe {
                (*text.font).draw(r, &text.text, text.pos.x, text.pos.y, &text.color);
            }

            text_id_opt = text.next;
        }

        let mut layer_id_opt = self.children;
        while let Some(layer_id) = layer_id_opt {
            let layer = &layers[*layer_id];

            layer.draw(r, layers, images, panels, rects, texts);

            layer_id_opt = layer.next;
        }

        if self.clip {
            ClipRect::pop(r);
        }
    }
}
