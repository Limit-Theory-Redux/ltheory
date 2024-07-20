use std::ops::Range;

use glam::Vec2;
use parley::layout::Cursor;

use crate::input::{Button, Input};

use super::TextLayout;

const NEWLINE_SEPARATORS: &[char] = &['\n', '\r'];

#[derive(Clone, Debug, PartialEq)]
pub enum TextSelection {
    /// No real selection, only cursor position.
    Cursor(usize),
    /// Start and end position of the selection.
    /// Start can be bigger than end if selection was done from right to left.
    Selection(Range<usize>),
}

impl TextSelection {
    pub fn new() -> Self {
        Self::Cursor(0)
    }

    pub fn selection(start: usize, end: usize) -> Self {
        Self::Selection(Range { start, end })
    }

    pub fn start(&self) -> usize {
        match self {
            Self::Cursor(pos) => *pos,
            Self::Selection(range) => range.start,
        }
    }

    pub fn end(&self) -> usize {
        match self {
            Self::Cursor(pos) => *pos,
            Self::Selection(range) => range.end,
        }
    }

    pub fn is_cursor(&self) -> bool {
        matches!(self, Self::Cursor(_))
    }

    pub fn is_selection(&self) -> bool {
        matches!(self, Self::Selection(_))
    }

    /// Returns cursor position or end position of the selection.
    pub fn cursor_position(&self) -> usize {
        self.end()
    }

    pub fn get_forward_range(&self) -> Range<usize> {
        match self {
            Self::Cursor(pos) => Range {
                start: *pos,
                end: *pos,
            },
            Self::Selection(range) => {
                if range.start <= range.end {
                    range.clone()
                } else {
                    Range {
                        start: range.end,
                        end: range.start,
                    }
                }
            }
        }
    }

    pub fn is_forward(&self) -> bool {
        match self {
            Self::Cursor(_) => true,
            Self::Selection(range) => range.start <= range.end,
        }
    }

    pub fn is_backward(&self) -> bool {
        match self {
            Self::Cursor(_) => true,
            Self::Selection(range) => range.start > range.end,
        }
    }

    /// Create a new text selection where start is always >= end.
    pub fn normalized(&self) -> Self {
        match self {
            Self::Cursor(pos) => Self::Cursor(*pos),
            Self::Selection(range) => {
                if range.start <= range.end {
                    Self::Selection(range.clone())
                } else {
                    Self::selection(range.end, range.start)
                }
            }
        }
    }

    pub fn set_cursor(&mut self, pos: usize) {
        *self = Self::Cursor(pos);
    }

    /// Sets either cursor position or end position of the selection.
    /// Converts selection into cursor position if start == end.
    pub fn set_position(&mut self, end_pos: usize) {
        *self = match self {
            Self::Cursor(start_pos) => Self::Cursor(*start_pos),
            Self::Selection(range) => {
                if range.start == end_pos {
                    Self::Cursor(range.start)
                } else {
                    Self::selection(range.start, end_pos)
                }
            }
        };
    }

    /// Keeps cursor position or sets start position of the selection.
    /// Converts selection into cursor position if start == end.
    pub fn set_start(&mut self, start_pos: usize) {
        *self = match self {
            Self::Cursor(_) => Self::Cursor(start_pos),
            Self::Selection(range) => {
                if start_pos == range.end {
                    Self::Cursor(start_pos)
                } else {
                    Self::selection(start_pos, range.end)
                }
            }
        };
    }

    /// Converts cursor into selection by adding end position or
    /// update end position of the selection.
    /// Converts selection into cursor position if start == end.
    pub fn set_end(&mut self, end_pos: usize) {
        *self = match self {
            Self::Cursor(pos) => {
                if *pos == end_pos {
                    Self::Cursor(*pos)
                } else {
                    Self::selection(*pos, end_pos)
                }
            }
            Self::Selection(range) => {
                if range.start == end_pos {
                    Self::Cursor(end_pos)
                } else {
                    Self::selection(range.start, end_pos)
                }
            }
        };
    }

    pub fn update(
        &mut self,
        layout: &TextLayout,
        widget_pos: Vec2,
        input: &Input,
        text: &str,
        mouse_pos: &mut Vec2,
    ) -> bool {
        let cur_mouse_pos = input.mouse().position();

        if (input.is_pressed(Button::MouseLeft) || input.is_down(Button::MouseLeft))
            && *mouse_pos != cur_mouse_pos
        {
            let widget_mouse_pos = cur_mouse_pos - widget_pos;

            self.on_mouse(layout, widget_mouse_pos, input, text);

            *mouse_pos = cur_mouse_pos;

            return true;
        }

        // Ctrl+A - select all text
        if input.is_keyboard_ctrl_down() && input.is_pressed(Button::KeyboardA) {
            self.set_start(0);
            self.set_end(text.len());
            return true;
        }

        if input.is_pressed(Button::KeyboardLeft) {
            return self.on_kb_left(input);
        }

        if input.is_pressed(Button::KeyboardRight) {
            return self.on_kb_right(input, text);
        }

        if input.is_pressed(Button::KeyboardUp) {
            self.on_kb_up(input, text);
            return true;
        }

        if input.is_pressed(Button::KeyboardDown) {
            self.on_kb_down(input, text);
            return true;
        }

        if input.is_pressed(Button::KeyboardHome) {
            self.on_kb_home(input, text);
            return true;
        }

        if input.is_pressed(Button::KeyboardEnd) {
            self.on_kb_end(input, text);
            return true;
        }

        false
    }

    fn on_mouse(&mut self, layout: &TextLayout, widget_mouse_pos: Vec2, input: &Input, text: &str) {
        let cursor = Cursor::from_point(layout, widget_mouse_pos.x, widget_mouse_pos.y);

        let pos = if self.is_forward() && cursor.text_end >= text.len() {
            cursor.text_end
        } else {
            cursor.text_start
        };

        if input.is_pressed(Button::MouseLeft) {
            if input.is_keyboard_shift_down() {
                self.set_end(pos);
            } else {
                self.set_cursor(pos);
            }
        } else {
            self.set_end(pos);
        }
    }

    fn on_kb_left(&mut self, input: &Input) -> bool {
        let cursor_position = self.cursor_position();
        if cursor_position > 0 {
            if input.is_keyboard_shift_down() {
                self.set_end(cursor_position - 1);
            } else {
                match &self {
                    TextSelection::Cursor(pos) => self.set_cursor(*pos - 1),
                    TextSelection::Selection(range) => {
                        if range.start < range.end {
                            self.set_cursor(range.start);
                        } else {
                            self.set_cursor(range.end);
                        }
                    }
                }
            }
            return true;
        }

        if self.is_selection() {
            self.set_cursor(0);
            return true;
        }

        false
    }

    fn on_kb_right(&mut self, input: &Input, text: &str) -> bool {
        let cursor_position = self.cursor_position();
        if cursor_position < text.len() {
            if input.is_keyboard_shift_down() {
                self.set_end(cursor_position + 1);
            } else {
                match &self {
                    TextSelection::Cursor(pos) => self.set_cursor(*pos + 1),
                    TextSelection::Selection(range) => {
                        if range.start < range.end {
                            self.set_cursor(range.end);
                        } else {
                            self.set_cursor(range.start);
                        }
                    }
                }
            }
            return true;
        }

        if self.is_selection() {
            self.set_cursor(text.len());
            return true;
        }

        false
    }

    fn on_kb_up(&mut self, input: &Input, text: &str) {
        let line_end = find_prev_char_pos(text, self.end(), NEWLINE_SEPARATORS);
        let line_start = find_prev_char_pos(text, line_end.saturating_sub(1), NEWLINE_SEPARATORS);

        let pos = if line_start < line_end {
            let offset = self.end() - line_end;
            let pos = line_start + offset.min(line_end - line_start);

            if pos == line_end {
                pos - 1
            } else {
                pos
            }
        } else {
            0
        };

        if input.is_keyboard_shift_down() {
            self.set_end(pos);
        } else {
            self.set_cursor(pos);
        }
    }

    fn on_kb_down(&mut self, input: &Input, text: &str) {
        let line_start = find_next_char_pos(text, self.end(), NEWLINE_SEPARATORS) + 1;
        let line_end = find_next_char_pos(text, line_start + 1, NEWLINE_SEPARATORS);

        let pos = if line_start < line_end {
            let cur_line_start = find_prev_char_pos(text, self.end(), NEWLINE_SEPARATORS);
            let offset = self.end() - cur_line_start;
            let pos = line_start + offset.min(line_end - line_start);

            if pos == line_end {
                pos - 1
            } else {
                pos
            }
        } else {
            text.len()
        };

        if input.is_keyboard_shift_down() {
            self.set_end(pos);
        } else {
            self.set_cursor(pos);
        }
    }

    fn on_kb_home(&mut self, input: &Input, text: &str) {
        if input.is_keyboard_ctrl_down() {
            // till the beginning of the text
            if input.is_keyboard_shift_down() {
                self.set_end(0);
            } else {
                self.set_cursor(0);
            }
        } else {
            let cursor_position = find_prev_char_pos(text, self.end(), NEWLINE_SEPARATORS);

            if input.is_keyboard_shift_down() {
                self.set_end(cursor_position);
            } else {
                self.set_cursor(cursor_position);
            }
        }
    }

    fn on_kb_end(&mut self, input: &Input, text: &str) {
        if input.is_keyboard_ctrl_down() {
            // till the end of the text
            if input.is_keyboard_shift_down() {
                self.set_end(text.len());
            } else {
                self.set_cursor(text.len());
            }
        } else {
            let cursor_position = find_next_char_pos(text, self.end(), NEWLINE_SEPARATORS);

            if input.is_keyboard_shift_down() {
                self.set_end(cursor_position);
            } else {
                self.set_cursor(cursor_position);
            }
        }
    }
}

fn find_prev_char_pos(text: &str, pos: usize, chars: &[char]) -> usize {
    text[..pos]
        .chars()
        .rev()
        .enumerate()
        .find_map(|(i, c)| {
            if chars.contains(&c) {
                Some(pos - i)
            } else {
                None
            }
        })
        .unwrap_or(0)
}

fn find_next_char_pos(text: &str, pos: usize, chars: &[char]) -> usize {
    text.chars()
        .skip(pos)
        .enumerate()
        .find_map(|(i, c)| {
            if chars.contains(&c) {
                Some(pos + i)
            } else {
                None
            }
        })
        .unwrap_or_else(|| text.len())
}
