#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub enum RenderStyle {
    /// Basic rectangular area filled with color and additional alpha transparency
    #[default]
    None,
    /// Fill area depending on if mouse is over it
    Fill,
    /// Draw border of the area only
    Outline,
    /// Same as [`Self::Fill`] but without additional alpha transparency
    Underline,
}
