/// Horizontal alignment of the container elements.
#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AlignHorizontal {
    /// Default alignment (Left).
    /// If element has default alignment then container's children alignment will be taken in account in layouting.
    #[default]
    Default,
    /// Center element inside parent container.
    Center,
    /// Align element to the left inside parent container.
    Left,
    /// Align element to the right inside parent container.
    Right,
    /// Expand element horizontally inside parent container.
    /// Container with expand alignment will always fit its parent width.
    /// This is in contrast to stretch alignment in which case width can be bigger than the parent one.
    Expand,
    /// Stretch element horizontally inside parent container.
    /// Container with stretch alignment will grow in size to stick to the parent sides or to envelop it's children if they are bigger.
    Stretch,
}

/// Vertical alignment of the container elements.
#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AlignVertical {
    /// Default alignment (Top).
    /// If element has default alignment then container's children alignment will be taken in account in layouting.
    #[default]
    Default,
    /// Center element inside parent container.
    Center,
    /// Align element to the top inside parent container.
    Top,
    /// Align element to the bottom inside parent container.
    Bottom,
    /// Expand element vertically inside parent container.
    /// Container with expand alignment will always fit its parent height.
    /// This is in contrast to stretch alignment in which case height can be bigger than the parent one.
    Expand,
    /// Stretch element vertically inside parent container.
    /// Container with stretch alignment will grow in size to stick to the parent sides or to envelop it's children if they are bigger.
    Stretch,
}

/// Generic container elements alignment
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    #[default]
    Default,
    /// Center element inside parent container.
    Center,
    /// Align element to the left/top inside parent container.
    Begin,
    /// Align element to the right/bottom inside parent container.
    End,
    /// Expand element inside parent container.
    /// Container with expand alignment will always fit its parent size.
    /// This is in contrast to stretch alignment in which case size can be bigger than the parent one.
    Expand,
    /// Stretch element inside parent container.
    /// Container with stretch alignment will grow in size to stick to the parent sides or to envelop it's children if they are bigger.
    Stretch,
}

impl Alignment {
    pub fn is_default(&self) -> bool {
        *self == Self::Default
    }

    pub fn is_center(&self) -> bool {
        *self == Self::Center
    }

    pub fn is_begin(&self) -> bool {
        *self == Self::Begin || *self == Self::Default
    }

    pub fn is_end(&self) -> bool {
        *self == Self::End
    }

    pub fn is_stretch(&self) -> bool {
        *self == Self::Stretch
    }

    pub fn is_expand(&self) -> bool {
        *self == Self::Expand
    }

    pub fn is_extend(&self) -> bool {
        *self == Self::Stretch || *self == Self::Expand
    }
}

impl From<AlignHorizontal> for Alignment {
    fn from(value: AlignHorizontal) -> Self {
        match value {
            AlignHorizontal::Default => Self::Default,
            AlignHorizontal::Center => Self::Center,
            AlignHorizontal::Left => Self::Begin,
            AlignHorizontal::Right => Self::End,
            AlignHorizontal::Expand => Self::Expand,
            AlignHorizontal::Stretch => Self::Stretch,
        }
    }
}

impl From<AlignVertical> for Alignment {
    fn from(value: AlignVertical) -> Self {
        match value {
            AlignVertical::Default => Self::Default,
            AlignVertical::Center => Self::Center,
            AlignVertical::Top => Self::Begin,
            AlignVertical::Bottom => Self::End,
            AlignVertical::Expand => Self::Expand,
            AlignVertical::Stretch => Self::Stretch,
        }
    }
}
