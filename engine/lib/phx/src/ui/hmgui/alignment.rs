#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AlignHorizontal {
    /// Default alignment (Left)
    #[default]
    Default,
    Center,
    Left,
    Right,
    Stretch,
}

#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AlignVertical {
    /// Default alignment (Top)
    #[default]
    Default,
    Center,
    Top,
    Bottom,
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
}
