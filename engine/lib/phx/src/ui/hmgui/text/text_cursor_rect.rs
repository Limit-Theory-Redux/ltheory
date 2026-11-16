use glam::Vec2;
use parley::{Affinity, Cursor};

use super::TextLayout;
use crate::render::Color;

/// Width of the text cursor, in pixels.
const CURSOR_WIDTH: f32 = 3.0;

/// Rectangular area that represents cursor current position in text editing.
#[derive(Debug, Clone, PartialEq)]
pub struct TextCursorRect {
    pos: Vec2,
    size: Vec2,
    color: Color,
}

impl TextCursorRect {
    pub fn new(color: &Color) -> Self {
        Self {
            pos: Default::default(),
            size: Default::default(),
            color: *color,
        }
    }

    pub fn pos(&self) -> Vec2 {
        self.pos
    }

    pub fn size(&self) -> Vec2 {
        self.size
    }

    pub fn color(&self) -> Color {
        self.color
    }

    /// Rebuild cursor rect based on it's position in the text layout.
    pub fn build(
        &mut self,
        layout: &TextLayout,
        widget_height: u32,
        cursor_position: usize,
        padding: f32,
    ) {
        let cursor = Cursor::from_byte_index(layout, cursor_position, Affinity::Downstream);
        let rect = cursor.geometry(layout, CURSOR_WIDTH);
        let top = (rect.y0 as f32).floor().max(0.0);
        let bottom = (rect.y1 as f32).floor().min(widget_height as f32);

        self.pos = Vec2::new(rect.x0 as f32 + padding, top);
        self.size = Vec2::new(CURSOR_WIDTH, (bottom - top).max(0.0));
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use parley::style::{FontFamily, StyleProperty};

    use super::*;
    use crate::ui::hmgui::TextContext;

    fn layout(text: &str) -> TextLayout {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../res/font");
        let mut ctx = TextContext::with_font_dirs([dir]);
        let TextContext { font, layout, .. } = &mut ctx;
        let mut builder = layout.ranged_builder(font, text, 1.0, true);
        builder.push_default(FontFamily::from("Exo2"));
        builder.push_default(StyleProperty::FontSize(20.0));
        let mut layout = builder.build(text);
        layout.break_all_lines(None);
        layout
    }

    fn cursor_at(layout: &TextLayout, pos: usize) -> TextCursorRect {
        let mut rect = TextCursorRect::new(&Color::WHITE);
        rect.build(layout, layout.height().ceil() as u32, pos, 5.0);
        rect
    }

    #[test]
    fn cursor_follows_the_text() {
        let layout = layout("ab\ncd");
        let height = layout.height().ceil();

        let start = cursor_at(&layout, 0);
        assert_eq!(start.pos(), Vec2::new(5.0, 0.0), "padding, first line");
        assert_eq!(start.size().x, CURSOR_WIDTH);
        assert!(start.size().y > 0.0 && start.size().y <= height);

        let after_b = cursor_at(&layout, 2);
        assert!(
            after_b.pos().x > start.pos().x,
            "moves right along the line"
        );
        assert_eq!(after_b.pos().y, 0.0);

        let second_line = cursor_at(&layout, 3);
        assert_eq!(second_line.pos().x, 5.0, "a new line starts at the left");
        assert!(second_line.pos().y > 0.0, "and below the first");

        let end = cursor_at(&layout, 5);
        assert_eq!(end.pos().y, second_line.pos().y);
        assert!(
            end.pos().y + end.size().y <= height,
            "stays inside the text"
        );
    }
}
