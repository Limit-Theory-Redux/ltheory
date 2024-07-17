use std::ops::Range;

use glam::Vec2;
use parley::{layout::Cursor, Layout};

use crate::input::{Button, Input};
use crate::render::Color;

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

    pub fn range(&self) -> Range<usize> {
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
                    Self::Selection(Range {
                        start: range.end,
                        end: range.start,
                    })
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
                    Self::Selection(Range {
                        start: range.start,
                        end: end_pos,
                    })
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
                    Self::Selection(Range {
                        start: start_pos,
                        end: range.end,
                    })
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
                    Self::Selection(Range {
                        start: *pos,
                        end: end_pos,
                    })
                }
            }
            Self::Selection(range) => {
                if range.start == end_pos {
                    Self::Cursor(end_pos)
                } else {
                    Self::Selection(Range {
                        start: range.start,
                        end: end_pos,
                    })
                }
            }
        };
    }

    pub fn update(
        &mut self,
        layout: &Layout<Color>,
        widget_pos: Vec2,
        input: &Input,
        text_len: usize,
        mouse_pos: &mut Vec2,
    ) -> bool {
        let cur_mouse_pos = input.mouse().position();

        if (input.is_pressed(Button::MouseLeft)
            || input.is_down(Button::MouseLeft)
            || input.is_released(Button::MouseLeft))
            && *mouse_pos != cur_mouse_pos
        {
            let widget_mouse_pos = cur_mouse_pos - widget_pos;

            self.on_mouse(layout, widget_mouse_pos, input, text_len);

            *mouse_pos = cur_mouse_pos;

            return true;
        }

        // Ctrl+A - select all text
        if input.is_keyboard_ctrl_down() && input.is_pressed(Button::KeyboardA) {
            self.set_start(0);
            self.set_end(text_len);
            return true;
        }

        if input.is_pressed(Button::KeyboardLeft) {
            return self.on_kb_left(input);
        }

        if input.is_pressed(Button::KeyboardRight) {
            return self.on_kb_right(input, text_len);
        }

        if input.is_pressed(Button::KeyboardUp) {
            self.on_kb_up(layout, input);
            return true;
        }

        if input.is_pressed(Button::KeyboardDown) {
            self.on_kb_down(layout, input, text_len);
            return true;
        }

        if input.is_pressed(Button::KeyboardHome) {
            self.on_kb_home(layout, input);
            return true;
        }

        if input.is_pressed(Button::KeyboardEnd) {
            self.on_kb_end(layout, input, text_len);
            return true;
        }

        false
    }

    fn on_mouse(
        &mut self,
        layout: &Layout<Color>,
        widget_mouse_pos: Vec2,
        input: &Input,
        text_len: usize,
    ) {
        let cursor = Cursor::from_point(layout, widget_mouse_pos.x, widget_mouse_pos.y);

        if input.is_pressed(Button::MouseLeft) && !input.is_keyboard_shift_down() {
            let pos = if cursor.text_end < text_len {
                cursor.text_start
            } else {
                cursor.text_end
            };

            self.set_cursor(pos);
        } else {
            let end_pos = if self.is_forward() {
                cursor.text_end
            } else {
                cursor.text_start
            };

            self.set_end(end_pos);
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

    fn on_kb_right(&mut self, input: &Input, text_len: usize) -> bool {
        let cursor_position = self.cursor_position();
        if cursor_position < text_len {
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
            self.set_cursor(text_len);
            return true;
        }

        false
    }

    fn on_kb_up(&mut self, layout: &Layout<Color>, input: &Input) {
        let cursor_position = self.cursor_position();
        let cursor = Cursor::from_position(layout, cursor_position, false);
        let line = cursor.path.line(layout).expect("Cannot get cursor line");
        let line_text_range = line.text_range();

        // if there is previous line
        let cursor_position = if line_text_range.start > 0 {
            let line_cursor_offset = cursor_position - line_text_range.start;
            let cursor = Cursor::from_position(layout, line_text_range.start - 1, false);
            let line = cursor.path.line(layout).expect("Cannot get cursor line");
            let line_text_range = line.text_range();
            let mut cursor_position = line_text_range.start + line_cursor_offset;

            if cursor_position >= line_text_range.end {
                cursor_position = line_text_range.end - 1;
            }

            cursor_position
        } else {
            0
        };

        if input.is_keyboard_shift_down() {
            self.set_end(cursor_position);
        } else {
            self.set_cursor(cursor_position);
        }
    }

    fn on_kb_down(&mut self, layout: &Layout<Color>, input: &Input, text_len: usize) {
        let cursor_position = self.cursor_position();
        let cursor = Cursor::from_position(layout, cursor_position, false);
        let line = cursor.path.line(layout).expect("Cannot get cursor line");
        let line_text_range = line.text_range();

        // if there is next line
        let cursor_position = if line_text_range.end + 1 < text_len {
            let line_cursor_offset = cursor_position - line_text_range.start;
            let cursor = Cursor::from_position(layout, line_text_range.end + 1, false);
            let line = cursor.path.line(layout).expect("Cannot get cursor line");
            let line_text_range = line.text_range();
            let mut cursor_position = line_text_range.start + line_cursor_offset;

            if cursor_position >= line_text_range.end {
                cursor_position = line_text_range.end - 1;
            }

            cursor_position
        } else {
            text_len
        };

        if input.is_keyboard_shift_down() {
            self.set_end(cursor_position);
        } else {
            self.set_cursor(cursor_position);
        }
    }

    fn on_kb_home(&mut self, layout: &Layout<Color>, input: &Input) {
        if input.is_keyboard_ctrl_down() {
            // till the beginning of the text
            if input.is_keyboard_shift_down() {
                self.set_end(0);
            } else {
                self.set_cursor(0);
            }
        } else {
            // till the beginning of the current line
            let cursor = Cursor::from_position(layout, self.cursor_position(), false);
            let line = cursor.path.line(layout).expect("Cannot get cursor line");
            let line_range = line.text_range();

            if input.is_keyboard_shift_down() {
                self.set_end(line_range.start);
            } else {
                self.set_cursor(line_range.start);
            }
        }
    }

    fn on_kb_end(&mut self, layout: &Layout<Color>, input: &Input, text_len: usize) {
        if input.is_keyboard_ctrl_down() {
            // till the end of the text
            if input.is_keyboard_shift_down() {
                self.set_end(text_len);
            } else {
                self.set_cursor(text_len);
            }
        } else {
            // till the end of the current line
            let cursor = Cursor::from_position(layout, self.cursor_position(), false);
            let line = cursor.path.line(layout).expect("Cannot get cursor line");
            let line_range = line.text_range();

            // decrease by 1 if it's not the last line to avoid moving cursor to the start of the next line
            let cursor_position = if line_range.end == text_len {
                line_range.end
            } else {
                line_range.end - 1
            };

            if input.is_keyboard_shift_down() {
                self.set_end(cursor_position);
            } else {
                self.set_cursor(cursor_position);
            }
        }
    }
}
