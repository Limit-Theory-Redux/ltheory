use glam::Vec2;

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
}

impl PartialEq for HmGuiText {
    fn eq(&self, other: &Self) -> bool {
        self.font.name() == other.font.name()
            && self.text == other.text
            && self.color == other.color
            && self.outline == other.outline
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
        }
    }

    /// Measure the text (its own size, for the widget's min size).
    pub fn measure(&mut self, r: &mut Renderer) -> Vec2 {
        self.size = self.font.get_size2(r, &self.text).as_vec2();
        self.size
    }

    /// Fit the text to a width: when it is wider (by more than half a pixel, so
    /// rounding in the layout doesn't cut a text that fits), keep as much of its
    /// start as fits together with [`ELLIPSIS`].
    pub fn fit(&mut self, r: &mut Renderer, width: f32) {
        self.shown = if self.size.x - width > 0.5 {
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

    pub fn draw(&self, renderer: &mut UIRenderer, pos: Vec2) {
        renderer.text(&self.font, self.shown_text(), pos, self.color, self.outline);
    }

    // For testing.
    #[allow(dead_code)]
    pub(crate) fn dump(&self, ident: usize) {
        let ident_str = crate::ui::hmgui::IDENT.repeat(ident).to_string();

        println!("{ident_str}- text:  {}", self.text);
        println!("{ident_str}- shown: {:?}", self.shown);
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
