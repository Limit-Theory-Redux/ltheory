use super::*;
use crate::common::*;
use crate::math::*;
use crate::render::*;

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
    pub fn draw(
        &self,
        panel_shader: &mut Shader,
        layers: &Vec<UIRendererLayer>,
        images: &Vec<UIRendererImage>,
        panels: &Vec<UIRendererPanel>,
        rects: &Vec<UIRendererRect>,
        texts: &Vec<UIRendererText>,
    ) {
        unsafe {
            if self.clip {
                // extend clip area by 1 pixel to avoid border overlapping
                ClipRect_PushCombined(
                    self.pos.x - 1.0,
                    self.pos.y - 1.0,
                    self.size.x + 2.0,
                    self.size.y + 2.0,
                );
            }

            if self.panel_id.is_some() {
                Shader_Start(panel_shader);

                let pad: f32 = 64.0;
                Shader_SetFloat(c_str!("padding"), pad);

                let mut panel_id_opt = self.panel_id;
                while let Some(panel_id) = panel_id_opt {
                    let panel = &panels[*panel_id];

                    let x = panel.pos.x - pad;
                    let y = panel.pos.y - pad;
                    let sx = panel.size.x + 2.0 * pad;
                    let sy = panel.size.y + 2.0 * pad;

                    Shader_SetFloat(c_str!("innerAlpha"), panel.inner_alpha);
                    Shader_SetFloat(c_str!("bevel"), panel.bevel);
                    Shader_SetFloat2(c_str!("size"), sx, sy);
                    Shader_SetFloat4(
                        c_str!("color"),
                        panel.color.r,
                        panel.color.g,
                        panel.color.b,
                        panel.color.a,
                    );

                    Draw_Rect(x, y, sx, sy);

                    panel_id_opt = panel.next;
                }

                Shader_Stop(panel_shader as *mut _);
            }

            let mut image_id_opt = self.image_id;
            while let Some(image_id) = image_id_opt {
                let image = &images[*image_id];

                Tex2D_Draw(
                    &mut *image.image,
                    image.pos.x,
                    image.pos.y,
                    image.size.x,
                    image.size.y,
                );
                image_id_opt = image.next;
            }

            let mut rect_id_opt = self.rect_id;
            while let Some(rect_id) = rect_id_opt {
                let rect = &rects[*rect_id];

                Draw_Color(rect.color.r, rect.color.g, rect.color.b, rect.color.a);

                if let Some(s) = rect.outline {
                    Draw_Border(s, rect.pos.x, rect.pos.y, rect.size.x, rect.size.y);
                } else {
                    Draw_Rect(rect.pos.x, rect.pos.y, rect.size.x, rect.size.y);
                }

                rect_id_opt = rect.next;
            }

            let mut text_id_opt = self.text_id;
            while let Some(text_id) = text_id_opt {
                let text = &texts[*text_id];

                (&*text.font).draw(&text.text, text.pos.x, text.pos.y, &text.color);

                text_id_opt = text.next;
            }

            let mut layer_id_opt = self.children;
            while let Some(layer_id) = layer_id_opt {
                let layer = &layers[*layer_id];

                layer.draw(panel_shader, layers, images, panels, rects, texts);

                layer_id_opt = layer.next;
            }

            if self.clip {
                ClipRect_Pop();
            }
        }
    }
}
