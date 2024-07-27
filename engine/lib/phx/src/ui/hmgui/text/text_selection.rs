use std::ops::Range;

use glam::Vec2;
use parley::layout::Cursor;

use super::TextLayout;
use crate::input::{Button, Input};

const NEWLINE_SEPARATORS: &[char] = &['\n', '\r'];

/// Text selection range or simple cursor position in a text.
#[derive(Clone, Debug, PartialEq)]
pub enum TextSelection {
    /// No real selection, only cursor position.
    Cursor(usize),
    /// Start and end position of the selection.
    /// Start can be bigger than end if selection was done from right to left.
    Selection(Range<usize>),
}

impl Default for TextSelection {
    fn default() -> Self {
        Self::Cursor(0)
    }
}

impl TextSelection {
    /// Create text selection variant based on start and end positions.
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

    /// Returns selection range in correct order (start < end) or None if it is cursor.
    pub fn get_forward_range(&self) -> Option<Range<usize>> {
        if let Self::Selection(range) = self {
            if range.start <= range.end {
                Some(range.clone())
            } else {
                Some(Range {
                    start: range.end,
                    end: range.start,
                })
            }
        } else {
            None
        }
    }

    /// Returns if selection start < end or true for the cursor.
    pub fn is_forward(&self) -> bool {
        match self {
            Self::Cursor(_) => true,
            Self::Selection(range) => range.start <= range.end,
        }
    }

    /// Returns if selection start > end or true for the cursor.
    pub fn is_backward(&self) -> bool {
        match self {
            Self::Cursor(_) => true,
            Self::Selection(range) => range.start > range.end,
        }
    }

    /// Creates a new text selection where start is always >= end.
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

    /// Make self a cursor.
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

    /// Use user input (keyboard and mouse) to update cursor position.
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

            self.on_mouse(layout, widget_mouse_pos, input, text.len());

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

    fn on_mouse(
        &mut self,
        layout: &TextLayout,
        widget_mouse_pos: Vec2,
        input: &Input,
        text_len: usize,
    ) {
        let cursor = Cursor::from_point(layout, widget_mouse_pos.x, widget_mouse_pos.y);

        if input.is_pressed(Button::MouseLeft) {
            if input.is_keyboard_shift_down() {
                if self.is_forward() && cursor.text_end >= text_len {
                    // to allow last text symbol to be selected
                    self.set_end(cursor.text_end);
                } else {
                    // this is needed to avoid jumping cursor to the start of the next line
                    // while Shift+clicking outside of the non-last line
                    self.set_end(cursor.text_start);
                }
            } else if cursor.text_end >= text_len {
                // to allow last text symbol to be selected
                self.set_cursor(cursor.text_end);
            } else {
                // this is needed to avoid jumping cursor to the start of the next line
                // while clicking outside of the non-last line
                self.set_cursor(cursor.text_start);
            }
        } else if self.is_forward() && cursor.text_end >= text_len {
            // TODO: there is a problem with condition above -
            // it keeps either 1 or 2 last symbols unselected depending on
            // if >= or > comparison is used
            self.set_end(cursor.text_end);
        } else {
            // this is needed to avoid jumping cursor to the start of the next line
            // while dragging mouse outside of the non-last line
            self.set_end(cursor.text_start);
        }
    }

    fn on_kb_left(&mut self, input: &Input) -> bool {
        let cursor_position = self.cursor_position();
        if cursor_position > 0 {
            if input.is_keyboard_shift_down() {
                self.set_end(cursor_position - 1);
            } else {
                let pos = match &self {
                    TextSelection::Cursor(pos) => *pos - 1,
                    TextSelection::Selection(range) => {
                        if range.start < range.end {
                            range.start
                        } else {
                            range.end
                        }
                    }
                };
                self.set_cursor(pos);
            }
            true
        } else {
            false
        }
    }

    fn on_kb_right(&mut self, input: &Input, text: &str) -> bool {
        let cursor_position = self.cursor_position();
        if cursor_position < text.len() {
            if input.is_keyboard_shift_down() {
                self.set_end(cursor_position + 1);
            } else {
                let pos = match &self {
                    TextSelection::Cursor(pos) => *pos + 1,
                    TextSelection::Selection(range) => {
                        if range.start < range.end {
                            range.end
                        } else {
                            range.start
                        }
                    }
                };
                self.set_cursor(pos);
            }
            true
        } else {
            false
        }
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

/// Search for the index of the previous character in the text
/// that is in the `chars` list, starting at the position `pos`.
/// Can be used for example for searching the end of the previous line or start of the word.
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

/// Search for the index of the next character in the text
/// that is in the `chars` list, starting at the position `pos`.
/// Can be used for example for searching the end of the current line or end of the word.
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
        .unwrap_or(text.len())
}
