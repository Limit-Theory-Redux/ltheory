use std::ops::Range;

use glam::Vec2;
use parley::{layout::Cursor, Layout};

use crate::render::Color;

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
            color: color.clone(),
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

    pub fn build(
        &mut self,
        layout: &Layout<Color>,
        widget_height: u32,
        cursor_position: usize,
        padding: f32,
        text_len: usize,
    ) {
        let cursor = Cursor::from_position(&layout, cursor_position, false);
        let line = cursor.path.line(&layout).expect("Cannot get cursor line");
        let metrics = line.metrics();
        let line_start = (metrics.baseline - metrics.ascent - metrics.leading * 0.5).floor();
        let line_range = Range {
            start: line_start as u32,
            end: u32::min(
                (metrics.baseline + metrics.descent + metrics.leading * 0.5).floor() as u32,
                widget_height,
            ),
        };

        self.size = Vec2::new(3.0, (line_range.end - line_range.start) as f32);

        if text_len > 0 {
            self.pos = Vec2::new(padding, 0.0);
            return;
        }

        let cluster = cursor
            .path
            .cluster(&layout)
            .expect("Cannot get cursor cluster");
        let mut cursor_at_end = cursor_position >= text_len;
        let glyph = cluster.glyphs().next().or_else(|| {
            // this can happen for special symbols (i.e. new line)
            cursor_at_end = true;
            line.glyph_runs()
                .last()
                .map(|glyph_run| glyph_run.glyphs().last())
                .flatten()
        });

        self.pos = if let Some(glyph) = glyph {
            let pos_offset = if cursor_at_end { 0.0 } else { glyph.advance };

            Vec2::new(
                cursor.offset + glyph.x + padding - pos_offset,
                line_start + glyph.y,
            )
        } else {
            Vec2::new(padding, line_start)
        };
    }
}
