use std::collections::HashMap;
use std::ffi::CStr;
use std::ffi::CString;

use super::*;
use crate::common::*;
use crate::math::*;
use crate::system::{Profiler_Begin, Profiler_End, ResourceType_Font, Resource_GetPath};
use crate::ui::hmgui::Rf;

use freetype::face::{KerningMode, LoadFlag};
use internal::*;
use tracing::warn;

/* TODO : Atlas instead of individual textures. */

/* NOTE : Gamma of 1.8 recommended by FreeType */
const K_GAMMA: f64 = 1.8;
const K_RCP_GAMMA: f64 = 1.0 / K_GAMMA;

#[derive(Clone)]
pub struct Font(Rf<FontData>);

impl Font {
    pub fn new(name: &str, face: freetype::Face) -> Self {
        Self(
            FontData {
                name: name.into(),
                face,
                glyphs: Default::default(),
            }
            .into(),
        )
    }
}

impl Drop for Font {
    #[track_caller]
    fn drop(&mut self) {
        panic!("Delete font: {}", self.0.as_ref().name);
    }
}

#[derive(Clone)]
struct FontData {
    name: String,
    face: freetype::Face,
    glyphs: HashMap<u32, Glyph>,
}

#[derive(Clone)]
pub struct Glyph {
    pub index: u32,
    pub tex: *mut Tex2D,
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
    pub sx: i32,
    pub sy: i32,
    pub advance: i32,
}

impl Font {
    pub fn name(&self) -> String {
        let font_data = self.0.as_ref();

        font_data.name.clone()
    }

    fn get_glyph(&self, code_point: u32) {
        tracing::warn!("Font::get_glyph({code_point})");

        let mut font_data = self.0.as_mut();

        if !font_data.glyphs.contains_key(&code_point) {
            let face = &font_data.face;
            let glyph_index = face
                .get_char_index(code_point as usize)
                .expect(&format!("Cannot get char index for: {code_point}"));

            if face
                .load_glyph(
                    glyph_index.get(),
                    LoadFlag::FORCE_AUTOHINT | LoadFlag::RENDER,
                )
                .is_err()
            {
                warn!("Cannot load glyph for: {glyph_index}");

                return;
            }

            let glyph = face.glyph();
            let bitmap = glyph.bitmap();

            /* Create a new glyph and fill out metrics. */
            let x0 = glyph.bitmap_left();
            let y0 = -glyph.bitmap_top();
            let sx = bitmap.width();
            let sy = bitmap.rows();

            let mut glyph = Glyph {
                index: glyph_index.get(),
                tex: std::ptr::null_mut(),
                x0,
                y0,
                sx,
                sy,
                x1: x0 + sx,
                y1: y0 + sy,
                advance: (glyph.advance().x >> 6) as _,
            };

            let mut buffer = Vec::with_capacity((glyph.sx * glyph.sy) as usize);

            /* Copy rendered bitmap into buffer. */
            let p_bitmap = bitmap.buffer();
            let mut pitch = 0;
            for _dy in 0..bitmap.rows() {
                for dx in 0..bitmap.width() {
                    let value = (p_bitmap[pitch + dx as usize] as f32 / 255.0) as f64;
                    let a = value.powf(K_RCP_GAMMA) as f32;

                    buffer.push(Vec4::new(1.0, 1.0, 1.0, a));
                }

                pitch = bitmap.pitch() as usize;
            }

            /* Upload to texture. */
            unsafe {
                glyph.tex = Tex2D_Create(glyph.sx, glyph.sy, TexFormat_RGBA8);

                Tex2D_SetData(
                    &mut *glyph.tex,
                    buffer.as_ptr() as _,
                    PixelFormat_RGBA,
                    DataFormat_Float,
                );
            }

            /* Add to glyph cache. */
            font_data.glyphs.insert(code_point, glyph);
        }
    }

    fn get_kerning(&self, face: &freetype::Face, a: u32, b: u32) -> i32 {
        let kern = face
            .get_kerning(a, b, KerningMode::KerningDefault)
            .expect("Cannot get font face kerning");

        (kern.x >> 6) as i32
    }
}

#[luajit_ffi_gen::luajit_ffi(managed = true)]
impl Font {
    pub fn draw(&self, text: &str, mut x: f32, mut y: f32, r: f32, g: f32, b: f32, a: f32) {
        unsafe { Profiler_Begin(c_str!("Font_Draw")) };

        let mut glyph_last = 0;

        x = f64::floor(x as f64) as f32;
        y = f64::floor(y as f64) as f32;

        unsafe {
            RenderState_PushBlendMode(1);
            Draw_Color(r, g, b, a);
        }

        for c in text.chars() {
            let code_point = c as u32;

            self.get_glyph(code_point);

            let mut font_data = self.0.as_mut();
            let face = font_data.face.clone();
            let glyph = font_data.glyphs.get_mut(&code_point);

            if let Some(glyph) = glyph {
                if glyph_last != 0 {
                    x += self.get_kerning(&face, glyph_last, glyph.index) as f32;
                }

                let x0 = x + glyph.x0 as f32;
                let y0 = y + glyph.y0 as f32;
                let x1 = x + glyph.x1 as f32;
                let y1 = y + glyph.y1 as f32;

                Tex2D_DrawEx(
                    unsafe { &mut *glyph.tex },
                    x0,
                    y0,
                    x1,
                    y1,
                    0.0,
                    0.0,
                    1.0,
                    1.0,
                );

                x += glyph.advance as f32;
                glyph_last = glyph.index;
            } else {
                glyph_last = 0;
            }
        }

        unsafe {
            Draw_Color(1.0, 1.0, 1.0, 1.0);
            RenderState_PopBlendMode();

            Profiler_End();
        }
    }

    pub fn draw_shaded(&self, text: &str, mut x: f32, mut y: f32) {
        unsafe { Profiler_Begin(c_str!("Font_DrawShaded")) };

        let mut glyph_last = 0;

        x = f64::floor(x as f64) as f32;
        y = f64::floor(y as f64) as f32;

        for c in text.chars() {
            let code_point = c as u32;

            self.get_glyph(code_point);

            let mut font_data = self.0.as_mut();
            let face = font_data.face.clone();
            let glyph = font_data.glyphs.get_mut(&code_point);

            if let Some(glyph) = glyph {
                if glyph_last != 0 {
                    x += self.get_kerning(&face, glyph_last, glyph.index) as f32;
                }

                let x0 = x + glyph.x0 as f32;
                let y0 = y + glyph.y0 as f32;
                let x1 = x + glyph.x1 as f32;
                let y1 = y + glyph.y1 as f32;

                unsafe {
                    Shader_ResetTexIndex();
                    Shader_SetTex2D(c_str!("glyph"), &mut *glyph.tex);
                }

                Tex2D_DrawEx(
                    unsafe { &mut *glyph.tex },
                    x0,
                    y0,
                    x1,
                    y1,
                    0.0,
                    0.0,
                    1.0,
                    1.0,
                );

                x += glyph.advance as f32;
                glyph_last = glyph.index;
            } else {
                glyph_last = 0;
            }
        }

        unsafe { Profiler_End() };
    }

    pub fn get_line_height(&self) -> i32 {
        let font_data = self.0.as_ref();

        (font_data
            .face
            .size_metrics()
            .map(|sm| sm.height)
            .unwrap_or_default() // TODO: error?
            >> 6) as _
    }

    pub fn get_size(&self, text: &str, out: &mut IVec4) {
        unsafe { Profiler_Begin(c_str!("Font_GetSize")) };

        let mut x = 0;
        let y = 0;
        let mut lower = IVec2::new(i32::MAX, i32::MAX);
        let mut upper = IVec2::new(i32::MIN, i32::MIN);

        let mut glyph_last = 0;

        if text.is_empty() {
            *out = IVec4::ZERO;
            return;
        }

        for c in text.chars() {
            let code_point = c as u32;

            self.get_glyph(code_point);

            let mut font_data = self.0.as_mut();
            let face = font_data.face.clone();
            let glyph = font_data.glyphs.get_mut(&code_point);

            if let Some(glyph) = glyph {
                if glyph_last != 0 {
                    x += self.get_kerning(&face, glyph_last, glyph.index);
                }

                lower.x = i32::min(lower.x, x + glyph.x0);
                lower.y = i32::min(lower.y, y + glyph.y0);
                upper.x = i32::max(upper.x, x + glyph.x1);
                upper.y = i32::max(upper.y, y + glyph.y1);

                x += glyph.advance;
                glyph_last = glyph.index;
            } else {
                glyph_last = 0;
            }
        }

        *out = IVec4::new(lower.x, lower.y, upper.x - lower.x, upper.y - lower.y);

        unsafe { Profiler_End() };
    }

    /* NOTE : The height returned here is the maximal *ascender* height for the
     *        string. This allows easy centering of text while still allowing
     *        descending characters to look correct.
     *
     *        To correctly center text, first compute bounds via this function,
     *        then draw it at:
     *
     *           pos.x - (size.x - bound.x) / 2
     *           pos.y - (size.y + bound.y) / 2
     */

    pub fn get_size2(&self, text: &str) -> IVec2 {
        unsafe { Profiler_Begin(c_str!("Font_GetSize2")) };

        let mut res = IVec2::ZERO;
        let mut glyph_last = 0;

        for c in text.chars() {
            let code_point = c as u32;

            self.get_glyph(code_point);

            let mut font_data = self.0.as_mut();
            let face = font_data.face.clone();
            let glyph = font_data.glyphs.get_mut(&code_point);

            if let Some(glyph) = glyph {
                if glyph_last != 0 {
                    res.x += self.get_kerning(&face, glyph_last, glyph.index);
                }

                res.x += glyph.advance;
                res.y = i32::max(res.y, -glyph.y0 + 1);

                glyph_last = glyph.index;
            } else {
                glyph_last = 0;
            }
        }

        unsafe { Profiler_End() };

        res
    }
}
