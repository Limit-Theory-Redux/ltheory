#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FocusStyle {
    None,
    Fill,
    Outline,
    Underline,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FocusType {
    Mouse,
    Scroll,
}
