use glam::Vec2;

use crate::common::*;
use crate::math::*;
use crate::render::*;

use super::image::UIRendererImage;
use super::panel::UIRendererPanel;
use super::rect::UIRendererRect;
use super::text::UIRendererText;

#[derive(Clone)]
pub struct UIRendererLayer {
    pub parent: *mut UIRendererLayer,
    pub next: *mut UIRendererLayer,
    pub children: *mut UIRendererLayer,
    pub image_list: *mut UIRendererImage,
    pub panel_list: *mut UIRendererPanel,
    pub rect_list: *mut UIRendererRect,
    pub text_list: *mut UIRendererText,
    pub pos: Vec2,
    pub size: Vec2,
    pub clip: bool,
}

impl UIRendererLayer {
    pub fn draw(&self) {
        unsafe {
            if self.clip {
                ClipRect_PushCombined(self.pos.x, self.pos.y, self.size.x, self.size.y);
            }
            if !(self.panel_list).is_null() {
                // TODO: Store the shader in the UI renderer and use a Box to manage its memory.
                static mut shader: *mut Shader = std::ptr::null_mut();

                if shader.is_null() {
                    shader = Box::into_raw(Shader_Load(
                        c_str!("vertex/ui"),
                        c_str!("fragment/ui/panel"),
                    ));
                }

                Shader_Start(&mut *shader);

                let pad: f32 = 64.0;
                Shader_SetFloat(c_str!("padding"), pad);

                let mut e = self.panel_list;
                while !e.is_null() {
                    let x: f32 = (*e).pos.x - pad;
                    let y: f32 = (*e).pos.y - pad;
                    let sx: f32 = (*e).size.x + 2.0 * pad;
                    let sy: f32 = (*e).size.y + 2.0 * pad;

                    Shader_SetFloat(c_str!("innerAlpha"), (*e).inner_alpha);
                    Shader_SetFloat(c_str!("bevel"), (*e).bevel);
                    Shader_SetFloat2(c_str!("size"), sx, sy);
                    Shader_SetFloat4(
                        c_str!("color"),
                        (*e).color.x,
                        (*e).color.y,
                        (*e).color.z,
                        (*e).color.w,
                    );

                    Draw_Rect(x, y, sx, sy);

                    e = (*e).next;
                }

                Shader_Stop(shader);
            }

            let mut e_0 = self.image_list;
            while !e_0.is_null() {
                Tex2D_Draw(
                    &mut *(*e_0).image,
                    (*e_0).pos.x,
                    (*e_0).pos.y,
                    (*e_0).size.x,
                    (*e_0).size.y,
                );
                e_0 = (*e_0).next;
            }

            let mut e_1 = self.rect_list;
            while !e_1.is_null() {
                Draw_Color(
                    (*e_1).color.x,
                    (*e_1).color.y,
                    (*e_1).color.z,
                    (*e_1).color.w,
                );

                if (*e_1).outline {
                    Draw_Border(
                        1.0,
                        (*e_1).pos.x,
                        (*e_1).pos.y,
                        (*e_1).size.x,
                        (*e_1).size.y,
                    );
                } else {
                    Draw_Rect((*e_1).pos.x, (*e_1).pos.y, (*e_1).size.x, (*e_1).size.y);
                }

                e_1 = (*e_1).next;
            }

            let mut e_2 = self.text_list;
            while !e_2.is_null() {
                (&*(*e_2).font).draw(
                    &(*e_2).text,
                    (*e_2).pos.x,
                    (*e_2).pos.y,
                    (*e_2).color.x,
                    (*e_2).color.y,
                    (*e_2).color.z,
                    (*e_2).color.w,
                );

                e_2 = (*e_2).next;
            }

            let mut e_3 = self.children;
            while !e_3.is_null() {
                (&*e_3).draw();
                e_3 = (*e_3).next;
            }

            if self.clip {
                ClipRect_Pop();
            }
        }
    }
}
