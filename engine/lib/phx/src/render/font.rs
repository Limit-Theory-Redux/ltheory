#![allow(unsafe_code)] // TODO: remove

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::ptr::addr_of_mut;

use freetype_sys::*;

use super::*;
use crate::math::*;
use crate::rf::Rf;
use crate::system::{Profiler, ResourceType, Resource_GetPath};

/* TODO : Re-implement UTF-8 support */
/* TODO : Atlas instead of individual textures. */

/* NOTE : Gamma of 1.8 recommended by FreeType */
const K_GAMMA: f32 = 1.8;
const K_RCP_GAMMA: f32 = 1.0 / K_GAMMA;

#[derive(Clone)]
pub struct Font(Rf<FontData>);

impl std::fmt::Debug for Font {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Font").field(&self.name()).finish()
    }
}

struct FontData {
    name: String,
    handle: FT_Face,
    shader: RefCell<Shader>,
    glyphs: HashMap<u32, Glyph>,
}

#[derive(Clone)]
pub struct Glyph {
    pub index: i32,
    pub tex: Tex2D,
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
    pub sx: i32,
    pub sy: i32,
    pub advance: i32,
}

static mut FT: FT_Library = std::ptr::null_mut();

impl Font {
    pub fn name(&self) -> String {
        let font_data = self.0.as_ref();

        font_data.name.clone()
    }

    fn get_glyph(&self, code_point: u32) {
        let mut font_data = self.0.as_mut();
        let face: FT_Face = font_data.handle;

        if let std::collections::hash_map::Entry::Vacant(e) = font_data.glyphs.entry(code_point) {
            let glyph_index = unsafe { FT_Get_Char_Index(face, code_point as FT_ULong) };

            if glyph_index == 0 {
                return;
            }

            unsafe {
                if FT_Load_Glyph(
                    face,
                    glyph_index,
                    (((1 as libc::c_long) << 5) | ((1 as libc::c_long) << 2)) as FT_Int32,
                ) != 0
                {
                    return;
                }
            }

            let face_glyph = unsafe { &mut *(*face).glyph };
            let bitmap = &mut face_glyph.bitmap;
            let mut p_bitmap = bitmap.buffer;

            /* Create a new glyph and fill out metrics. */
            let x0 = face_glyph.bitmap_left;
            let y0 = -face_glyph.bitmap_top;
            let sx = bitmap.width as i32;
            let sy = bitmap.rows as i32;
            let mut glyph = Glyph {
                index: glyph_index as _,
                tex: Tex2D::new(sx, sy, TexFormat_RGBA8),
                x0,
                y0,
                sx,
                sy,
                x1: x0 + sx,
                y1: y0 + sy,
                advance: (face_glyph.advance.x >> 6) as i32,
            };

            let mut buffer = Vec::with_capacity((glyph.sx * glyph.sy) as usize);

            /* Copy rendered bitmap into buffer. */
            for _dy in 0..(*bitmap).rows {
                for dx in 0..(*bitmap).width {
                    let value = unsafe { (*p_bitmap.offset(dx as isize) as f32 / 255.0) as f64 };
                    let a = value.powf(K_RCP_GAMMA as f64) as f32;

                    buffer.push(Vec4::new(1.0, 1.0, 1.0, a));
                }

                p_bitmap = unsafe { p_bitmap.offset(bitmap.pitch as isize) };
            }

            /* Upload to texture. */
            glyph
                .tex
                .set_data(buffer.as_slice(), PixelFormat::RGBA, DataFormat::Float);

            /* Add to glyph cache. */
            e.insert(glyph);
        }
    }

    fn get_kerning(&self, face: FT_Face, a: i32, b: i32) -> i32 {
        let mut kern: FT_Vector = FT_Vector { x: 0, y: 0 };

        unsafe {
            FT_Get_Kerning(
                face,
                a as FT_UInt,
                b as FT_UInt,
                FT_KERNING_DEFAULT as i32 as FT_UInt,
                &mut kern,
            )
        };

        (kern.x >> 6) as i32
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl Font {
    pub fn load(name: &str, size: u32) -> Self {
        let handle = unsafe {
            if FT.is_null() {
                FT_Init_FreeType(addr_of_mut!(FT));
            }

            let name_cstr = CString::new(name).expect("Cannot convert string to C string");
            let path = Resource_GetPath(ResourceType::Font, name_cstr.as_ptr());
            let mut handle = std::ptr::null_mut();

            if FT_New_Face(FT, path, 0 as FT_Long, &mut handle) != 0 {
                panic!(
                    "Font_Load: Failed to load font <{name}> from file {:?}. Current folder: {:?}",
                    CStr::from_ptr(path),
                    std::env::current_dir()
                );
            }

            FT_Set_Pixel_Sizes(handle, 0 as FT_UInt, size);

            handle
        };

        Self(
            FontData {
                name: name.into(),
                handle,
                shader: RefCell::new(Shader::load("vertex/ui", "fragment/ui/text")),
                glyphs: Default::default(),
            }
            .into(),
        )
    }

    pub fn draw(&self, text: &str, mut x: f32, mut y: f32, color: &Color) {
        Profiler::begin("Font_Draw");

        let mut glyph_last: i32 = 0;

        x = f64::floor(x as f64) as f32;
        y = f64::floor(y as f64) as f32;

        unsafe {
            RenderState_PushBlendMode(BlendMode::Alpha);
        }

        self.0.as_ref().shader.borrow_mut().start();
        self.0
            .as_ref()
            .shader
            .borrow_mut()
            .set_float4("color", color.r, color.g, color.b, color.a);

        for c in text.chars() {
            let code_point = c as u32;

            self.get_glyph(code_point);

            let font_data = self.0.as_ref();

            let face = font_data.handle;
            let glyph = font_data.glyphs.get(&code_point);
            let mut shader = font_data.shader.borrow_mut();

            if let Some(glyph) = glyph {
                if glyph_last != 0 {
                    x += self.get_kerning(face, glyph_last, glyph.index) as f32;
                }

                let x0: f32 = x + glyph.x0 as f32;
                let y0: f32 = y + glyph.y0 as f32;
                let xs: f32 = glyph.sx as f32;
                let ys: f32 = glyph.sy as f32;

                shader.reset_tex_index();
                shader.set_tex2d("glyph", &glyph.tex);

                Draw::rect_ex(x0, y0, xs, ys, 0.0, 0.0, 1.0, 1.0);

                x += glyph.advance as f32;
                glyph_last = glyph.index;
            } else {
                glyph_last = 0;
            }
        }

        self.0.as_ref().shader.borrow().stop();
        unsafe {
            RenderState_PopBlendMode();
        }
        Profiler::end();
    }

    pub fn get_line_height(&self) -> i32 {
        let font_data = self.0.as_ref();

        unsafe { ((*(*font_data.handle).size).metrics.height >> 6) as _ }
    }

    pub fn get_size(&self, text: &str, out: &mut IVec4) {
        Profiler::begin("Font_GetSize");

        let mut x: i32 = 0;
        let y: i32 = 0;
        let mut lower = IVec2::new(i32::MAX, i32::MAX);
        let mut upper = IVec2::new(i32::MIN, i32::MIN);

        let mut glyph_last: i32 = 0;

        if text.is_empty() {
            *out = IVec4::ZERO;
            return;
        }

        for c in text.chars() {
            let code_point = c as u32;

            self.get_glyph(code_point);

            let mut font_data = self.0.as_mut();
            let face = font_data.handle;
            let glyph = font_data.glyphs.get_mut(&code_point);

            if let Some(glyph) = glyph {
                if glyph_last != 0 {
                    x += self.get_kerning(face, glyph_last, glyph.index);
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

        Profiler::end();
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
        Profiler::begin("Font_GetSize2");

        let mut res = IVec2::ZERO;
        let mut glyph_last: i32 = 0;

        for c in text.chars() {
            let code_point = c as u32;
            self.get_glyph(code_point);

            let mut font_data = self.0.as_mut();
            let face = font_data.handle;
            let glyph = font_data.glyphs.get_mut(&code_point);

            if let Some(glyph) = glyph {
                if glyph_last != 0 {
                    res.x += self.get_kerning(face, glyph_last, glyph.index);
                }

                res.x += glyph.advance;
                res.y = i32::max(res.y, -glyph.y0 + 1);

                glyph_last = glyph.index;
            } else {
                glyph_last = 0;
            }
        }

        Profiler::end();

        res
    }
}
