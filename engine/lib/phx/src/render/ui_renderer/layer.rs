use glam::Vec2;

use crate::common::*;
use crate::math::*;
use crate::render::*;

#[derive(Clone, Default)]
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
    pub fn draw(&self, renderer: &UIRenderer) {
        unsafe {
            if self.clip {
                ClipRect_PushCombined(self.pos.x, self.pos.y, self.size.x, self.size.y);
            }

            if self.panel_id.is_some() {
                // TODO: Store the shader in the UI renderer and use a Rf/Box to manage its memory.
                static mut SHADER: *mut Shader = std::ptr::null_mut();

                if SHADER.is_null() {
                    SHADER = Box::into_raw(Shader_Load(
                        c_str!("vertex/ui"),
                        c_str!("fragment/ui/panel"),
                    ));
                }

                Shader_Start(&mut *SHADER);

                let pad: f32 = 64.0;
                Shader_SetFloat(c_str!("padding"), pad);

                let mut panel_id_opt = self.panel_id;
                while let Some(panel_id) = panel_id_opt {
                    let panel = &renderer.panels[*panel_id];

                    let x = panel.pos.x - pad;
                    let y = panel.pos.y - pad;
                    let sx = panel.size.x + 2.0 * pad;
                    let sy = panel.size.y + 2.0 * pad;

                    Shader_SetFloat(c_str!("innerAlpha"), panel.inner_alpha);
                    Shader_SetFloat(c_str!("bevel"), panel.bevel);
                    Shader_SetFloat2(c_str!("size"), sx, sy);
                    Shader_SetFloat4(
                        c_str!("color"),
                        panel.color.x,
                        panel.color.y,
                        panel.color.z,
                        panel.color.w,
                    );

                    Draw_Rect(x, y, sx, sy);

                    panel_id_opt = panel.next;
                }

                Shader_Stop(SHADER);
            }

            let mut image_id_opt = self.image_id;
            while let Some(image_id) = image_id_opt {
                let image = &renderer.images[*image_id];

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
                let rect = &renderer.rects[*rect_id];

                Draw_Color(rect.color.x, rect.color.y, rect.color.z, rect.color.w);

                if let Some(s) = rect.outline {
                    Draw_Border(s, rect.pos.x, rect.pos.y, rect.size.x, rect.size.y);
                } else {
                    Draw_Rect(rect.pos.x, rect.pos.y, rect.size.x, rect.size.y);
                }

                rect_id_opt = rect.next;
            }

            let mut text_id_opt = self.text_id;
            while let Some(text_id) = text_id_opt {
                let text = &renderer.texts[*text_id];

                (&*text.font).draw(
                    &text.text,
                    text.pos.x,
                    text.pos.y,
                    text.color.x,
                    text.color.y,
                    text.color.z,
                    text.color.w,
                );

                text_id_opt = text.next;
            }

            let mut layer_id_opt = self.children;
            while let Some(layer_id) = layer_id_opt {
                let layer = &renderer.layers[*layer_id];

                layer.draw(renderer);

                layer_id_opt = layer.next;
            }

            if self.clip {
                ClipRect_Pop();
            }
        }
    }
}
