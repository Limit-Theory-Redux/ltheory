use std::ops::Range;

use glam::Vec2;
use parley::layout::{Alignment, Cursor, GlyphRun};
use parley::Layout;
use swash::scale::ScaleContext;
use swash::FontRef;

use internal::ConvertIntoString;

use crate::input::{Button, Input};
use crate::render::{
    Color, DataFormat_Float, PixelFormat_RGBA, Tex2D, Tex2D_Create, Tex2D_SetData, TexFormat_RGBA8,
};

use super::text_render::render_glyph;
use super::{
    TextAlignment, TextContext, TextCursorRect, TextSectionStyle, TextSelection, TextStyle,
};

/// Text string, styling and layouting parameters.
#[derive(Clone, PartialEq)]
pub struct TextData {
    text: String,
    text_changed: bool,
    default_style: TextStyle,
    section_style: TextSectionStyle,
    alignment: Alignment,
    multiline: bool,
    selection: TextSelection,
    selection_color: Color,
    mouse_pos: Vec2,
    cursor_rect: TextCursorRect,

    // Horizontal padding around the output image
    // TODO: workaround. For some reason zeno crate (used by swash) shifts placement.left
    // by several pixels to the left that makes position coordinate negative in some cases
    padding: f32,
}

#[luajit_ffi_gen::luajit_ffi]
impl TextData {
    #[bind(name = "Create")]
    pub fn new(
        text: &str,
        default_style: &TextStyle,
        cursor_color: &Color,
        selection_color: &Color,
        alignment: TextAlignment,
        multiline: bool,
    ) -> Self {
        let text = if multiline {
            text.into()
        } else {
            text.replace(&['\n', '\r'], " ")
        };

        Self {
            text,
            text_changed: false,
            default_style: default_style.clone(),
            section_style: Default::default(),
            alignment: alignment.into(),
            multiline,
            selection: TextSelection::new(),
            selection_color: selection_color.clone(),
            mouse_pos: Vec2::new(-1.0, -1.0),
            cursor_rect: TextCursorRect::new(cursor_color),
            padding: 5.0,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    /// Set style of the text section beginning at 'start_pos' position and up to 'end_pos'.
    pub fn set_section_style(&mut self, start_pos: usize, end_pos: usize, style: &TextStyle) {
        self.section_style.add(start_pos, end_pos, style);
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
    pub fn set_text(&mut self, text: &str) {
        self.text = text.into();
    }

    pub fn is_multiline(&self) -> bool {
        self.multiline
    }

    pub fn is_text_changed(&self) -> bool {
        self.text_changed
    }

    pub fn unset_text_changed(&mut self) {
        self.text_changed = false;
    }

    pub fn cursor_rect(&self) -> &TextCursorRect {
        &self.cursor_rect
    }

    pub fn selection(&self) -> &TextSelection {
        &self.selection
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

        updated |= if self.section_style != text_data.section_style {
            self.section_style = text_data.section_style.clone();
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

    pub fn calculate_rect(&self, text_ctx: &mut TextContext, scale_factor: f32) -> Option<Vec2> {
        if self.multiline {
            // size of the multiline text should be specified explicitly
            return None;
        }

        let mut builder =
            text_ctx
                .layout
                .ranged_builder(&mut text_ctx.font, &self.text, scale_factor);

        self.default_style.apply_default(&mut builder);

        self.section_style.apply(&mut builder);

        let mut layout: Layout<Color> = builder.build();

        // Perform layout (including bidi resolution and shaping) with alignment
        layout.break_all_lines(None, self.alignment);

        let width = layout.width().ceil() + self.padding * 2.0;
        let height = layout.height().ceil();

        Some(Vec2::new(width, height))
    }

    /// Generate Tex2D texture with layouted text based on text parameters.
    // TODO: keeping a texture for a large texts will be memory consuming.
    // Generate per-line textures and keep only visible ones with some buffered pre- and post-lines.
    pub fn render(
        &mut self,
        text_ctx: &mut TextContext,
        width: f32,
        scale_factor: f32,
        mut widget_pos: Vec2,
        input: Option<&Input>,
        editable: bool,
        focused: bool,
        clipboard: &mut String,
    ) -> *mut Tex2D {
        if editable && focused {
            if let Some(input) = input {
                self.process_text_edit(input, clipboard);
            }
        }

        let mut builder =
            text_ctx
                .layout
                .ranged_builder(&mut text_ctx.font, &self.text, scale_factor);

        self.default_style.apply_default(&mut builder);

        self.section_style.apply(&mut builder);

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

        let mut selection_changed = false;
        if focused && !self.text.is_empty() {
            if let Some(input) = input {
                widget_pos.x += self.padding;

                selection_changed = self.selection.update(
                    &layout,
                    widget_pos,
                    input,
                    &self.text,
                    &mut self.mouse_pos,
                );
            }
        }

        // Create buffer to render into
        let width = (layout.width().ceil() + self.padding * 2.0) as u32;
        let height = layout.height().ceil() as u32;
        let mut buffer = vec![Color::TRANSPARENT; (width * height) as usize];
        let mut glyph_idx = 0;
        let selection_end = self.selection.end();

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
                self.render_glyph_run(
                    &mut text_ctx.scale,
                    &layout,
                    &glyph_run,
                    &mut buffer,
                    width,
                    &mut glyph_idx,
                    &line_range,
                    focused,
                );
            }
        }

        // calculate cursor rect
        if (self.text_changed || selection_changed) && editable && focused {
            self.cursor_rect.build(
                &layout,
                height,
                selection_end,
                self.padding,
                self.text.len(),
            );
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

    fn process_text_edit(&mut self, input: &Input, clipboard: &mut String) {
        // remove backspace, del and new line characters from the text input
        let chars_to_remove: &[char] = if self.multiline {
            &['\u{7f}', '\u{8}']
        } else {
            &['\u{7f}', '\u{8}', '\n', '\r']
        };
        let mut typed_text = if !input.is_keyboard_alt_down() && !input.is_keyboard_ctrl_down() {
            input
                .keyboard()
                .text()
                .replace(chars_to_remove, "")
                .replace('\r', "\n") // Pressing Enter generates \r but parley uses \n for line division
        } else {
            "".into()
        };

        let insertFromClipboard = input.is_keyboard_ctrl_down()
            && input.is_pressed(Button::KeyboardV)
            || input.is_keyboard_shift_down() && input.is_pressed(Button::KeyboardInsert);

        if typed_text.is_empty() && insertFromClipboard {
            typed_text = clipboard.replace(chars_to_remove, "");
        }

        *clipboard = "".into();

        // information about change: Some((pos, removed, added))
        // pos - position in a text where change happened
        // removed - how much of a text was removed
        // added - how much of a text was added
        let mut change = None;

        if !typed_text.is_empty() {
            match &mut self.selection {
                TextSelection::Cursor(pos) => {
                    let mut added = 0;

                    if *pos >= self.text.len() {
                        if typed_text == "\n"
                            && self.text.get(self.text.len() - 1..self.text.len()) != Some("\n")
                        {
                            self.text += "\n";
                            added += 1;
                        }

                        self.text += &typed_text;
                    } else {
                        self.text.insert_str(*pos, &typed_text);
                    }

                    added += typed_text.len();

                    if !self.section_style.is_empty() {
                        change = Some((*pos, 0, added));
                    }

                    *pos += typed_text.len();
                }
                TextSelection::Selection(range) => {
                    let (start, end) = if range.start < range.end {
                        (range.start, range.end)
                    } else {
                        (range.end, range.start)
                    };

                    self.text.replace_range(start..end, &typed_text);
                    self.selection = TextSelection::Cursor(start + typed_text.len());

                    if !self.section_style.is_empty() {
                        change = Some((start, end - start, typed_text.len()));
                    }
                }
            }

            self.text_changed = true;
        } else {
            let cutToClipboard =
                input.is_keyboard_ctrl_down() && input.is_pressed(Button::KeyboardX);
            let copyToClipboard =
                input.is_keyboard_ctrl_down() && input.is_pressed(Button::KeyboardC);

            if input.is_pressed(Button::KeyboardBackspace)
                || input.is_pressed(Button::KeyboardDelete)
                || cutToClipboard
                || copyToClipboard
            {
                match &mut self.selection {
                    TextSelection::Cursor(pos) => {
                        if input.is_pressed(Button::KeyboardBackspace) {
                            if *pos > 0 {
                                *pos -= 1;
                                self.text.remove(*pos);
                                self.text_changed = true;

                                if !self.section_style.is_empty() {
                                    change = Some((*pos, 1, 0));
                                }
                            }
                        } else if input.is_pressed(Button::KeyboardDelete) {
                            if *pos < self.text.len() {
                                self.text.remove(*pos);
                                self.text_changed = true;

                                if !self.section_style.is_empty() {
                                    change = Some((*pos, 1, 0));
                                }
                            }
                        }
                    }
                    TextSelection::Selection(range) => {
                        let (start, end) = if range.start < range.end {
                            (range.start, range.end)
                        } else {
                            (range.end, range.start)
                        };

                        if cutToClipboard || copyToClipboard {
                            *clipboard = self.text[start..end].into();
                        }

                        if !copyToClipboard {
                            self.text.replace_range(start..end, "");
                            self.selection = TextSelection::Cursor(start);
                            self.text_changed = true;

                            if !self.section_style.is_empty() {
                                change = Some((start, end - start, 0));
                            }
                        }
                    }
                }
            }
        }

        if let Some((pos, removed, added)) = change {
            self.section_style.update(pos, removed, added);
        }
    }

    fn render_glyph_run(
        &self,
        context: &mut ScaleContext,
        layout: &Layout<Color>,
        glyph_run: &GlyphRun<Color>,
        buffer: &mut [Color],
        image_width: u32,
        glyph_idx: &mut usize,
        line_range: &Range<u32>,
        focused: bool,
    ) {
        let is_selection = focused && !self.selection.is_cursor();
        let selection_range = self.selection.range();

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
            let glyph_x = run_x + glyph.x + self.padding;
            let glyph_y = run_y - glyph.y;

            run_x += glyph.advance;

            let cursor = Cursor::from_point(layout, glyph_x, glyph_y);
            let is_selected = is_selection
                && selection_range.start < cursor.text_end
                && cursor.text_start < selection_range.end;

            let bg_color = if is_selected {
                &self.selection_color
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
}