use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use glam::{IVec2, IVec4};
use parley::fontique::Blob;
use parley::style::{FontFamily, FontStyle, FontWeight, FontWidth, StyleProperty};
use parley::{Layout, PositionedLayoutItem};
use swash::scale::image::Image;
use swash::scale::{Render, ScaleContext, Source};
use swash::zeno::{Format, Vector};
use swash::{CacheKey, FontRef};

use super::{Color, Imm2DVertex, Renderer, Samplers, Shape, Tex2D, TexFormat};
use crate::rf::Rf;
use crate::system::{Profiler, Resource, ResourceType};
use crate::ui::hmgui::{RegisteredFont, TEXT_CTX, TextContext, font_generation};

/// Side of a page of the plain atlas, in texels (one 1024x1024 R8 texture per
/// page).
const ATLAS_SIZE: u32 = 1024;
/// Side of a page of the outlined atlas, in texels: 512x512 RGBA8 is the same
/// 1 MiB as a plain page. A font is drawn outlined in a few HUD lines, not in
/// whole screens of text (and a glyph of up to ~500 px still fits).
const OUTLINED_ATLAS_SIZE: u32 = 512;
/// Empty texels around every glyph, so linear sampling of a scaled quad can
/// never reach a neighbour.
const ATLAS_PAD: u32 = 1;

/// Pixels the halo reaches past a glyph's ink.
const HALO_REACH: i32 = 3;
/// The halo is the glyph stamped at every whole-pixel offset up to this far,
/// each stamp darkening by [`HALO_STEP_ALPHA`] (the look picked with the Lua
/// prototype: 36 stamps, about 0.95 opaque right next to the ink).
const HALO_RADIUS: f32 = 3.2;
const HALO_STEP_ALPHA: f32 = 0.08;
/// The outline is the glyph stamped at the 8 neighbouring pixels, opaque.
const OUTLINE_RADIUS: f32 = 1.5;
/// The glow is the glyph's ink blurred with a Gaussian of [`GLOW_SIGMA`]
/// pixels, cut off at [`GLOW_REACH`], and brought up with
/// `1 - exp(-GLOW_GAIN * blur)`: thin strokes glow too, and the glow of a
/// solid area saturates instead of clipping.
const GLOW_SIGMA: f32 = 2.5;
const GLOW_REACH: i32 = 8;
const GLOW_GAIN: f32 = 4.0;
/// Pixels every glyph of the outlined atlas grows by on each side: room for
/// the halo and the glow, whichever reaches further.
const OUTLINED_MARGIN: i32 = if GLOW_REACH > HALO_REACH {
    GLOW_REACH
} else {
    HALO_REACH
};

/// Glyphs are rasterized at this many horizontal subpixel positions (quarter
/// pixels), so a line keeps the spacing the font asks for instead of every
/// glyph snapping to a whole pixel.
const SUBPIXEL_STEPS: u32 = 4;

/// Coverage is gamma corrected with 1.8 (the value FreeType recommends, kept
/// from the FreeType renderer so text keeps its weight).
const K_GAMMA: f32 = 1.8;
const K_RCP_GAMMA: f32 = 1.0 / K_GAMMA;

/// Engine frames so far, advanced by [`Font::next_frame`]. A font keeps the
/// lines it shaped for as long as they are asked for every frame.
static FRAME: AtomicU64 = AtomicU64::new(0);

/// The outline, halo and glow colours of a text drawn with
/// [`Font::draw_with_outline`]. A colour with alpha 0 leaves its part out.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextOutline {
    pub outline: Color,
    pub halo: Color,
    /// A soft light round the glyphs, under the halo: it shows past the
    /// halo's dark rim, so the text stays readable with it.
    pub glow: Color,
}

impl TextOutline {
    /// No outline, halo or glow.
    pub const NONE: TextOutline = TextOutline {
        outline: Color::TRANSPARENT,
        halo: Color::TRANSPARENT,
        glow: Color::TRANSPARENT,
    };
}

#[derive(Clone)]
pub struct Font(Rf<FontData>);

impl std::fmt::Debug for Font {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Font").field(&self.name()).finish()
    }
}

struct FontData {
    name: String,
    /// The family, width, style and weight that make parley pick this font's
    /// file (see [`RegisteredFont`]).
    family: String,
    width: FontWidth,
    style: FontStyle,
    weight: FontWeight,
    /// Pixels per em.
    size: f32,
    /// The line box: whole pixels above and below the baseline (rounded up),
    /// from the font's own metrics.
    ascent: i32,
    descent: i32,
    /// Distance between the baselines of two lines.
    line_height: i32,
    /// The font files glyphs were taken from: this font's own file first,
    /// then every fallback font parley picked for a character it lacks.
    faces: Vec<Face>,
    /// Reused for every line this font lays out.
    layout: Layout<Color>,
    /// Shaped lines by text. HmGui asks for the same texts every frame (and
    /// twice: to measure and to draw), so a line is shaped once and kept while
    /// it is used; one not asked for during a whole frame is dropped.
    lines: HashMap<String, CachedLine>,
    /// The frame `lines` was last swept in.
    lines_frame: u64,
    /// The text context's font generation the lines were shaped with: a font
    /// registered later can change which fallback font a character gets.
    lines_generation: u64,
    /// Texts broken into lines ([`Font::wrap`]) by text and width, kept and
    /// dropped like `lines`.
    wraps: HashMap<(String, u32), CachedWrap>,
    scale_ctx: ScaleContext,
    /// The glyphs [`Font::draw`] draws.
    plain: Atlas,
    /// The glyphs [`Font::draw_outlined`] draws, with their outline and halo.
    /// Empty (no pages) until the font is first drawn outlined.
    outlined: Atlas,
}

/// The two glyph atlases a font can have.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AtlasKind {
    /// R8 pages: the gamma corrected coverage (`fragment/ui/text`).
    Plain,
    /// RGBA8 pages, every glyph [`OUTLINED_MARGIN`] pixels bigger on each
    /// side: the coverage in red (as in the plain atlas), the 1 px outline in
    /// green, the halo in blue and the glow in alpha
    /// (`fragment/ui/text_outlined`).
    Outlined,
}

impl AtlasKind {
    fn page_size(self) -> u32 {
        match self {
            AtlasKind::Plain => ATLAS_SIZE,
            AtlasKind::Outlined => OUTLINED_ATLAS_SIZE,
        }
    }

    fn format(self) -> TexFormat {
        match self {
            AtlasKind::Plain => TexFormat::R8,
            AtlasKind::Outlined => TexFormat::RGBA8,
        }
    }

    fn texel_bytes(self) -> u32 {
        match self {
            AtlasKind::Plain => 1,
            AtlasKind::Outlined => 4,
        }
    }

    /// Pixels a glyph's bitmap grows by on each side.
    fn margin(self) -> i32 {
        match self {
            AtlasKind::Plain => 0,
            AtlasKind::Outlined => OUTLINED_MARGIN,
        }
    }

    fn shape(self) -> Shape {
        match self {
            AtlasKind::Plain => Shape::Text,
            AtlasKind::Outlined => Shape::TextOutlined,
        }
    }

    /// The texels of a glyph from its coverage bitmap (`w` x `h`, one byte
    /// per pixel), `margin()` pixels bigger on each side.
    fn texels(self, coverage: &[u8], w: u32, h: u32) -> Vec<u8> {
        match self {
            AtlasKind::Plain => coverage.iter().map(|&c| gamma_coverage(c)).collect(),
            AtlasKind::Outlined => outlined_texels(coverage, w, h),
        }
    }
}

/// Coverage as the atlas stores it: gamma corrected with [`K_GAMMA`].
fn gamma_coverage(coverage: u8) -> u8 {
    let value = (coverage as f32 / 255.0) as f64;
    let a = value.powf(K_RCP_GAMMA as f64) as f32;
    (a * 255.0 + 0.5) as u8
}

/// The whole-pixel offsets within `radius` of a pixel, the pixel itself left
/// out.
fn stamp_offsets(radius: f32) -> Vec<(i32, i32)> {
    let reach = radius as i32;
    let mut offsets = Vec::new();
    for dy in -reach..=reach {
        for dx in -reach..=reach {
            let d2 = (dx * dx + dy * dy) as f32;
            if d2 > 0.0 && d2 <= radius * radius {
                offsets.push((dx, dy));
            }
        }
    }
    offsets
}

/// The RGBA texels of a glyph in the outlined atlas, [`OUTLINED_MARGIN`]
/// pixels bigger on each side than its coverage bitmap (`w` x `h`).
///
/// Red is the coverage exactly as the plain atlas stores it, so the text
/// itself looks the same. Green (outline) and blue (halo) are what the Lua
/// prototype drew with extra draws of the text at small offsets ("stamps"),
/// added up once here: the outline is the glyph at the 8 neighbouring pixels,
/// opaque; the halo is the glyph at every offset within [`HALO_RADIUS`], each
/// at [`HALO_STEP_ALPHA`]. A stamp covers what `text.glsl` draws of a pixel:
/// the square root of its red. Alpha (glow) is that ink blurred (see
/// [`GLOW_SIGMA`]).
fn outlined_texels(coverage: &[u8], w: u32, h: u32) -> Vec<u8> {
    let m = OUTLINED_MARGIN;
    let (w, h) = (w as i32, h as i32);
    let (sw, sh) = (w + 2 * m, h + 2 * m);

    let mut red = vec![0u8; (sw * sh) as usize];
    let mut ink = vec![0.0f32; (sw * sh) as usize];
    for y in 0..h {
        for x in 0..w {
            let r = gamma_coverage(coverage[(y * w + x) as usize]);
            let i = ((y + m) * sw + x + m) as usize;
            red[i] = r;
            ink[i] = (r as f32 / 255.0).sqrt();
        }
    }
    let ink_at = |x: i32, y: i32| {
        if x < 0 || y < 0 || x >= sw || y >= sh {
            0.0
        } else {
            ink[(y * sw + x) as usize]
        }
    };

    let outline = stamp_offsets(OUTLINE_RADIUS);
    let halo = stamp_offsets(HALO_RADIUS);
    let to_byte = |a: f32| (a.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;

    // The glow: the ink blurred with a normalized Gaussian, one pass per axis.
    let kernel: Vec<f32> = (-GLOW_REACH..=GLOW_REACH)
        .map(|d| (-((d * d) as f32) / (2.0 * GLOW_SIGMA * GLOW_SIGMA)).exp())
        .collect();
    let kernel_sum: f32 = kernel.iter().sum();
    let blur = |src: &[f32], (sx, sy): (i32, i32)| -> Vec<f32> {
        let mut out = vec![0.0f32; src.len()];
        for y in 0..sh {
            for x in 0..sw {
                let mut sum = 0.0;
                for (k, weight) in kernel.iter().enumerate() {
                    let d = k as i32 - GLOW_REACH;
                    let (px, py) = (x + d * sx, y + d * sy);
                    if px >= 0 && py >= 0 && px < sw && py < sh {
                        sum += weight * src[(py * sw + px) as usize];
                    }
                }
                out[(y * sw + x) as usize] = sum / kernel_sum;
            }
        }
        out
    };
    let glow = blur(&blur(&ink, (1, 0)), (0, 1));

    let mut texels = vec![0u8; (sw * sh * 4) as usize];
    for y in 0..sh {
        for x in 0..sw {
            // What shows through all the stamps, multiplied up like the
            // blending of separate draws would.
            let outline_clear: f32 = outline
                .iter()
                .map(|&(dx, dy)| 1.0 - ink_at(x - dx, y - dy))
                .product();
            let halo_clear: f32 = halo
                .iter()
                .map(|&(dx, dy)| 1.0 - HALO_STEP_ALPHA * ink_at(x - dx, y - dy))
                .product();

            let i = (y * sw + x) as usize;
            texels[i * 4] = red[i];
            texels[i * 4 + 1] = to_byte(1.0 - outline_clear);
            texels[i * 4 + 2] = to_byte(1.0 - halo_clear);
            texels[i * 4 + 3] = to_byte(1.0 - (-GLOW_GAIN * glow[i]).exp());
        }
    }
    texels
}

/// The rasterized glyphs of one atlas kind and the pages holding them.
struct Atlas {
    kind: AtlasKind,
    /// Rasterized glyphs by face, glyph id and subpixel step.
    glyphs: HashMap<(u16, u16, u8), Glyph>,
    /// Pages are added as the current one fills up.
    pages: Vec<AtlasPage>,
}

impl Atlas {
    fn new(kind: AtlasKind) -> Self {
        Self {
            kind,
            glyphs: HashMap::new(),
            pages: Vec::new(),
        }
    }

    /// Put the glyph rasterized as `image` into the atlas (`None`: nothing to
    /// draw, like a space).
    fn add(&mut self, r: &mut Renderer, image: Option<&Image>) -> Glyph {
        let mut glyph = Glyph {
            page: None,
            ax: 0,
            ay: 0,
            x0: 0,
            y0: 0,
            sx: 0,
            sy: 0,
        };
        let Some(image) = image else {
            return glyph;
        };

        let (w, h) = (image.placement.width, image.placement.height);
        glyph.x0 = image.placement.left;
        glyph.y0 = -image.placement.top;
        glyph.sx = w as i32;
        glyph.sy = h as i32;
        if w == 0 || h == 0 {
            return glyph;
        }

        let m = self.kind.margin();
        glyph.x0 -= m;
        glyph.y0 -= m;
        glyph.sx += 2 * m;
        glyph.sy += 2 * m;
        let (sw, sh) = (glyph.sx as u32, glyph.sy as u32);
        let texels = self.kind.texels(&image.data, w, h);

        /* Find room for it in the atlas. */
        let kind = self.kind;
        let slot = self
            .pages
            .iter_mut()
            .enumerate()
            .find_map(|(i, page)| page.alloc(sw, sh).map(|at| (i, at)));
        let (page_index, (ax, ay)) = slot.unwrap_or_else(|| {
            let mut page = AtlasPage::new(r, kind);
            let at = page.alloc(sw, sh).expect("glyph larger than an atlas page");
            self.pages.push(page);
            (self.pages.len() - 1, at)
        });

        /* Copy its rows into the page. */
        let page = &mut self.pages[page_index];
        let bytes = kind.texel_bytes() as usize;
        let row_bytes = sw as usize * bytes;
        for dy in 0..sh {
            let at = (((ay + dy) * page.size + ax) as usize) * bytes;
            let from = dy as usize * row_bytes;
            page.cpu[at..at + row_bytes].copy_from_slice(&texels[from..from + row_bytes]);
        }
        page.mark_dirty(ay, ay + sh);

        glyph.page = Some(page_index);
        glyph.ax = ax;
        glyph.ay = ay;
        glyph
    }

    /// Send the rows of every page that changed since the last upload.
    fn flush(&mut self, r: &mut Renderer) {
        let bytes = self.kind.texel_bytes();
        for page in &mut self.pages {
            let Some((y0, y1)) = page.dirty.take() else {
                continue;
            };
            let from = (y0 * page.size * bytes) as usize;
            let to = (y1 * page.size * bytes) as usize;
            page.tex.update_rect_bytes(
                r,
                0,
                y0 as i32,
                page.size as i32,
                (y1 - y0) as i32,
                page.cpu[from..to].to_vec(),
            );
        }
    }
}

/// One page of a font's glyph atlas, with a shelf packer. Glyphs are
/// rasterized into `cpu`; the rows touched since the last upload are sent in
/// one sub-rect update before the next draw.
struct AtlasPage {
    tex: Tex2D,
    /// Side in texels.
    size: u32,
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
    fn new(r: &mut Renderer, kind: AtlasKind) -> Self {
        let size = kind.page_size();
        let bytes = (size * size * kind.texel_bytes()) as usize;
        Self {
            tex: Tex2D::new_with_bytes(r, size as i32, size as i32, kind.format(), vec![0; bytes]),
            size,
            cpu: vec![0; bytes],
            shelves: Vec::new(),
            next_y: 0,
            dirty: None,
        }
    }

    /// Reserve a `w` x `h` slot (plus padding); `None` if the page is full.
    fn alloc(&mut self, w: u32, h: u32) -> Option<(u32, u32)> {
        let (pw, ph) = (w + ATLAS_PAD, h + ATLAS_PAD);
        if pw > self.size || ph > self.size {
            return None;
        }
        // The first shelf that is tall enough (and not absurdly taller than
        // the glyph) with room left in its row.
        for shelf in &mut self.shelves {
            if shelf.height >= ph && shelf.height <= ph + ph / 2 + 2 && shelf.x + pw <= self.size {
                let at = (shelf.x, shelf.y);
                shelf.x += pw;
                return Some(at);
            }
        }
        if self.next_y + ph > self.size {
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

/// A font file glyphs come from, as swash needs it to rasterize them.
struct Face {
    blob: Blob<u8>,
    /// Index of the font in its file (a collection holds several).
    index: u32,
    offset: u32,
    key: CacheKey,
}

impl Face {
    fn new(blob: Blob<u8>, index: u32) -> Option<Self> {
        let font = FontRef::from_index(blob.data(), index as usize)?;
        let (offset, key) = (font.offset, font.key);
        Some(Self {
            blob,
            index,
            offset,
            key,
        })
    }

    fn font_ref(&self) -> FontRef<'_> {
        FontRef {
            data: self.blob.data(),
            offset: self.offset,
            key: self.key,
        }
    }
}

/// The index of parley's `font` in `faces`, added the first time it is used.
/// `None` if swash can't read it.
fn face_index(faces: &mut Vec<Face>, font: &parley::FontData) -> Option<u16> {
    let found = faces
        .iter()
        .position(|face| face.blob.id() == font.data.id() && face.index == font.index);
    let index = match found {
        Some(index) => index,
        None => {
            faces.push(Face::new(font.data.clone(), font.index)?);
            faces.len() - 1
        }
    };
    u16::try_from(index).ok()
}

/// One glyph of a shaped line, with its pen position relative to the start
/// of the line on the baseline, in pixels (y down).
#[derive(Clone, Copy)]
struct PlacedGlyph {
    /// Index into the font's faces.
    face: u16,
    id: u16,
    x: f32,
    y: f32,
}

struct ShapedLine {
    glyphs: Vec<PlacedGlyph>,
    /// Total advance width in pixels.
    width: f32,
}

/// A shaped line in a font's cache, with the last frame it was asked for.
struct CachedLine {
    line: Rc<ShapedLine>,
    frame: u64,
}

/// A text broken into lines in a font's cache, with the last frame it was
/// asked for.
struct CachedWrap {
    lines: Rc<Vec<String>>,
    frame: u64,
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
    /// `text` laid out as one line: from the cache, or shaped with parley in
    /// the shared text context.
    fn shape(&mut self, text: &str) -> Rc<ShapedLine> {
        self.shape_at(text, FRAME.load(Ordering::Relaxed), font_generation())
    }

    /// [`Self::shape`] in engine frame `frame`, with the text context at font
    /// generation `generation`. Lines last asked for before the previous frame
    /// are dropped first (once per frame), and all lines when the generation
    /// changed.
    fn shape_at(&mut self, text: &str, frame: u64, generation: u64) -> Rc<ShapedLine> {
        self.sweep(frame, generation);

        if let Some(cached) = self.lines.get_mut(text) {
            cached.frame = frame;
            return cached.line.clone();
        }

        let line = {
            let mut text_ctx = TEXT_CTX.lock().expect("Cannot use text context");
            Rc::new(self.shape_in(&mut text_ctx, text))
        };
        self.lines.insert(
            text.to_owned(),
            CachedLine {
                line: line.clone(),
                frame,
            },
        );
        line
    }

    /// Drop the cached lines and wraps last asked for before the previous frame
    /// (once per frame), and all of them when the font generation changed.
    fn sweep(&mut self, frame: u64, generation: u64) {
        if generation != self.lines_generation {
            self.lines.clear();
            self.wraps.clear();
            self.lines_generation = generation;
        }
        if frame != self.lines_frame {
            self.lines
                .retain(|_, cached| cached.frame.saturating_add(1) >= frame);
            self.wraps
                .retain(|_, cached| cached.frame.saturating_add(1) >= frame);
            self.lines_frame = frame;
        }
    }

    /// `text` broken into lines: at its line breaks, and where a line would be
    /// wider than `max_width`, at the places Unicode allows (between words; a
    /// word wider than that stays whole). Each line without its trailing white
    /// space. From the cache, or laid out with parley.
    fn wrap(&mut self, text: &str, max_width: Option<f32>) -> Rc<Vec<String>> {
        let frame = FRAME.load(Ordering::Relaxed);
        self.sweep(frame, font_generation());

        let key = (text.to_owned(), max_width.map_or(u32::MAX, f32::to_bits));
        if let Some(cached) = self.wraps.get_mut(&key) {
            cached.frame = frame;
            return cached.lines.clone();
        }

        let mut lines: Vec<String> = {
            let mut text_ctx = TEXT_CTX.lock().expect("Cannot use text context");
            self.build_layout(&mut text_ctx, text, max_width);
            self.layout
                .lines()
                .map(|line| text[line.text_range()].trim_end().to_owned())
                .collect()
        };
        if lines.is_empty() {
            lines.push(String::new());
        }

        let lines = Rc::new(lines);
        self.wraps.insert(
            key,
            CachedWrap {
                lines: lines.clone(),
                frame,
            },
        );
        lines
    }

    /// Lay `text` out in this font with parley, into `self.layout`, its lines
    /// broken at line breaks and where they would be wider than `max_width`.
    fn build_layout(&mut self, text_ctx: &mut TextContext, text: &str, max_width: Option<f32>) {
        let TextContext { font, layout, .. } = text_ctx;
        let mut builder = layout.ranged_builder(font, text, 1.0, false);
        builder.push_default(FontFamily::named(&self.family));
        builder.push_default(StyleProperty::FontSize(self.size));
        builder.push_default(StyleProperty::FontWidth(self.width));
        builder.push_default(StyleProperty::FontStyle(self.style));
        builder.push_default(StyleProperty::FontWeight(self.weight));
        builder.build_into(&mut self.layout, text);
        self.layout.break_all_lines(max_width);
    }

    /// Lay out `text` as one line with parley: glyph ids and positions, with
    /// the font's kerning (GPOS) and other default OpenType features applied.
    /// A character this font lacks comes from the fallback font (DejaVu Sans);
    /// one that no font has (the `.notdef` glyph) keeps its advance but isn't
    /// drawn. Line breaks are dropped: this text is always one line.
    fn shape_in(&mut self, text_ctx: &mut TextContext, text: &str) -> ShapedLine {
        let text: Cow<'_, str> = if text.contains(['\n', '\r']) {
            text.replace(['\n', '\r'], "").into()
        } else {
            text.into()
        };

        self.build_layout(text_ctx, &text, None);

        let mut glyphs = Vec::with_capacity(text.len());
        if let Some(line) = self.layout.lines().next() {
            let baseline = line.metrics().baseline;
            for item in line.items() {
                let PositionedLayoutItem::GlyphRun(run) = item else {
                    continue;
                };
                let Some(face) = face_index(&mut self.faces, run.run().font()) else {
                    continue;
                };
                for glyph in run.positioned_glyphs() {
                    if glyph.id == 0 {
                        continue;
                    }
                    glyphs.push(PlacedGlyph {
                        face,
                        id: glyph.id as u16,
                        x: glyph.x,
                        y: glyph.y - baseline,
                    });
                }
            }
        }

        ShapedLine {
            glyphs,
            width: self.layout.full_width(),
        }
    }

    fn atlas(&mut self, kind: AtlasKind) -> &mut Atlas {
        match kind {
            AtlasKind::Plain => &mut self.plain,
            AtlasKind::Outlined => &mut self.outlined,
        }
    }

    /// The glyph `id` of `face` rasterized at subpixel `step` in the atlas of
    /// `kind`, added to it the first time it is asked for.
    fn glyph(&mut self, r: &mut Renderer, kind: AtlasKind, face: u16, id: u16, step: u8) -> Glyph {
        if let Some(glyph) = self.atlas(kind).glyphs.get(&(face, id, step)) {
            return *glyph;
        }

        let image = {
            let font = self.faces[face as usize].font_ref();
            let mut scaler = self.scale_ctx.builder(font).size(self.size).build();
            Render::new(&[Source::Outline])
                .format(Format::Alpha)
                .offset(Vector::new(step as f32 / SUBPIXEL_STEPS as f32, 0.0))
                .render(&mut scaler, id)
        };

        let atlas = self.atlas(kind);
        let glyph = atlas.add(r, image.as_ref());
        atlas.glyphs.insert((face, id, step), glyph);
        glyph
    }
}

impl Font {
    pub fn name(&self) -> String {
        self.0.as_ref().name.clone()
    }

    /// Pixels per em: the size the font was loaded at.
    pub fn size(&self) -> f32 {
        self.0.as_ref().size
    }

    /// `text` broken into lines: at its line breaks (`\n`), and where a line
    /// would be wider than `max_width` pixels, at the places Unicode allows
    /// (between words; a word wider than that stays whole). Each line without
    /// its trailing white space; at least one line.
    pub fn wrap(&self, text: &str, max_width: Option<f32>) -> Rc<Vec<String>> {
        self.0.as_mut().wrap(text, max_width)
    }

    /// Pixels the font's line box reaches above the baseline.
    pub fn ascent(&self) -> i32 {
        self.0.as_ref().ascent
    }

    /// Pixels the font's line box reaches below the baseline.
    pub fn descent(&self) -> i32 {
        self.0.as_ref().descent
    }

    /// Start a new engine frame for the fonts' line caches: a line not asked
    /// for during the frame that just ended is dropped next time its font is
    /// used. Called once per frame by the main loop.
    pub fn next_frame() {
        FRAME.fetch_add(1, Ordering::Relaxed);
    }

    /// Make a font of `size` pixels per em from the font file at `path`,
    /// registered with the shared text context.
    pub(crate) fn from_file(name: &str, path: &Path, size: u32) -> Self {
        let registered = TEXT_CTX
            .lock()
            .expect("Cannot use text context")
            .register_font_file(path)
            .unwrap_or_else(|| {
                panic!(
                    "Font_Load: Failed to load font <{name}> from file {path:?}. Current folder: {:?}",
                    std::env::current_dir()
                )
            });

        Self::from_registered(name, &registered, size)
    }

    /// Make a font of `size` pixels per em from a font file registered with
    /// the text context. Its line box comes from the file's own metrics.
    fn from_registered(name: &str, registered: &RegisteredFont, size: u32) -> Self {
        let size = size as f32;
        let face = Face::new(registered.blob.clone(), 0)
            .unwrap_or_else(|| panic!("Font_Load: <{name}> is not a font file we can read"));
        let metrics = face.font_ref().metrics(&[]).scale(size);

        Self(
            FontData {
                name: name.into(),
                family: registered.family.clone(),
                width: registered.width,
                style: registered.style,
                weight: registered.weight,
                size,
                ascent: metrics.ascent.ceil() as i32,
                descent: metrics.descent.ceil() as i32,
                line_height: (metrics.ascent + metrics.descent + metrics.leading).round() as i32,
                faces: vec![face],
                layout: Layout::new(),
                lines: HashMap::new(),
                lines_frame: FRAME.load(Ordering::Relaxed),
                lines_generation: font_generation(),
                wraps: HashMap::new(),
                scale_ctx: ScaleContext::new(),
                plain: Atlas::new(AtlasKind::Plain),
                outlined: Atlas::new(AtlasKind::Outlined),
            }
            .into(),
        )
    }

    /// Draw `text` like [`Self::draw_outlined`], with `outline`'s glow as
    /// well. Glow, then halo and outline, then the text: every glyph of one
    /// before any of the next, so no glyph's glow covers a neighbour's halo
    /// and no halo a neighbour's text. The text's alpha fades all three.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_with_outline(
        &self,
        r: &mut Renderer,
        text: &str,
        x: f32,
        y: f32,
        color: &Color,
        outline: &TextOutline,
    ) {
        let fade = color.a;
        let faded = |c: &Color| [c.r, c.g, c.b, c.a * fade];
        let (glow, line, halo) = (
            faded(&outline.glow),
            faded(&outline.outline),
            faded(&outline.halo),
        );
        let none = [0.0; 4];

        let mut passes = Vec::with_capacity(3);
        if glow[3] > 0.0 {
            passes.push((none, none, none, glow));
        }
        if line[3] > 0.0 || halo[3] > 0.0 {
            passes.push((none, line, halo, none));
        }
        passes.push(([color.r, color.g, color.b, color.a], none, none, none));
        self.draw_atlas(r, AtlasKind::Outlined, text, x, y, &passes);
    }

    /// Draw `text` with the glyphs of the atlas of `kind`, once per pass:
    /// every glyph of a pass before any of the next, each pass with its own
    /// vertex `color`, `p`, `q` and `r`.
    fn draw_atlas(
        &self,
        r: &mut Renderer,
        kind: AtlasKind,
        text: &str,
        x: f32,
        y: f32,
        passes: &[([f32; 4], [f32; 4], [f32; 4], [f32; 4])],
    ) {
        Profiler::begin("Font_Draw");

        let x = x.floor();
        let y = y.floor();
        let inv = 1.0 / kind.page_size() as f32;
        // Glyph quads by atlas page: corners and texture coordinates.
        let mut quads: Vec<Vec<[f32; 8]>> = Vec::new();

        {
            let mut font_data = self.0.as_mut();
            let line = font_data.shape(text);

            // Rasterize what is missing first, then upload the changed rows once.
            for placed in &line.glyphs {
                let (base_x, step) = snap_x(x + placed.x);
                let glyph = font_data.glyph(r, kind, placed.face, placed.id, step);
                let Some(page) = glyph.page else {
                    continue;
                };
                if quads.len() <= page {
                    quads.resize_with(page + 1, Vec::new);
                }

                let x0 = base_x + glyph.x0 as f32;
                let y0 = (y + placed.y).round() + glyph.y0 as f32;
                let (x1, y1) = (x0 + glyph.sx as f32, y0 + glyph.sy as f32);
                let (u0, v0) = (glyph.ax as f32 * inv, glyph.ay as f32 * inv);
                let (u1, v1) = (
                    (glyph.ax + glyph.sx as u32) as f32 * inv,
                    (glyph.ay + glyph.sy as u32) as f32 * inv,
                );
                quads[page].push([x0, y0, x1, y1, u0, v0, u1, v1]);
            }
            font_data.atlas(kind).flush(r);
        }

        let mut verts = Vec::new();
        for &(color, p, q, pass_r) in passes {
            for (page, page_quads) in quads.iter().enumerate() {
                if page_quads.is_empty() {
                    continue;
                }
                verts.clear();
                for &[x0, y0, x1, y1, u0, v0, u1, v1] in page_quads {
                    let v = |px: f32, py: f32, u: f32, w: f32| Imm2DVertex {
                        pos: [px, py],
                        uv: [u, w],
                        color,
                        p,
                        q,
                        r: pass_r,
                    };
                    // The corners of `Draw.RectEx`, as two triangles of its fan.
                    let (a, b, c, d) = (
                        v(x0, y0, u0, v0),
                        v(x0, y1, u0, v1),
                        v(x1, y1, u1, v1),
                        v(x1, y0, u1, v0),
                    );
                    verts.extend_from_slice(&[a, b, c, a, c, d]);
                }
                let view = self.0.as_mut().atlas(kind).pages[page].tex.view();
                r.imm_textured_vertices(kind.shape(), view, Samplers::Point.id(), &verts);
            }
        }

        Profiler::end();
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl Font {
    pub fn load(_r: &mut Renderer, name: &str, size: u32) -> Self {
        let path = Resource::get_path(ResourceType::Font, name);
        Self::from_file(name, Path::new(&path), size)
    }

    /// Draw `text` with its baseline-relative top-left corner at `x, y`, alpha
    /// blended. Every glyph is a quad of the font's atlas: a string is one
    /// draw per atlas page it touches.
    pub fn draw(&self, r: &mut Renderer, text: &str, x: f32, y: f32, color: &Color) {
        let color = [color.r, color.g, color.b, color.a];
        self.draw_atlas(
            r,
            AtlasKind::Plain,
            text,
            x,
            y,
            &[(color, [0.0; 4], [0.0; 4], [0.0; 4])],
        );
    }

    /// Draw `text` like [`Self::draw`], with a 1 px `outline` and a 3 px
    /// `halo` around its glyphs, for text that has to stay readable on any
    /// background. Both are baked into a second atlas of this font the first
    /// time a glyph is drawn outlined, so nothing is added up at run time:
    /// the glyphs are drawn twice, outline and halo of every glyph first, the
    /// text on top (so a glyph's halo never covers its neighbour), both from
    /// the same atlas page in one draw.
    ///
    /// A colour with alpha 0 leaves its part out (a transparent outline gives
    /// the halo alone); a halo alpha of 1 is the halo's full strength, about
    /// 0.95 opaque next to the glyphs. The text's alpha fades outline and
    /// halo with it. With both left out the text looks exactly as `draw`
    /// draws it. (The same atlas holds a glow: see
    /// [`Self::draw_with_outline`].)
    #[allow(clippy::too_many_arguments)]
    pub fn draw_outlined(
        &self,
        r: &mut Renderer,
        text: &str,
        x: f32,
        y: f32,
        color: &Color,
        outline: &Color,
        halo: &Color,
    ) {
        let outline = TextOutline {
            outline: *outline,
            halo: *halo,
            ..TextOutline::NONE
        };
        self.draw_with_outline(r, text, x, y, color, &outline);
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
            let glyph = font_data.glyph(r, AtlasKind::Plain, placed.face, placed.id, step);
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
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../res/font")
            .join(format!("{name}.ttf"));
        Font::from_file(name, &path, size)
    }

    #[test]
    fn wrap_breaks_at_line_breaks_and_between_words() {
        let font = font("Exo2", 14);
        let mut r = Renderer::new_headless();

        assert_eq!(*font.wrap("one\ntwo", None), vec!["one", "two"]);
        assert_eq!(*font.wrap("no break", None), vec!["no break"]);

        let width = font.get_size2(&mut r, "one two").x as f32;
        assert_eq!(
            *font.wrap("one two three", Some(width + 1.0)),
            vec!["one two", "three"]
        );
        // a word wider than the width stays whole
        assert_eq!(*font.wrap("unbreakable", Some(5.0)), vec!["unbreakable"]);
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
    fn missing_characters_come_from_the_fallback_font() {
        let font = font("Exo2", 16);
        let mut font_data = font.0.as_mut();

        let latin = font_data.shape("Exo");
        assert!(latin.glyphs.iter().all(|glyph| glyph.face == 0));

        // Exo 2 has no Hebrew: before parley these glyphs were dropped.
        let hebrew = font_data.shape("\u{5e9}\u{5dc}\u{5d5}\u{5dd}");
        assert_eq!(hebrew.glyphs.len(), 4, "every character is drawn");
        assert!(
            hebrew.glyphs.iter().all(|glyph| glyph.face != 0),
            "the glyphs come from the fallback font"
        );
    }

    #[test]
    fn text_stays_one_line() {
        let font = font("Exo2", 16);
        let mut font_data = font.0.as_mut();

        let joined = font_data.shape("ab");
        let broken = font_data.shape("a\nb\r");
        assert_eq!(broken.glyphs.len(), 2);
        assert_eq!(broken.width, joined.width, "line breaks take no room");
        assert!(
            broken.glyphs.iter().all(|glyph| glyph.y == 0.0),
            "both glyphs sit on the one baseline"
        );
    }

    #[test]
    fn lines_are_kept_while_they_are_used() {
        let font = font("Exo2", 16);
        let mut font_data = font.0.as_mut();
        let generation = font_data.lines_generation;

        let first = font_data.shape_at("Back", 10, generation);
        let again = font_data.shape_at("Back", 10, generation);
        assert!(Rc::ptr_eq(&first, &again), "measure and draw share a line");

        for frame in 11..14 {
            let next = font_data.shape_at("Back", frame, generation);
            assert!(
                Rc::ptr_eq(&first, &next),
                "a line asked for every frame stays (frame {frame})"
            );
        }
    }

    #[test]
    fn unused_lines_are_dropped() {
        let font = font("Exo2", 16);
        let mut font_data = font.0.as_mut();
        let generation = font_data.lines_generation;

        let back = font_data.shape_at("Back", 10, generation);
        font_data.shape_at("Speed 1", 11, generation);
        assert!(
            font_data.lines.contains_key("Back"),
            "kept through the next frame"
        );

        font_data.shape_at("Speed 2", 12, generation);
        assert!(
            !font_data.lines.contains_key("Back"),
            "gone once a whole frame passed without it"
        );
        assert!(
            font_data.lines.contains_key("Speed 1"),
            "used in the previous frame"
        );
        assert!(font_data.lines.contains_key("Speed 2"));

        let later = font_data.shape_at("Back", 20, generation);
        assert!(!Rc::ptr_eq(&back, &later), "shaped again");
        assert_eq!(font_data.lines.len(), 1, "frames without the font drop all");
    }

    #[test]
    fn a_registered_font_clears_the_lines() {
        let font = font("Exo2", 16);
        let mut font_data = font.0.as_mut();
        let generation = font_data.lines_generation;

        let before = font_data.shape_at("Back", 10, generation);
        let after = font_data.shape_at("Back", 10, generation + 1);
        assert!(!Rc::ptr_eq(&before, &after));
        assert_eq!(before.glyphs.len(), after.glyphs.len());
    }

    /// The RGBA texel at `x, y` of an outlined glyph `w` pixels wide.
    fn texel(texels: &[u8], w: usize, x: usize, y: usize) -> [u8; 4] {
        let i = (y * w + x) * 4;
        [texels[i], texels[i + 1], texels[i + 2], texels[i + 3]]
    }

    #[test]
    fn outline_and_halo_are_the_prototype_stamps() {
        // One fully covered pixel in the middle of its margins.
        let texels = outlined_texels(&[255], 1, 1);
        let size = (1 + 2 * OUTLINED_MARGIN) as usize;
        assert_eq!(texels.len(), size * size * 4);
        // Fill, outline and halo; alpha is the glow (its own test).
        let at = |dx: i32, dy: i32| {
            let m = OUTLINED_MARGIN;
            let t = texel(&texels, size, (m + dx) as usize, (m + dy) as usize);
            [t[0], t[1], t[2]]
        };
        let step = (HALO_STEP_ALPHA * 255.0 + 0.5) as u8;

        assert_eq!(at(0, 0), [255, 0, 0], "the ink itself: fill only");
        assert_eq!(at(1, 0), [0, 255, step], "next to it: outline");
        assert_eq!(at(-1, -1), [0, 255, step], "diagonally next to it");
        assert_eq!(at(2, 0), [0, 0, step], "two away: halo only");
        assert_eq!(at(3, 1), [0, 0, step], "inside the halo radius");
        assert_eq!(at(3, 2), [0, 0, 0], "outside the halo radius");
    }

    #[test]
    fn the_glow_is_strongest_on_the_ink_and_fades_out_within_the_margin() {
        let texels = outlined_texels(&[255; 9], 3, 3);
        let size = (3 + 2 * OUTLINED_MARGIN) as usize;
        let middle = size / 2;
        let glow = |dx: usize| texel(&texels, size, middle + dx, middle)[3];

        assert!(glow(0) > 128, "on the ink: {}", glow(0));
        for dx in 0..(size - middle - 1) {
            assert!(glow(dx + 1) <= glow(dx), "fades outwards at {dx}");
        }
        assert!(
            glow(size - middle - 1) < 4,
            "next to nothing at the margin's edge"
        );
    }

    #[test]
    fn the_halo_adds_up_its_stamps() {
        // Deep inside ink every one of the 36 stamps covers the pixel.
        let texels = outlined_texels(&[255; 81], 9, 9);
        let size = 9 + 2 * OUTLINED_MARGIN as usize;
        let middle = texel(&texels, size, size / 2, size / 2);
        let full = 1.0 - (1.0 - HALO_STEP_ALPHA).powi(stamp_offsets(HALO_RADIUS).len() as i32);
        assert_eq!(stamp_offsets(HALO_RADIUS).len(), 36);
        assert_eq!(stamp_offsets(OUTLINE_RADIUS).len(), 8);
        assert_eq!(middle[2], (full * 255.0 + 0.5) as u8);
        assert_eq!(middle[1], 255);
    }

    #[test]
    fn outlined_glyphs_grow_by_their_margin_and_keep_their_fill() {
        let font = font("Exo2", 32);
        let mut r = Renderer::new_headless();
        let mut font_data = font.0.as_mut();
        let placed = font_data.shape("A").glyphs[0];

        let plain = font_data.glyph(&mut r, AtlasKind::Plain, placed.face, placed.id, 0);
        let outlined = font_data.glyph(&mut r, AtlasKind::Outlined, placed.face, placed.id, 0);
        let m = OUTLINED_MARGIN;
        assert_eq!(outlined.x0, plain.x0 - m);
        assert_eq!(outlined.y0, plain.y0 - m);
        assert_eq!(outlined.sx, plain.sx + 2 * m);
        assert_eq!(outlined.sy, plain.sy + 2 * m);
        assert!(font_data.outlined.pages[0].size == OUTLINED_ATLAS_SIZE);

        // Red of the outlined glyph = the plain glyph's texels, so the text
        // itself is drawn exactly as `Font::draw` draws it.
        let plain_page = &font_data.plain.pages[plain.page.unwrap()];
        let outlined_page = &font_data.outlined.pages[outlined.page.unwrap()];
        for y in 0..plain.sy as u32 {
            for x in 0..plain.sx as u32 {
                let p = plain_page.cpu[((plain.ay + y) * plain_page.size + plain.ax + x) as usize];
                let (ox, oy) = (outlined.ax + x + m as u32, outlined.ay + y + m as u32);
                let o = outlined_page.cpu[((oy * outlined_page.size + ox) * 4) as usize];
                assert_eq!(p, o, "fill at {x}, {y}");
            }
        }

        // Both kinds of page upload their changed rows (RGBA8 too).
        font_data.plain.flush(&mut r);
        font_data.outlined.flush(&mut r);
        assert!(
            font_data
                .outlined
                .pages
                .iter()
                .all(|page| page.dirty.is_none())
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
