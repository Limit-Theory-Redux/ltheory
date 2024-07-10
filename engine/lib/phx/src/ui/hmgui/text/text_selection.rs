use std::ops::Range;

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

    /// Returns cursor position or end position of the selection.
    pub fn cursor_position(&self) -> usize {
        match self {
            Self::Cursor(pos) => *pos,
            Self::Selection(range) => range.end,
        }
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

    /// Create a new text selection where start is always >= end.
    pub fn normalized(&self) -> Self {
        match self {
            Self::Cursor(pos) => Self::Cursor(*pos),
            Self::Selection(range) => {
                if range.start < range.end {
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
        match self {
            Self::Cursor(start_pos) => {
                *self = Self::Cursor(*start_pos);
            }
            Self::Selection(range) => {
                if range.start == end_pos {
                    *self = Self::Cursor(range.start);
                } else {
                    *self = Self::Selection(Range {
                        start: range.start,
                        end: end_pos,
                    });
                }
            }
        }
    }

    /// Keeps cursor position or sets start position of the selection.
    /// Converts selection into cursor position if start == end.
    pub fn set_start(&mut self, start_pos: usize) {
        match self {
            Self::Cursor(_) => *self = Self::Cursor(start_pos),
            Self::Selection(range) => {
                if start_pos == range.end {
                    *self = Self::Cursor(start_pos);
                } else {
                    *self = Self::Selection(Range {
                        start: start_pos,
                        end: range.end,
                    });
                }
            }
        }
    }

    /// Converts cursor into selection by adding end position or
    /// update end position of the selection.
    /// Converts selection into cursor position if start == end.
    pub fn set_end(&mut self, end_pos: usize) {
        match self {
            Self::Cursor(pos) => {
                if *pos == end_pos {
                    *self = Self::Cursor(*pos);
                } else {
                    *self = Self::Selection(Range {
                        start: *pos,
                        end: end_pos,
                    });
                }
            }
            Self::Selection(range) => {
                if range.start == end_pos {
                    *self = Self::Cursor(end_pos);
                } else {
                    *self = Self::Selection(Range {
                        start: range.start,
                        end: end_pos,
                    });
                }
            }
        }
    }
}
