use std::collections::HashMap;

use glam::{IVec2, IVec4};
use swash::scale::{Render, ScaleContext, Source};
use swash::shape::ShapeContext;
use swash::zeno::{Format, Vector};
use swash::{CacheKey, FontRef};

use super::{Color, Imm2DVertex, Renderer, Samplers, Shape, Tex2D, TexFormat};
use crate::rf::Rf;
use crate::system::{Profiler, Resource, ResourceType};

/// Side of an atlas page, in texels (one 1024x1024 R8 texture per page).
const ATLAS_SIZE: u32 = 1024;
/// Empty texels around every glyph, so linear sampling of a scaled quad can
/// never reach a neighbour.
const ATLAS_PAD: u32 = 1;

/// Glyphs are rasterized at this many horizontal subpixel positions (quarter
/// pixels), so a line keeps the spacing the font asks for instead of every
/// glyph snapping to a whole pixel.
const SUBPIXEL_STEPS: u32 = 4;

/// Coverage is gamma corrected with 1.8 (the value FreeType recommends, kept
/// from the FreeType renderer so text keeps its weight).
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
    /// The font file; with `offset` and `key` it makes a swash `FontRef`.
    data: Vec<u8>,
    offset: u32,
    key: CacheKey,
    /// Pixels per em.
    size: f32,
    /// The line box: whole pixels above and below the baseline (rounded up),
    /// from the font's own metrics.
    ascent: i32,
    descent: i32,
    /// Distance between the baselines of two lines.
    line_height: i32,
    shape_ctx: ShapeContext,
    scale_ctx: ScaleContext,
    /// Rasterized glyphs by glyph id and subpixel step.
    glyphs: HashMap<(u16, u8), Glyph>,
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

/// A rasterized glyph: its bitmap's offset from the pen position (`x0`, `y0`,
/// y down) and size, and where the bitmap sits in the atlas. Glyphs without
/// a bitmap (spaces) have no page.
#[derive(Clone, Copy)]
struct Glyph {
    page: Option<usize>,
    ax: u32,
    ay: u32,
    x0: i32,
    y0: i32,
    sx: i32,
    sy: i32,
}

/// One glyph of a shaped line, with its pen position relative to the start
/// of the line on the baseline, in pixels (y down).
#[derive(Clone, Copy)]
struct PlacedGlyph {
    id: u16,
    x: f32,
    y: f32,
}

struct ShapedLine {
    glyphs: Vec<PlacedGlyph>,
    /// Total advance width in pixels.
    width: f32,
}

/// Splits a pen x position into the whole pixel the glyph's bitmap is placed
/// from and the subpixel step it is rasterized at.
fn snap_x(x: f32) -> (f32, u8) {
    let base = x.floor();
    let step = ((x - base) * SUBPIXEL_STEPS as f32).round() as u32;
    if step == SUBPIXEL_STEPS {
        (base + 1.0, 0)
    } else {
        (base, step as u8)
    }
}

impl FontData {
    /// Lay out `text` as one line: glyph ids and positions, with the font's
    /// kerning (GPOS) and other default OpenType features applied. Characters
    /// the font has no glyph for are skipped, without an advance.
    fn shape(&mut self, text: &str) -> ShapedLine {
        let font = FontRef {
            data: &self.data,
            offset: self.offset,
            key: self.key,
        };
        let mut shaper = self.shape_ctx.builder(font).size(self.size).build();
        shaper.add_str(text);

        let mut glyphs = Vec::with_capacity(text.len());
        let mut x = 0.0;
        shaper.shape_with(|cluster| {
            for glyph in cluster.glyphs {
                if glyph.id == 0 {
                    continue;
                }
                // swash offsets are y up, the screen is y down.
                glyphs.push(PlacedGlyph {
                    id: glyph.id,
                    x: x + glyph.x,
                    y: -glyph.y,
                });
                x += glyph.advance;
            }
        });

        ShapedLine { glyphs, width: x }
    }

    /// The glyph `id` rasterized at subpixel `step`, added to the atlas the
    /// first time it is asked for.
    fn glyph(&mut self, r: &mut Renderer, id: u16, step: u8) -> Glyph {
        if let Some(glyph) = self.glyphs.get(&(id, step)) {
            return *glyph;
        }

        let image = {
            let font = FontRef {
                data: &self.data,
                offset: self.offset,
                key: self.key,
            };
            let mut scaler = self.scale_ctx.builder(font).size(self.size).build();
            Render::new(&[Source::Outline])
                .format(Format::Alpha)
                .offset(Vector::new(step as f32 / SUBPIXEL_STEPS as f32, 0.0))
                .render(&mut scaler, id)
        };

        let mut glyph = Glyph {
            page: None,
            ax: 0,
            ay: 0,
            x0: 0,
            y0: 0,
            sx: 0,
            sy: 0,
        };

        if let Some(image) = image {
            let (w, h) = (image.placement.width, image.placement.height);
            glyph.x0 = image.placement.left;
            glyph.y0 = -image.placement.top;
            glyph.sx = w as i32;
            glyph.sy = h as i32;

            if w > 0 && h > 0 {
                /* Find room for it in the atlas. */
                let slot = self
                    .pages
                    .iter_mut()
                    .enumerate()
                    .find_map(|(i, page)| page.alloc(w, h).map(|at| (i, at)));
                let (page_index, (ax, ay)) = slot.unwrap_or_else(|| {
                    let mut page = AtlasPage::new(r);
                    let at = page.alloc(w, h).expect("glyph larger than an atlas page");
                    self.pages.push(page);
                    (self.pages.len() - 1, at)
                });

                /* Copy the coverage (gamma corrected) into the page. */
                let page = &mut self.pages[page_index];
                for dy in 0..h {
                    let row = ((ay + dy) * ATLAS_SIZE + ax) as usize;
                    for dx in 0..w {
                        let value = (image.data[(dy * w + dx) as usize] as f32 / 255.0) as f64;
                        let a = value.powf(K_RCP_GAMMA as f64) as f32;
                        page.cpu[row + dx as usize] = (a * 255.0 + 0.5) as u8;
                    }
                }
                page.mark_dirty(ay, ay + h);

                glyph.page = Some(page_index);
                glyph.ax = ax;
                glyph.ay = ay;
            }
        }

        self.glyphs.insert((id, step), glyph);
        glyph
    }

    /// Send the rows of every page that changed since the last upload.
    fn flush_atlas(&mut self, r: &mut Renderer) {
        for page in &mut self.pages {
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
}

impl Font {
    pub fn name(&self) -> String {
        self.0.as_ref().name.clone()
    }

    /// Pixels the font's line box reaches above the baseline.
    pub fn ascent(&self) -> i32 {
        self.0.as_ref().ascent
    }

    /// Pixels the font's line box reaches below the baseline.
    pub fn descent(&self) -> i32 {
        self.0.as_ref().descent
    }

    /// Make a font of `size` pixels per em from the contents of a font file.
    pub(crate) fn from_bytes(name: &str, data: Vec<u8>, size: u32) -> Self {
        let size = size as f32;
        let (offset, key, ascent, descent, line_height) = {
            let font = FontRef::from_index(&data, 0)
                .unwrap_or_else(|| panic!("Font_Load: <{name}> is not a font file we can read"));
            let metrics = font.metrics(&[]).scale(size);
            (
                font.offset,
                font.key,
                metrics.ascent.ceil() as i32,
                metrics.descent.ceil() as i32,
                (metrics.ascent + metrics.descent + metrics.leading).round() as i32,
            )
        };

        Self(
            FontData {
                name: name.into(),
                data,
                offset,
                key,
                size,
                ascent,
                descent,
                line_height,
                shape_ctx: ShapeContext::new(),
                scale_ctx: ScaleContext::new(),
                glyphs: HashMap::new(),
                pages: Vec::new(),
            }
            .into(),
        )
    }

    fn draw_shape(
        &self,
        r: &mut Renderer,
        shape: Shape,
        text: &str,
        x: f32,
        y: f32,
        color: &Color,
    ) {
        Profiler::begin("Font_Draw");

        let x = x.floor();
        let y = y.floor();
        let c = [color.r, color.g, color.b, color.a];
        let inv = 1.0 / ATLAS_SIZE as f32;
        let mut runs: Vec<Vec<Imm2DVertex>> = Vec::new();

        {
            let mut font_data = self.0.as_mut();
            let line = font_data.shape(text);

            // Rasterize what is missing first, then upload the changed rows once.
            for placed in &line.glyphs {
                let (base_x, step) = snap_x(x + placed.x);
                let glyph = font_data.glyph(r, placed.id, step);
                let Some(page) = glyph.page else {
                    continue;
                };
                if runs.len() <= page {
                    runs.resize_with(page + 1, Vec::new);
                }

                let x0 = base_x + glyph.x0 as f32;
                let y0 = (y + placed.y).round() + glyph.y0 as f32;
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
            font_data.flush_atlas(r);
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
}

#[luajit_ffi_gen::luajit_ffi]
impl Font {
    pub fn load(_r: &mut Renderer, name: &str, size: u32) -> Self {
        let path = Resource::get_path(ResourceType::Font, name);
        let data = std::fs::read(&path).unwrap_or_else(|err| {
            panic!(
                "Font_Load: Failed to load font <{name}> from file {path:?}: {err}. Current folder: {:?}",
                std::env::current_dir()
            )
        });

        Self::from_bytes(name, data, size)
    }

    /// Draw `text` with its baseline-relative top-left corner at `x, y`, alpha
    /// blended. Every glyph is a quad of the font's atlas: a string is one
    /// draw per atlas page it touches.
    pub fn draw(&self, r: &mut Renderer, text: &str, x: f32, y: f32, color: &Color) {
        self.draw_shape(r, Shape::Text, text, x, y, color);
    }

    /// Distance between the baselines of two lines of this font.
    pub fn get_line_height(&self) -> i32 {
        self.0.as_ref().line_height
    }

    /// The box the glyphs of `text` actually cover (their "ink"), relative to
    /// the start of the line on the baseline: `(x, y, width, height)`, y down.
    pub fn get_size(&self, r: &mut Renderer, text: &str, out: &mut IVec4) {
        Profiler::begin("Font_GetSize");

        let mut font_data = self.0.as_mut();
        let line = font_data.shape(text);
        let mut lower = IVec2::new(i32::MAX, i32::MAX);
        let mut upper = IVec2::new(i32::MIN, i32::MIN);

        for placed in &line.glyphs {
            let (base_x, step) = snap_x(placed.x);
            let glyph = font_data.glyph(r, placed.id, step);
            let x0 = base_x as i32 + glyph.x0;
            let y0 = placed.y.round() as i32 + glyph.y0;

            lower.x = lower.x.min(x0);
            lower.y = lower.y.min(y0);
            upper.x = upper.x.max(x0 + glyph.sx);
            upper.y = upper.y.max(y0 + glyph.sy);
        }

        *out = if line.glyphs.is_empty() {
            IVec4::ZERO
        } else {
            IVec4::new(lower.x, lower.y, upper.x - lower.x, upper.y - lower.y)
        };

        Profiler::end();
    }

    /// The size of `text` as one line of layout: `x` is its advance width,
    /// `y` the font's line box (ascent + descent). The height is the same for
    /// every text of this font, so descenders fit and lines line up; the
    /// baseline is `ascent()` below the top of the box.
    pub fn get_size2(&self, _r: &mut Renderer, text: &str) -> IVec2 {
        Profiler::begin("Font_GetSize2");

        let mut font_data = self.0.as_mut();
        let width = font_data.shape(text).width.ceil() as i32;
        let size = IVec2::new(width, font_data.ascent + font_data.descent);

        Profiler::end();

        size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn font(name: &str, size: u32) -> Font {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../res/font")
            .join(format!("{name}.ttf"));
        Font::from_bytes(name, std::fs::read(path).expect("font file"), size)
    }

    #[test]
    fn line_box_holds_descenders() {
        let font = font("Exo2", 14);
        let mut r = Renderer::new_headless();

        let caps = font.get_size2(&mut r, "AAA");
        let tails = font.get_size2(&mut r, "gjpqy");
        assert_eq!(
            caps.y, tails.y,
            "every text of a font gets the same line box"
        );
        assert_eq!(caps.y, font.ascent() + font.descent());

        let mut ink = IVec4::ZERO;
        font.get_size(&mut r, "gjpqy", &mut ink);
        assert!(ink.y + ink.w > 0, "the tails reach below the baseline");
        assert!(
            ink.y + ink.w <= font.descent(),
            "the tails end inside the line box ({} > {})",
            ink.y + ink.w,
            font.descent()
        );

        font.get_size(&mut r, "AAA", &mut ink);
        assert!(-ink.y <= font.ascent(), "capitals fit above the baseline");
    }

    #[test]
    fn kerning_is_applied() {
        let font = font("Exo2", 32);
        let mut font_data = font.0.as_mut();

        let pair = font_data.shape("AV").width;
        let apart = font_data.shape("A").width + font_data.shape("V").width;
        assert!(
            pair < apart - 0.5,
            "the font's kerning pulls A and V together ({pair} vs {apart})"
        );
    }

    #[test]
    fn subpixel_steps_round_into_the_next_pixel() {
        assert_eq!(snap_x(3.0), (3.0, 0));
        assert_eq!(snap_x(3.3), (3.0, 1));
        assert_eq!(snap_x(3.5), (3.0, 2));
        assert_eq!(snap_x(3.9), (4.0, 0));
    }
}
