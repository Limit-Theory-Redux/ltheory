use glam::Vec2;

use super::Alignment;
use crate::render::{Color, Font, Renderer, TextOutline, UIRenderer};

/// What a text widget that is narrower than its text ends with.
pub const ELLIPSIS: &str = "…";

#[derive(Clone)]
pub struct HmGuiText {
    pub font: Font,
    pub text: String,
    pub color: Color,
    /// Drawn with an outline and halo (`Gui:textOutlined`).
    pub outline: Option<TextOutline>,
    /// The text's own size (without the widget's min/fixed size), from compute_size().
    pub size: Vec2,
    /// The text cut to the widget's width and ended with [`ELLIPSIS`], when the
    /// widget is narrower than its text (from layout()).
    pub shown: Option<String>,
    /// Break the text into lines where it is wider than the widget's max or
    /// fixed width (`Gui:setTextWrap`).
    pub wrap: bool,
    /// Where each line sits in the widget (`Gui:setTextAlign`): begin = left,
    /// end = right, anything else centred.
    pub align: Alignment,
    /// The text's lines and their widths, when it has more than one (line
    /// breaks, or `wrap`), from measure(). Empty for a single line.
    pub lines: Vec<(String, f32)>,
}

impl PartialEq for HmGuiText {
    fn eq(&self, other: &Self) -> bool {
        self.font.name() == other.font.name()
            && self.text == other.text
            && self.color == other.color
            && self.outline == other.outline
            && self.wrap == other.wrap
            && self.align == other.align
    }
}

impl HmGuiText {
    pub fn new(text: &str, font: &Font, color: &Color) -> Self {
        Self {
            font: font.clone(),
            text: text.into(),
            color: *color,
            outline: None,
            size: Vec2::ZERO,
            shown: None,
            wrap: false,
            align: Alignment::Default,
            lines: vec![],
        }
    }

    /// Measure the text (its own size, for the widget's min size). It breaks
    /// into lines at its line breaks, and with `wrap` where it is wider than
    /// `wrap_width` (the widget's max or fixed width): then it is as wide as
    /// its widest line and as high as its lines, a line height apart.
    pub fn measure(&mut self, r: &mut Renderer, wrap_width: Option<f32>) -> Vec2 {
        let wrap_width = if self.wrap { wrap_width } else { None };
        self.lines.clear();

        if wrap_width.is_some() || self.text.contains('\n') {
            let lines = self.font.wrap(&self.text, wrap_width);
            if lines.len() > 1 {
                let mut width: f32 = 0.0;
                for line in lines.iter() {
                    let line_width = self.font.get_size2(r, line).x as f32;
                    width = width.max(line_width);
                    self.lines.push((line.clone(), line_width));
                }
                let line_box = (self.font.ascent() + self.font.descent()) as f32;
                let between = self.font.get_line_height() as f32;
                self.size = Vec2::new(width, line_box + (lines.len() - 1) as f32 * between);
                return self.size;
            }
        }

        self.size = self.font.get_size2(r, &self.text).as_vec2();
        self.size
    }

    /// Fit the text to a width: when it is wider (by more than half a pixel, so
    /// rounding in the layout doesn't cut a text that fits), keep as much of its
    /// start as fits together with [`ELLIPSIS`].
    pub fn fit(&mut self, r: &mut Renderer, width: f32) {
        self.shown = if self.lines.is_empty() && self.size.x - width > 0.5 {
            Some(fit_text(&self.text, width, |s| {
                self.font.get_size2(r, s).x as f32
            }))
        } else {
            None
        };
    }

    /// The text as drawn: cut with [`ELLIPSIS`] if it doesn't fit.
    pub fn shown_text(&self) -> &str {
        self.shown.as_deref().unwrap_or(&self.text)
    }

    /// Draw the text in a widget box starting at `x`, `width` wide, whose
    /// bottom is at `bottom`. The box is the font's line box (ascent +
    /// descent) per line, so the last line's baseline sits `descent` above the
    /// bottom and every line above it a line height higher. Each line is
    /// placed by `align` (centred by default); a cut text fills the box from
    /// the left.
    pub fn draw(&self, renderer: &mut UIRenderer, x: f32, width: f32, bottom: f32) {
        let baseline = bottom - self.font.descent() as f32;

        if self.lines.is_empty() {
            let x = if self.shown.is_some() {
                x
            } else {
                self.line_x(x, width, self.size.x)
            };
            let pos = Vec2::new(x, baseline);
            renderer.text(&self.font, self.shown_text(), pos, self.color, self.outline);
            return;
        }

        let between = self.font.get_line_height() as f32;
        let last = self.lines.len() - 1;
        for (i, (line, line_width)) in self.lines.iter().enumerate() {
            let pos = Vec2::new(
                self.line_x(x, width, *line_width),
                baseline - (last - i) as f32 * between,
            );
            renderer.text(&self.font, line, pos, self.color, self.outline);
        }
    }

    /// Where a line `line_width` wide starts in a box at `x`, `width` wide.
    fn line_x(&self, x: f32, width: f32, line_width: f32) -> f32 {
        if self.align.is_begin() && !self.align.is_default() {
            x
        } else if self.align.is_end() {
            x + width - line_width
        } else {
            x + (width - line_width) / 2.0
        }
    }

    // For testing.
    #[allow(dead_code)]
    pub(crate) fn dump(&self, ident: usize) {
        let ident_str = crate::ui::hmgui::IDENT.repeat(ident).to_string();

        println!("{ident_str}- text:  {}", self.text);
        println!("{ident_str}- shown: {:?}", self.shown);
        if !self.lines.is_empty() {
            println!("{ident_str}- lines: {:?}", self.lines);
        }
        println!("{ident_str}- color: {:?}", self.color);
        if let Some(outline) = &self.outline {
            println!("{ident_str}- outline: {outline:?}");
        }
        println!("{ident_str}- font:  {:?}", self.font.name());
    }
}

/// The longest start of `text` that fits `width` with [`ELLIPSIS`] after it
/// (trailing spaces dropped), by the `width_of` measure. Just the ellipsis if
/// no character fits, and nothing if not even the ellipsis does.
fn fit_text(text: &str, width: f32, mut width_of: impl FnMut(&str) -> f32) -> String {
    let with_ellipsis = |end: usize| format!("{}{ELLIPSIS}", text[..end].trim_end());

    // character boundaries after the first character, up to the last one
    let ends: Vec<usize> = text.char_indices().map(|(i, _)| i).skip(1).collect();

    // binary search for the most characters that fit
    let (mut lo, mut hi) = (0, ends.len());
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        if width_of(&with_ellipsis(ends[mid - 1])) <= width {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }

    if lo > 0 {
        with_ellipsis(ends[lo - 1])
    } else if width_of(ELLIPSIS) <= width {
        ELLIPSIS.into()
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // every character (and the ellipsis) 10 wide
    fn chars(s: &str) -> f32 {
        s.chars().count() as f32 * 10.0
    }

    #[test]
    fn fit_text_keeps_the_longest_start_that_fits() {
        assert_eq!(fit_text("Fullscreen Windowed", 75.0, chars), "Fullsc…");
        assert_eq!(fit_text("Fullscreen Windowed", 120.0, chars), "Fullscreen…");
        // the space before the cut goes
        assert_eq!(fit_text("Fullscreen Windowed", 115.0, chars), "Fullscreen…");
        assert_eq!(fit_text("abc", 25.0, chars), "a…");
    }

    #[test]
    fn fit_text_at_the_extremes() {
        assert_eq!(fit_text("abc", 10.0, chars), ELLIPSIS);
        assert_eq!(fit_text("abc", 5.0, chars), "");
        assert_eq!(fit_text("", 5.0, chars), "");
    }

    #[test]
    fn fit_text_cuts_between_characters() {
        // multi-byte characters are never split
        assert_eq!(fit_text("éèêë", 30.0, chars), "éè…");
        assert_eq!(fit_text("70°", 20.0, chars), "7…");
    }
}
