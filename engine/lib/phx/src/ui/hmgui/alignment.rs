/// Horizontal alignment of the container elements.
#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AlignHorizontal {
    /// Default alignment (Left).
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

impl AlignHorizontal {
    pub fn is_default(&self) -> bool {
        *self == Self::Default
    }

    pub fn is_center(&self) -> bool {
        *self == Self::Center
    }

    pub fn is_left(&self) -> bool {
        *self == Self::Left || *self == Self::Default
    }

    pub fn is_right(&self) -> bool {
        *self == Self::Right
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

impl AlignVertical {
    pub fn is_default(&self) -> bool {
        *self == Self::Default
    }

    pub fn is_center(&self) -> bool {
        *self == Self::Center
    }

    pub fn is_top(&self) -> bool {
        *self == Self::Top || *self == Self::Default
    }

    pub fn is_bottom(&self) -> bool {
        *self == Self::Bottom
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
