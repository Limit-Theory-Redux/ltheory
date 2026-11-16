use glam::Vec2;
use parley::Layout;
use parley::layout::{Alignment, GlyphRun};
use swash::FontRef;
use swash::scale::ScaleContext;

use super::text_render::render_glyph;
use super::{
    TextAlignment, TextContext, TextCursorRect, TextSectionStyle, TextSelection, TextStyle,
};
use crate::input::{Button, Input};
use crate::render::{Color, DataFormat, PixelFormat, Renderer, Tex2D, TexFormat};

pub type TextLayout = Layout<Color>;

/// Text string, styling and layouting parameters.
#[derive(Clone)]
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
    scale_factor: f32,
    max_advance: f32,

    // Horizontal padding around the output image
    // TODO: workaround. For some reason zeno crate (used by swash) shifts placement.left
    // by several pixels to the left that makes position coordinate negative in some cases
    padding: f32,

    rebuild_layout: bool,
    rebuild_line_breaks: bool,
    layout: TextLayout,

    /// Changed through its API (from Lua) since `Gui:textView` last copied it
    /// into its view. New text data counts as changed.
    needs_sync: bool,
    /// The layout was rebuilt since the texture was last rendered. Measuring
    /// the text (`calculate_rect`, before the layout pass) can rebuild it, so
    /// "rebuilt now" isn't enough to know the texture is out of date.
    texture_stale: bool,
}

impl PartialEq for TextData {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
            && self.text_changed == other.text_changed
            && self.default_style == other.default_style
            && self.section_style == other.section_style
            && self.alignment == other.alignment
            && self.multiline == other.multiline
            && self.selection == other.selection
            && self.selection_color == other.selection_color
            && self.mouse_pos == other.mouse_pos
            && self.cursor_rect == other.cursor_rect
            && self.padding == other.padding
    }
}

/// Text information used in the [`TextView`] component.
/// Use `Gui:textView(textData, editable)` to add text view element to the gui hierarchy.
/// To retrieve changes of the editable text made by user, use `Gui:getTextViewChanges(textData)`
/// and `textData:text()` functions.
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
            text.replace(['\n', '\r'], " ")
        };

        Self {
            text,
            text_changed: false,
            default_style: default_style.clone(),
            section_style: Default::default(),
            alignment: alignment.into(),
            multiline,
            selection: TextSelection::default(),
            selection_color: *selection_color,
            mouse_pos: Vec2::new(-1.0, -1.0),
            cursor_rect: TextCursorRect::new(cursor_color),
            scale_factor: 1.0,
            max_advance: 0.0,

            padding: 5.0,

            rebuild_layout: true,
            rebuild_line_breaks: true,
            layout: TextLayout::new(),

            needs_sync: true,
            texture_stale: true,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: &str) {
        if self.text != text {
            self.text = text.into();
            self.invalidate_layout();
            self.needs_sync = true;
        }
    }

    pub fn set_scale_factor(&mut self, scale_factor: f32) {
        if self.scale_factor != scale_factor {
            self.scale_factor = scale_factor;
            self.invalidate_layout();
        }
    }

    pub fn is_multiline(&self) -> bool {
        self.multiline
    }

    pub fn set_multiline(&mut self, multiline: bool) {
        self.multiline = multiline;
    }

    /// Set style of the text section beginning at 'start_pos' position and up to 'end_pos'.
    pub fn set_section_style(&mut self, start_pos: usize, end_pos: usize, style: &TextStyle) {
        self.section_style.add(start_pos, end_pos, style);
        self.invalidate_layout();
        self.needs_sync = true;
    }

    /// Sets cursor position in a text before character at position `pos`.
    /// If pos >= text size then cursor is placed after the latest text character.
    pub fn set_cursor_pos(&mut self, pos: usize) {
        // pos == self.text.len() to select last symbol
        assert!(
            pos <= self.text.len(),
            "Cursor position is outside of the text range: {pos} > {}",
            self.text.len()
        );

        let selection = TextSelection::Cursor(pos);
        if self.selection != selection {
            self.selection = selection;
            self.invalidate_layout();
            self.needs_sync = true;
        }
    }

    pub fn set_selection_color(&mut self, color: &Color) {
        self.selection_color = *color;
        self.invalidate_layout();
        self.needs_sync = true;
    }

    pub fn set_selection(&mut self, start_pos: usize, end_pos: usize) {
        // pos == self.text.len() to select last symbol
        assert!(start_pos <= self.text.len());
        assert!(end_pos <= self.text.len());

        let selection = TextSelection::selection(start_pos, end_pos);
        if self.selection != selection {
            self.selection = selection;
            self.invalidate_layout();
            self.needs_sync = true;
        }
    }
}

impl TextData {
    /// Whether the text data was changed through its API since the last call,
    /// and forget it: `Gui:textView` copies such changes into its view.
    pub fn take_needs_sync(&mut self) -> bool {
        std::mem::take(&mut self.needs_sync)
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

    pub fn set_max_advance(&mut self, max_advance: f32) {
        if self.max_advance != max_advance {
            self.max_advance = max_advance;
            self.invalidate_layout();
        }
    }

    pub fn invalidate_layout(&mut self) {
        self.rebuild_layout = true;
        self.rebuild_line_breaks = true;
    }

    pub(super) fn update(&mut self, text_data: &TextData) {
        let mut updated = if self.text != text_data.text {
            self.text.clone_from(&text_data.text);
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

        updated |= if self.selection_color != text_data.selection_color {
            self.selection_color = text_data.selection_color;
            true
        } else {
            false
        };

        // a shorter text: the cursor or selection must stay inside it
        if self.selection.end() > self.text.len() || self.selection.start() > self.text.len() {
            self.selection = TextSelection::Cursor(self.text.len());
            updated = true;
        }

        if updated {
            self.invalidate_layout();
        }
    }

    pub fn calculate_rect(
        &mut self,
        text_ctx: &mut TextContext,
        scale_factor: f32,
    ) -> Option<Vec2> {
        if self.multiline {
            // size of the multiline text should be specified explicitly
            return None;
        }

        self.set_scale_factor(scale_factor);
        self.update_text_layout(text_ctx);

        let width = self.layout.width().ceil() + self.padding * 2.0;
        let height = self.layout.height().ceil();

        Some(Vec2::new(width, height))
    }

    /// Generate Tex2D texture with layouted text based on text parameters.
    // TODO: keeping a texture for a large texts will be memory consuming.
    // Generate per-line textures and keep only visible ones with some buffered pre- and post-lines.
    pub fn render(
        &mut self,
        r: &mut Renderer,
        text_ctx: &mut TextContext,
        width: f32,
        scale_factor: f32,
        mut widget_pos: Vec2,
        input: &Input,
        editable: bool,
        focused: bool,
        clipboard: &mut String,
    ) -> Option<Tex2D> {
        if editable && focused {
            self.process_text_edit(input, clipboard);
        }

        let mut selection_changed = false;
        if focused && !self.text.is_empty() {
            widget_pos.x += self.padding;

            selection_changed = self.selection.update(
                &self.layout,
                widget_pos,
                input,
                &self.text,
                &mut self.mouse_pos,
            );

            if selection_changed {
                self.invalidate_layout();
            }
        }

        self.set_scale_factor(scale_factor);
        self.set_max_advance(width);

        self.update_text_layout(text_ctx);
        if !std::mem::take(&mut self.texture_stale) {
            return None; // the texture shows the current layout
        }

        // Create buffer to render into
        let width = (self.layout.width().ceil() + self.padding * 2.0) as u32;
        let height = self.layout.height().ceil() as u32;
        let mut buffer = vec![Color::TRANSPARENT; (width * height) as usize];
        let selection_end = self.selection.end();

        // The selection: a highlight behind the selected text, which keeps its colour.
        if let Some(range) = self.selection.get_forward_range() {
            self.render_selection(&mut buffer, width, height, &range);
        }

        // Iterate over laid out lines
        for line in self.layout.lines() {
            // Iterate over GlyphRun's within each line
            for glyph_run in line.glyph_runs() {
                self.render_glyph_run(&mut text_ctx.scale, &glyph_run, &mut buffer, width);
            }
        }

        // calculate cursor rect
        if (self.text_changed || selection_changed) && editable && focused {
            self.cursor_rect.build(
                &self.layout,
                height,
                selection_end,
                self.padding,
                self.text.len(),
            );
        }

        // Create texture
        let mut tex = Tex2D::new(r, width as i32, height as i32, TexFormat::RGBA8);
        tex.set_data(r, &buffer, PixelFormat::RGBA, DataFormat::Float);
        Some(tex)
    }

    fn update_text_layout(&mut self, text_ctx: &mut TextContext) -> bool {
        if self.rebuild_layout {
            self.rebuild_layout = false;

            let mut builder =
                text_ctx
                    .layout
                    .ranged_builder(&mut text_ctx.font, &self.text, self.scale_factor);

            self.default_style.apply_default(&mut builder);

            self.section_style.apply(&mut builder);

            // Build the builder into a Layout
            builder.build_into(&mut self.layout);
            self.texture_stale = true;

            if self.rebuild_line_breaks {
                self.rebuild_line_breaks = false;

                // The width for line wrapping
                let max_advance = if self.multiline && self.max_advance > 0.0 {
                    Some(self.max_advance * self.scale_factor)
                } else {
                    None
                };

                // Perform layout (including bidi resolution and shaping) with alignment
                self.layout.break_all_lines(max_advance, self.alignment);
            }

            true
        } else {
            false
        }
    }

    fn process_text_edit(&mut self, input: &Input, clipboard: &mut String) {
        let mut text_changed = false;

        // remove backspace, del and new line characters from the text input
        // TODO: remove all non-printable characters
        let chars_to_remove: &[char] = if self.multiline {
            &['\u{1b}', '\u{7f}', '\u{8}']
        } else {
            &['\u{1b}', '\u{7f}', '\u{8}', '\n', '\r']
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

        let insert_from_clipboard = input.is_keyboard_ctrl_down()
            && input.is_pressed(Button::KeyboardV)
            || input.is_keyboard_shift_down() && input.is_pressed(Button::KeyboardInsert);

        if typed_text.is_empty() && insert_from_clipboard {
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

            text_changed = true;
        } else {
            let cut_to_clipboard =
                input.is_keyboard_ctrl_down() && input.is_pressed(Button::KeyboardX);
            let copy_to_clipboard =
                input.is_keyboard_ctrl_down() && input.is_pressed(Button::KeyboardC);

            if input.is_pressed(Button::KeyboardBackspace)
                || input.is_pressed(Button::KeyboardDelete)
                || cut_to_clipboard
                || copy_to_clipboard
            {
                match &mut self.selection {
                    TextSelection::Cursor(pos) => {
                        if input.is_pressed(Button::KeyboardBackspace) {
                            if *pos > 0 {
                                *pos -= 1;
                                self.text.remove(*pos);
                                text_changed = true;

                                if !self.section_style.is_empty() {
                                    change = Some((*pos, 1, 0));
                                }
                            }
                        } else if input.is_pressed(Button::KeyboardDelete) && *pos < self.text.len()
                        {
                            self.text.remove(*pos);
                            text_changed = true;

                            if !self.section_style.is_empty() {
                                change = Some((*pos, 1, 0));
                            }
                        }
                    }
                    TextSelection::Selection(range) => {
                        let (start, end) = if range.start < range.end {
                            (range.start, range.end)
                        } else {
                            (range.end, range.start)
                        };

                        if cut_to_clipboard || copy_to_clipboard {
                            *clipboard = self.text[start..end].into();
                        }

                        if !copy_to_clipboard {
                            self.text.replace_range(start..end, "");
                            self.selection = TextSelection::Cursor(start);
                            text_changed = true;

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

        if text_changed {
            self.text_changed = true;
            self.invalidate_layout();
        }
    }

    /// Fill the background of the selected characters with the selection
    /// colour, line by line (from the top of a line's leading to its bottom).
    fn render_selection(
        &self,
        buffer: &mut [Color],
        image_width: u32,
        image_height: u32,
        range: &std::ops::Range<usize>,
    ) {
        for line in self.layout.lines() {
            let metrics = line.metrics();
            let top = (metrics.baseline - metrics.ascent - metrics.leading * 0.5)
                .floor()
                .max(0.0) as u32;
            let bottom = ((metrics.baseline + metrics.descent + metrics.leading * 0.5).ceil()
                as u32)
                .min(image_height);

            // the selected characters as spans from left to right, neighbours
            // joined, so each pixel is painted once
            let mut spans: Vec<(f32, f32)> = vec![];
            let mut x = metrics.offset + self.padding;
            for run in line.runs() {
                for cluster in run.visual_clusters() {
                    let advance = cluster.advance();
                    let text_range = cluster.text_range();

                    if text_range.start < range.end && text_range.end > range.start {
                        match spans.last_mut() {
                            Some(span) if (span.1 - x).abs() < 0.01 => span.1 = x + advance,
                            _ => spans.push((x, x + advance)),
                        }
                    }

                    x += advance;
                }
            }

            for (start, end) in spans {
                let left = start.round().max(0.0) as usize;
                let right = (end.round().max(0.0) as usize).min(image_width as usize);

                for y in top..bottom {
                    let row = (y * image_width) as usize;
                    for pixel in &mut buffer[row + left..row + right.max(left)] {
                        pixel.paint_over(&self.selection_color);
                    }
                }
            }
        }
    }

    fn render_glyph_run(
        &self,
        context: &mut ScaleContext,
        glyph_run: &GlyphRun<Color>,
        buffer: &mut [Color],
        image_width: u32,
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
            let glyph_x = run_x + glyph.x + self.padding;
            let glyph_y = run_y - glyph.y;

            run_x += glyph.advance;

            render_glyph(
                buffer,
                &mut scaler,
                &color,
                &glyph,
                glyph_x,
                glyph_y,
                image_width,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data(text: &str) -> TextData {
        TextData::new(
            text,
            &TextStyle::default(),
            &Color::WHITE,
            &Color::WHITE,
            TextAlignment::Start,
            false,
        )
    }

    #[test]
    fn changes_through_the_api_are_copied_once() {
        let mut lua = data("123");
        assert!(
            lua.take_needs_sync(),
            "new text data is copied into its view"
        );
        assert!(!lua.take_needs_sync());

        lua.set_text("456");
        assert!(lua.take_needs_sync());

        lua.set_text("456"); // the same text: nothing changed
        assert!(!lua.take_needs_sync());
    }

    #[test]
    fn a_shorter_text_keeps_the_cursor_inside_it() {
        let mut view = data("12345678");
        view.set_cursor_pos(8);

        view.update(&data("123"));
        assert_eq!(view.selection(), &TextSelection::Cursor(3));
        assert_eq!(view.text(), "123");
    }
}
