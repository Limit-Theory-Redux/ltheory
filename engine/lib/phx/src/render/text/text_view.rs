use glam::{Vec2, Vec4};
use indexmap::IndexMap;
use parley::layout::{Alignment, Glyph, GlyphRun};
use parley::Layout;
use swash::scale::{image::Content, Render, ScaleContext, Scaler, Source, StrikeWith};
use swash::zeno::{Format, Vector};
use swash::FontRef;

use internal::ConvertIntoString;

use crate::render::{
    Color, DataFormat_Float, PixelFormat_RGBA, Tex2D, Tex2D_Create, Tex2D_Free, Tex2D_SetData,
    TexFormat_RGBA8,
};

use super::{TextAlignment, TextContext, TextStyle};

pub struct TextView {
    data: TextData,
    size: Vec2,
    dirty: bool,
    tex: *mut Tex2D,
}

impl TextView {
    pub fn new(data: &TextData) -> Self {
        Self {
            data: data.clone(),
            size: Default::default(),
            dirty: true,
            tex: std::ptr::null_mut(),
        }
    }

    pub fn set_data(&mut self, text_data: &TextData) {
        self.dirty = self.data.update(text_data);
    }

    pub fn update(&mut self, text_ctx: &mut TextContext, size: Vec2) -> *mut Tex2D {
        if self.size != size {
            self.size = size;
            self.dirty = true;
        }

        // Regenerate texture only if something was changed
        if self.dirty {
            let tex = self.data.render(text_ctx, self.size);

            if self.tex != std::ptr::null_mut() {
                unsafe { Tex2D_Free(self.tex) };
            }

            self.tex = tex;
            self.dirty = false;
        }

        self.tex
    }
}

impl Drop for TextView {
    fn drop(&mut self) {
        if self.tex != std::ptr::null_mut() {
            unsafe { Tex2D_Free(self.tex) };
        }
    }
}

/// Text string, styling and layouting parameters.
#[derive(Clone, PartialEq)]
pub struct TextData {
    text: String,
    default_style: TextStyle,
    section_styles: IndexMap<[usize; 2], TextStyle>,
    alignment: Alignment,
}

#[luajit_ffi_gen::luajit_ffi]
impl TextData {
    #[bind(name = "Create")]
    pub fn new(text: &str, default_style: &TextStyle, alignment: TextAlignment) -> Self {
        Self {
            text: text.into(),
            default_style: default_style.clone(),
            section_styles: Default::default(),
            alignment: alignment.into(),
        }
    }

    /// Set style of the text section beginning at 'start_pos' position and up to 'end_pos'.
    pub fn set_section_style(&mut self, start_pos: usize, end_pos: usize, style: &TextStyle) {
        // TODO: manage sections overlapping properly to avoid uncontrollable map growth
        self.section_styles[&[start_pos, end_pos]] = style.clone();
    }
}

impl TextData {
    fn update(&mut self, text_data: &TextData) -> bool {
        let mut updated = if self.text != text_data.text {
            self.text = text_data.text.clone();
            true
        } else {
            false
        };

        updated |= if self.default_style != text_data.default_style {
            self.default_style = text_data.default_style.clone();
            true
        } else {
            false
        };

        updated |= if self.section_styles != text_data.section_styles {
            self.section_styles = text_data.section_styles.clone();
            true
        } else {
            false
        };

        updated |= if self.alignment != text_data.alignment {
            self.alignment = text_data.alignment;
            true
        } else {
            false
        };

        updated
    }

    /// Generate Tex2D texture with layouted text based on text parameters.
    // TODO: keeping a texture for a large texts will be memory consuming.
    // Generate per-line textures and keep only visible ones with some buffered pre- and post-lines.
    fn render(&self, text_ctx: &mut TextContext, size: Vec2) -> *mut Tex2D {
        // The display scale for HiDPI rendering
        // TODO: set from outside
        let display_scale = 1.0;

        let mut builder =
            text_ctx
                .layout
                .ranged_builder(&mut text_ctx.font, &self.text, display_scale);

        self.default_style.apply_default(&mut builder);

        for (range, style) in &self.section_styles {
            style.apply_to_section(&mut builder, range[0], range[1]);
        }

        // let brush_style = StyleProperty::Brush(self.color);
        // builder.push_default(&brush_style);

        // // Set default font family
        // let font_stack = FontStack::Source(&self.font_family);
        // let font_stack_style = StyleProperty::FontStack(font_stack);
        // builder.push_default(&font_stack_style);
        // builder.push_default(&StyleProperty::LineHeight(1.3));
        // builder.push_default(&StyleProperty::FontSize(self.font_size));

        // TODO: implement custom per symbol styling
        // Set the first 4 characters to bold
        // let bold = FontWeight::new(600.0);
        // let bold_style = StyleProperty::FontWeight(bold);
        // builder.push(&bold_style, 0..4);

        // Build the builder into a Layout
        let mut layout: Layout<Color> = builder.build();

        // The width for line wrapping
        let max_advance = Some(size.x as f32 * display_scale);

        // Perform layout (including bidi resolution and shaping) with alignment
        layout.break_all_lines(max_advance, self.alignment);

        // Create buffer to render into
        let mut buffer = Vec::with_capacity((size.x * size.y) as usize);

        // Iterate over laid out lines
        for line in layout.lines() {
            // Iterate over GlyphRun's within each line
            for glyph_run in line.glyph_runs() {
                render_glyph_run(&mut text_ctx.scale, &glyph_run, &mut buffer);
            }
        }

        // Create texture
        unsafe {
            let tex = Tex2D_Create(size.x.ceil() as i32, size.y.ceil() as i32, TexFormat_RGBA8);

            Tex2D_SetData(
                &mut *tex,
                buffer.as_ptr() as _,
                PixelFormat_RGBA,
                DataFormat_Float,
            );

            tex
        }
    }
}

fn render_glyph_run(context: &mut ScaleContext, glyph_run: &GlyphRun<Color>, buffer: &mut [Vec4]) {
    // Resolve properties of the GlyphRun
    let mut run_x = glyph_run.offset();
    let run_y = glyph_run.baseline();
    let style = glyph_run.style();
    let color = style.brush;

    // Get the "Run" from the "GlyphRun"
    let run = glyph_run.run();

    // Resolve properties of the Run
    let font = run.font();
    let font_size = run.font_size();
    let normalized_coords = run.normalized_coords();

    // Convert from parley::Font to swash::FontRef
    let font_ref = FontRef::from_index(font.data.as_ref(), font.index as usize).unwrap();

    // Build a scaler. As the font properties are constant across an entire run of glyphs
    // we can build one scaler for the run and reuse it for each glyph.
    let mut scaler = context
        .builder(font_ref)
        .size(font_size)
        .hint(true)
        .normalized_coords(normalized_coords)
        .build();

    // Iterates over the glyphs in the GlyphRun
    for glyph in glyph_run.glyphs() {
        let glyph_x = run_x + glyph.x;
        let glyph_y = run_y - glyph.y;

        run_x += glyph.advance;

        render_glyph(buffer, &mut scaler, color, glyph, glyph_x, glyph_y);
    }
}

fn render_glyph(
    buffer: &mut [Vec4],
    scaler: &mut Scaler,
    color: Color,
    glyph: Glyph,
    glyph_x: f32,
    glyph_y: f32,
) {
    // Compute the fractional offset
    // You'll likely want to quantize this in a real renderer
    let offset = Vector::new(glyph_x.fract(), glyph_y.fract());

    // Render the glyph using swash
    let rendered_glyph = Render::new(
        // Select our source order
        &[
            Source::ColorOutline(0),
            Source::ColorBitmap(StrikeWith::BestFit),
            Source::Outline,
        ],
    )
    // Select the simple alpha (non-subpixel) format
    .format(Format::Alpha)
    // Apply the fractional offset
    .offset(offset)
    // Render the image
    .render(scaler, glyph.id)
    .unwrap();

    let glyph_width = rendered_glyph.placement.width;
    let glyph_height = rendered_glyph.placement.height;
    let glyph_x = (glyph_x.floor() as i32 + rendered_glyph.placement.left) as u32;
    let glyph_y = (glyph_y.floor() as i32 - rendered_glyph.placement.top) as u32;

    match rendered_glyph.content {
        Content::Mask => {
            let mut i = 0;
            for pixel_y in 0..glyph_height {
                for pixel_x in 0..glyph_width {
                    let x = glyph_x + pixel_x;
                    let y = glyph_y + pixel_y;
                    let idx = y * glyph_width + x;
                    let alpha = rendered_glyph.data[i];
                    // TODO: normalize alpha?
                    let color = Vec4::new(color.r, color.g, color.b, alpha as f32);

                    buffer[idx as usize] = color;

                    i += 1;
                }
            }
        }
        Content::SubpixelMask => unimplemented!(),
        Content::Color => {
            let row_size = glyph_width as usize * 4;
            for (pixel_y, row) in rendered_glyph.data.chunks_exact(row_size).enumerate() {
                for (pixel_x, pixel) in row.chunks_exact(4).enumerate() {
                    let x = glyph_x + pixel_x as u32;
                    let y = glyph_y + pixel_y as u32;
                    let idx = y * glyph_width + x;
                    // TODO: normalize color?
                    let color = Vec4::new(
                        pixel[0] as f32,
                        pixel[1] as f32,
                        pixel[2] as f32,
                        pixel[3] as f32,
                    );

                    buffer[idx as usize] = color;
                }
            }
        }
    };
}
