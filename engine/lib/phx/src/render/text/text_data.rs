use std::ops::Range;

use glam::Vec2;
use indexmap::IndexMap;
use parley::layout::{Alignment, Cursor, Glyph, GlyphRun};
use parley::Layout;
use swash::scale::{image::Content, Render, ScaleContext, Scaler, Source, StrikeWith};
use swash::zeno::{Format, Vector};
use swash::FontRef;

use internal::ConvertIntoString;

use crate::input::{Button, Input};
use crate::render::{
    Color, DataFormat_Float, PixelFormat_RGBA, Tex2D, Tex2D_Create, Tex2D_SetData, TexFormat_RGBA8,
};

use super::{TextAlignment, TextContext, TextSelection, TextStyle};

/// Text string, styling and layouting parameters.
#[derive(Clone)]
pub struct TextData {
    text: String,
    default_style: TextStyle,
    section_styles: IndexMap<[usize; 2], TextStyle>,
    alignment: Alignment,
    multiline: bool,
    selection: TextSelection,
    selection_color: Color,
    mouse_pos: Vec2,
}

#[luajit_ffi_gen::luajit_ffi]
impl TextData {
    #[bind(name = "Create")]
    pub fn new(
        text: &str,
        default_style: &TextStyle,
        alignment: TextAlignment,
        multiline: bool,
    ) -> Self {
        Self {
            text: text.into(),
            default_style: default_style.clone(),
            section_styles: Default::default(),
            alignment: alignment.into(),
            multiline,
            selection: TextSelection::new(),
            selection_color: Color::new(0.2, 0.2, 0.7, 0.8),
            mouse_pos: Vec2::new(-1.0, -1.0),
        }
    }

    /// Set style of the text section beginning at 'start_pos' position and up to 'end_pos'.
    pub fn set_section_style(&mut self, start_pos: usize, end_pos: usize, style: &TextStyle) {
        // TODO: manage sections overlapping properly to avoid uncontrollable map growth
        self.section_styles
            .insert([start_pos, end_pos], style.clone());
    }

    /// Sets cursor position in a text before character at position `pos`.
    /// If pos >= text size then cursor is placed after the latest text character.
    pub fn set_cursor_pos(&mut self, pos: usize) {
        // pos == self.text.len() to select last symbol
        assert!(pos <= self.text.len());

        self.selection = TextSelection::Cursor(pos);
    }

    pub fn set_selection_color(&mut self, color: &Color) {
        self.selection_color = *color;
    }

    pub fn set_selection(&mut self, start_pos: usize, end_pos: usize) {
        // pos == self.text.len() to select last symbol
        assert!(start_pos <= self.text.len());
        assert!(end_pos <= self.text.len());

        self.selection = TextSelection::Selection(Range {
            start: start_pos,
            end: end_pos,
        });
    }
}

impl TextData {
    pub fn is_multiline(&self) -> bool {
        self.multiline
    }

    pub(super) fn update(&mut self, text_data: &TextData) -> bool {
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
    pub fn render(
        &mut self,
        text_ctx: &mut TextContext,
        width: f32,
        scale_factor: f32,
        widget_pos: Vec2,
        input: Option<&Input>,
    ) -> *mut Tex2D {
        // TODO: replace all `\n` in self.text with spaces if not multiline?
        let mut builder =
            text_ctx
                .layout
                .ranged_builder(&mut text_ctx.font, &self.text, scale_factor);

        self.default_style.apply_default(&mut builder);

        for (range, style) in &self.section_styles {
            style.apply_to_section(&mut builder, range[0], range[1]);
        }

        // Build the builder into a Layout
        let mut layout: Layout<Color> = builder.build();

        // The width for line wrapping
        let max_advance = if self.multiline && width > 0.0 {
            Some(width * scale_factor)
        } else {
            None
        };

        // Perform layout (including bidi resolution and shaping) with alignment
        layout.break_all_lines(max_advance, self.alignment);

        // Padding around the output image
        // TODO: workaround. For some reason zeno crate (used by swash) shifts placement.left
        // by several pixels to the left that makes position coordinate negative in some cases
        let padding = 5;

        if let Some(input) = input {
            self.update_selection(&layout, widget_pos, padding, input);
        }

        // Create buffer to render into
        let width = layout.width().ceil() as u32 + (padding * 2);
        let height = layout.height().ceil() as u32;
        let mut buffer = vec![Color::TRANSPARENT; (width * height) as usize];
        let mut glyph_idx = 0;

        // Iterate over laid out lines
        for line in layout.lines() {
            let metrics = line.metrics();
            let line_range = Range {
                start: (metrics.baseline - metrics.ascent - metrics.leading * 0.5).floor() as u32,
                end: u32::min(
                    (metrics.baseline + metrics.descent + metrics.leading * 0.5).floor() as u32,
                    height,
                ),
            };

            // Iterate over GlyphRun's within each line
            for glyph_run in line.glyph_runs() {
                render_glyph_run(
                    &mut text_ctx.scale,
                    &layout,
                    &glyph_run,
                    &mut buffer,
                    padding,
                    width,
                    &mut glyph_idx,
                    &self.selection.range(),
                    &self.selection_color,
                    &line_range,
                );
            }
        }

        // Create texture
        unsafe {
            let tex = Tex2D_Create(width as i32, height as i32, TexFormat_RGBA8);

            Tex2D_SetData(
                &mut *tex,
                buffer.as_ptr() as _,
                PixelFormat_RGBA,
                DataFormat_Float,
            );

            tex
        }
    }

    fn update_selection(
        &mut self,
        layout: &Layout<Color>,
        widget_pos: Vec2,
        padding: u32,
        input: &Input,
    ) {
        let mouse_pos = input.mouse().position();

        if (input.is_pressed(Button::MouseLeft)
            || input.is_down(Button::MouseLeft)
            || input.is_released(Button::MouseLeft))
            && self.mouse_pos != mouse_pos
        {
            let widget_mouse_pos = mouse_pos - widget_pos;
            let cursor = Cursor::from_point(
                layout,
                widget_mouse_pos.x - padding as f32,
                widget_mouse_pos.y,
            );

            if input.is_pressed(Button::MouseLeft) {
                self.selection.set_cursor(cursor.text_start);
            } else {
                self.selection.set_end(cursor.text_end);
            }

            self.mouse_pos = mouse_pos;
        }
    }
}

fn render_glyph_run(
    context: &mut ScaleContext,
    layout: &Layout<Color>,
    glyph_run: &GlyphRun<Color>,
    buffer: &mut [Color],
    padding: u32,
    image_width: u32,
    glyph_idx: &mut usize,
    selection_range: &Range<usize>,
    selection_color: &Color,
    line_range: &Range<u32>,
) {
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
        let glyph_x = run_x + glyph.x + (padding as f32);
        let glyph_y = run_y - glyph.y;

        run_x += glyph.advance;

        let cursor = Cursor::from_point(layout, glyph_x, glyph_y);
        let is_selected =
            selection_range.start <= cursor.text_end && cursor.text_end <= selection_range.end;

        let bg_color = if is_selected {
            selection_color
        } else {
            &Color::TRANSPARENT
        };

        render_glyph(
            buffer,
            &mut scaler,
            &color,
            bg_color,
            &glyph,
            glyph_x,
            glyph_y,
            image_width,
            line_range,
        );

        *glyph_idx += 1;
    }
}

fn render_glyph(
    buffer: &mut [Color],
    scaler: &mut Scaler,
    color: &Color,
    bg_color: &Color,
    glyph: &Glyph,
    glyph_x: f32,
    glyph_y: f32,
    image_width: u32,
    line_range: &Range<u32>,
) {
    // Compute the fractional offset
    // You'll likely want to quantize this in a real renderer
    // TODO: swash for some reason shifts horizontal offset by 1 pixel to the left so we have to correct it here
    let offset = Vector::new(glyph_x.fract() + 1.0, glyph_y.fract());

    // Render the glyph using swash
    let glyph_image = Render::new(
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

    let glyph_width = glyph_image.placement.width;
    let glyph_height = glyph_image.placement.height;
    let glyph_x = (glyph_x.floor() as i32 + glyph_image.placement.left) as u32;
    let glyph_y = (glyph_y.floor() as i32 - glyph_image.placement.top) as u32;

    if bg_color.is_opaque() {
        // draw selection background
        for y in line_range.clone() {
            for x in glyph_x..glyph_x + glyph_width {
                let idx = y * image_width + x;

                buffer[idx as usize].blend_with(bg_color);
            }
        }
    }

    match glyph_image.content {
        Content::Mask => {
            // TODO: check if a single loop over i: [0..glyph_height*glyph_width] will be possible and more efficient
            let mut i = 0;
            for pixel_y in 0..glyph_height {
                for pixel_x in 0..glyph_width {
                    if glyph_image.data[i] > 0 {
                        let x = glyph_x + pixel_x;
                        let y = glyph_y + pixel_y;
                        let idx = y * image_width + x;

                        let alpha = color_u8_to_f32(glyph_image.data[i]);
                        let color = color.with_alpha(alpha);

                        // TODO: blend?
                        buffer[idx as usize] = color;
                    }

                    i += 1;
                }
            }
        }
        Content::SubpixelMask => unimplemented!(),
        Content::Color => {
            let row_size = glyph_width as usize * 4;
            for (pixel_y, row) in glyph_image.data.chunks_exact(row_size).enumerate() {
                for (pixel_x, pixel) in row.chunks_exact(4).enumerate() {
                    if pixel[3] > 0 {
                        let x = glyph_x + pixel_x as u32;
                        let y = glyph_y + pixel_y as u32;
                        let idx = y * glyph_width + x;

                        // TODO: blend?
                        buffer[idx as usize] = Color::new(
                            color_u8_to_f32(pixel[0]),
                            color_u8_to_f32(pixel[1]),
                            color_u8_to_f32(pixel[2]),
                            color_u8_to_f32(pixel[3]),
                        );
                    }
                }
            }
        }
    };
}

#[inline]
fn color_u8_to_f32(v: u8) -> f32 {
    v as f32 / 255.0
}
