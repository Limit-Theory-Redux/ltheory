#![allow(unsafe_code)] // TODO: remove

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::ptr::addr_of_mut;

use freetype_sys::{
    FT_Face, FT_Get_Char_Index, FT_Get_Kerning, FT_Init_FreeType, FT_KERNING_DEFAULT, FT_Library,
    FT_Load_Glyph, FT_New_Face, FT_Set_Pixel_Sizes, FT_Vector,
};
use glam::{IVec2, IVec4};

use super::{Color, Imm2DVertex, Renderer, Samplers, Shape, Tex2D, TexFormat};
use crate::rf::Rf;
use crate::system::{Profiler, Resource_GetPath, ResourceType};

/* TODO : Re-implement UTF-8 support */

/// Side of an atlas page, in texels (one 1024x1024 R8 texture per page).
const ATLAS_SIZE: u32 = 1024;
/// Empty texels around every glyph, so linear sampling of a scaled quad can
/// never reach a neighbour.
const ATLAS_PAD: u32 = 1;

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
    glyphs: HashMap<u32, Glyph>,
    /// The glyph atlas: pages are added as the current one fills up.
    pages: Vec<AtlasPage>,
}

/// One 1024x1024 R8 page of a font's glyph atlas, with a shelf packer. Glyphs
/// are rasterized into `cpu`; the rows touched since the last upload are
/// sent in one sub-rect update before the next draw.
struct AtlasPage {
    tex: Tex2D,
    cpu: Vec<u8>,
    shelves: Vec<Shelf>,
    /// First free row below the last shelf.
    next_y: u32,
    /// Rows (`start..end`) changed since the last upload.
    dirty: Option<(u32, u32)>,
}

#[derive(Clone, Copy)]
struct Shelf {
    y: u32,
    height: u32,
    /// First free column.
    x: u32,
}

impl AtlasPage {
    fn new(r: &mut Renderer) -> Self {
        let size = ATLAS_SIZE as usize;
        Self {
            tex: Tex2D::new_with_bytes(
                r,
                ATLAS_SIZE as i32,
                ATLAS_SIZE as i32,
                TexFormat::R8,
                vec![0; size * size],
            ),
            cpu: vec![0; size * size],
            shelves: Vec::new(),
            next_y: 0,
            dirty: None,
        }
    }

    /// Reserve a `w` x `h` slot (plus padding); `None` if the page is full.
    fn alloc(&mut self, w: u32, h: u32) -> Option<(u32, u32)> {
        let (pw, ph) = (w + ATLAS_PAD, h + ATLAS_PAD);
        if pw > ATLAS_SIZE || ph > ATLAS_SIZE {
            return None;
        }
        // The first shelf that is tall enough (and not absurdly taller than
        // the glyph) with room left in its row.
        for shelf in &mut self.shelves {
            if shelf.height >= ph && shelf.height <= ph + ph / 2 + 2 && shelf.x + pw <= ATLAS_SIZE {
                let at = (shelf.x, shelf.y);
                shelf.x += pw;
                return Some(at);
            }
        }
        if self.next_y + ph > ATLAS_SIZE {
            return None;
        }
        let at = (0, self.next_y);
        self.shelves.push(Shelf {
            y: self.next_y,
            height: ph,
            x: pw,
        });
        self.next_y += ph;
        Some(at)
    }

    fn mark_dirty(&mut self, y0: u32, y1: u32) {
        self.dirty = Some(match self.dirty {
            Some((a, b)) => (a.min(y0), b.max(y1)),
            None => (y0, y1),
        });
    }
}

#[derive(Clone)]
pub struct Glyph {
    pub index: i32,
    /// Atlas page and the glyph's top-left texel in it; `sx` x `sy` texels.
    /// Glyphs without a bitmap (spaces) have no page.
    pub page: Option<usize>,
    pub ax: u32,
    pub ay: u32,
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

    fn get_glyph(&self, r: &mut Renderer, code_point: u32) {
        let mut font_ref = self.0.as_mut();
        let font_data = &mut *font_ref;
        let face = font_data.handle;

        if !font_data.glyphs.contains_key(&code_point) {
            let glyph_index = unsafe { FT_Get_Char_Index(face, code_point as _) };

            if glyph_index == 0 {
                return;
            }

            unsafe {
                if FT_Load_Glyph(
                    face,
                    glyph_index,
                    (((1 as libc::c_long) << 5) | ((1 as libc::c_long) << 2)) as _,
                ) != 0
                {
                    return;
                }
            }

            let face_glyph = unsafe { &mut *(*face).glyph };
            let bitmap = &mut face_glyph.bitmap;

            /* Create a new glyph and fill out metrics. */
            let x0 = face_glyph.bitmap_left;
            let y0 = -face_glyph.bitmap_top;
            let sx = bitmap.width as i32;
            let sy = bitmap.rows as i32;
            let mut glyph = Glyph {
                index: glyph_index as _,
                page: None,
                ax: 0,
                ay: 0,
                x0,
                y0,
                sx,
                sy,
                x1: x0 + sx,
                y1: y0 + sy,
                advance: (face_glyph.advance.x >> 6) as _,
            };

            if sx > 0 && sy > 0 {
                /* Find room for it in the atlas. */
                let (w, h) = (sx as u32, sy as u32);
                let slot = font_data
                    .pages
                    .iter_mut()
                    .enumerate()
                    .find_map(|(i, page)| page.alloc(w, h).map(|at| (i, at)));
                let (page_index, (ax, ay)) = slot.unwrap_or_else(|| {
                    let mut page = AtlasPage::new(r);
                    let at = page.alloc(w, h).expect("glyph larger than an atlas page");
                    font_data.pages.push(page);
                    (font_data.pages.len() - 1, at)
                });

                /* Copy rendered bitmap (gamma corrected coverage) into the page. */
                let page = &mut font_data.pages[page_index];
                let mut p_bitmap = bitmap.buffer;
                for dy in 0..bitmap.rows {
                    let row = ((ay + dy as u32) * ATLAS_SIZE + ax) as usize;
                    for dx in 0..bitmap.width {
                        let value =
                            unsafe { (*p_bitmap.offset(dx as isize) as f32 / 255.0) as f64 };
                        let a = value.powf(K_RCP_GAMMA as f64) as f32;
                        page.cpu[row + dx as usize] = (a * 255.0 + 0.5) as u8;
                    }
                    p_bitmap = unsafe { p_bitmap.offset(bitmap.pitch as isize) };
                }
                page.mark_dirty(ay, ay + h);

                glyph.page = Some(page_index);
                glyph.ax = ax;
                glyph.ay = ay;
            }

            /* Add to glyph cache. */
            font_data.glyphs.insert(code_point, glyph);
        }
    }

    /// Send the rows of every page that changed since the last upload.
    fn flush_atlas(&self, r: &mut Renderer) {
        let mut font_data = self.0.as_mut();
        for page in &mut font_data.pages {
            let Some((y0, y1)) = page.dirty.take() else {
                continue;
            };
            let from = (y0 * ATLAS_SIZE) as usize;
            let to = (y1 * ATLAS_SIZE) as usize;
            page.tex.update_rect_bytes(
                r,
                0,
                y0 as i32,
                ATLAS_SIZE as i32,
                (y1 - y0) as i32,
                page.cpu[from..to].to_vec(),
            );
        }
    }

    fn draw_shape(
        &self,
        r: &mut Renderer,
        shape: Shape,
        text: &str,
        mut x: f32,
        mut y: f32,
        color: &Color,
    ) {
        Profiler::begin("Font_Draw");

        let mut glyph_last = 0;

        x = f64::floor(x as f64) as _;
        y = f64::floor(y as f64) as _;

        // Rasterize what is missing first, then upload the changed rows once.
        for c in text.chars() {
            self.get_glyph(r, c as u32);
        }
        self.flush_atlas(r);

        let c = [color.r, color.g, color.b, color.a];
        let inv = 1.0 / ATLAS_SIZE as f32;
        let mut runs: Vec<Vec<Imm2DVertex>> = Vec::new();

        {
            let font_data = self.0.as_ref();
            let face = font_data.handle;
            runs.resize_with(font_data.pages.len(), Vec::new);

            for ch in text.chars() {
                let Some(glyph) = font_data.glyphs.get(&(ch as u32)) else {
                    glyph_last = 0;
                    continue;
                };
                if glyph_last != 0 {
                    x += self.get_kerning(face, glyph_last, glyph.index) as f32;
                }

                if let Some(page) = glyph.page {
                    let x0 = x + glyph.x0 as f32;
                    let y0 = y + glyph.y0 as f32;
                    let (x1, y1) = (x0 + glyph.sx as f32, y0 + glyph.sy as f32);
                    let (u0, v0) = (glyph.ax as f32 * inv, glyph.ay as f32 * inv);
                    let (u1, v1) = (
                        (glyph.ax + glyph.sx as u32) as f32 * inv,
                        (glyph.ay + glyph.sy as u32) as f32 * inv,
                    );
                    let v = |px: f32, py: f32, u: f32, w: f32| Imm2DVertex {
                        pos: [px, py],
                        uv: [u, w],
                        color: c,
                        p: [0.0; 4],
                        q: [0.0; 4],
                    };
                    // The corners of `Draw.RectEx`, as two triangles of its fan.
                    let (a, b, cc, d) = (
                        v(x0, y0, u0, v0),
                        v(x0, y1, u0, v1),
                        v(x1, y1, u1, v1),
                        v(x1, y0, u1, v0),
                    );
                    runs[page].extend_from_slice(&[a, b, cc, a, cc, d]);
                }

                x += glyph.advance as f32;
                glyph_last = glyph.index;
            }
        }

        for (page, verts) in runs.iter().enumerate() {
            if verts.is_empty() {
                continue;
            }
            let view = self.0.as_ref().pages[page].tex.view();
            r.imm_textured_vertices(shape, view, Samplers::Point.id(), verts);
        }

        Profiler::end();
    }

    fn get_kerning(&self, face: FT_Face, a: i32, b: i32) -> i32 {
        let mut kern = FT_Vector { x: 0, y: 0 };

        unsafe {
            FT_Get_Kerning(
                face,
                a as _,
                b as _,
                FT_KERNING_DEFAULT as i32 as _,
                &mut kern,
            )
        };

        (kern.x >> 6) as _
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl Font {
    pub fn load(_r: &mut Renderer, name: &str, size: u32) -> Self {
        let handle = unsafe {
            if FT.is_null() {
                FT_Init_FreeType(addr_of_mut!(FT));
            }

            let name_cstr = CString::new(name).expect("Cannot convert string to C string");
            let path = Resource_GetPath(ResourceType::Font, name_cstr.as_ptr());
            let mut handle = std::ptr::null_mut();

            if FT_New_Face(FT, path, 0 as _, &mut handle) != 0 {
                panic!(
                    "Font_Load: Failed to load font <{name}> from file {:?}. Current folder: {:?}",
                    CStr::from_ptr(path),
                    std::env::current_dir()
                );
            }

            FT_Set_Pixel_Sizes(handle, 0 as _, size);

            handle
        };

        Self(
            FontData {
                name: name.into(),
                handle,
                glyphs: Default::default(),
                pages: Vec::new(),
            }
            .into(),
        )
    }

    /// Draw `text` with its baseline-relative top-left corner at `x, y`, alpha
    /// blended. Every glyph is a quad of the font's atlas: a string is one
    /// draw per atlas page it touches.
    pub fn draw(&self, r: &mut Renderer, text: &str, x: f32, y: f32, color: &Color) {
        self.draw_shape(r, Shape::Text, text, x, y, color);
    }

    pub fn get_line_height(&self) -> i32 {
        let font_data = self.0.as_ref();

        unsafe { ((*(*font_data.handle).size).metrics.height >> 6) as _ }
    }

    pub fn get_size(&self, r: &mut Renderer, text: &str, out: &mut IVec4) {
        Profiler::begin("Font_GetSize");

        let mut x = 0;
        let y = 0;
        let mut lower = IVec2::new(i32::MAX, i32::MAX);
        let mut upper = IVec2::new(i32::MIN, i32::MIN);

        let mut glyph_last = 0;

        if text.is_empty() {
            *out = IVec4::ZERO;
        } else {
            for c in text.chars() {
                let code_point = c as u32;

                self.get_glyph(r, code_point);

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
        }
        Profiler::end();
    }

    // NOTE : The height returned here is the maximal *ascender* height for the
    //        string. This allows easy centering of text while still allowing
    //        descending characters to look correct.
    //
    //        To correctly center text, first compute bounds via this function,
    //        then draw it at:
    //
    //           pos.x - (size.x - bound.x) / 2
    //           pos.y - (size.y + bound.y) / 2
    //

    pub fn get_size2(&self, r: &mut Renderer, text: &str) -> IVec2 {
        Profiler::begin("Font_GetSize2");

        let mut res = IVec2::ZERO;
        let mut glyph_last = 0;

        for c in text.chars() {
            let code_point = c as u32;
            self.get_glyph(r, code_point);

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
