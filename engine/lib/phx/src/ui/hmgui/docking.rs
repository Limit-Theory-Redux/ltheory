#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Docking {
    None = 0,
    Left = 1,
    Right = 2,
    Top = 4,
    Bottom = 8,
    StretchHorizontal = 3,
    StretchVertical = 12,
    StretchAll = 15,
}

pub const DOCKING_NONE: u8 = Docking::None.value();
pub const DOCKING_LEFT: u8 = Docking::Left.value();
pub const DOCKING_RIGHT: u8 = Docking::Right.value();
pub const DOCKING_TOP: u8 = Docking::Top.value();
pub const DOCKING_BOTTOM: u8 = Docking::Bottom.value();
pub const DOCKING_STRETCH_HORIZONTAL: u8 = Docking::StretchHorizontal.value();
pub const DOCKING_STRETCH_VERTICAL: u8 = Docking::StretchVertical.value();
pub const DOCKING_STRETCH_ALL: u8 = Docking::StretchAll.value();

const DOCKING_FLAGS_MASK: u8 = DOCKING_LEFT | DOCKING_RIGHT | DOCKING_TOP | DOCKING_BOTTOM;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct DockingType(u8);

impl DockingType {
    pub fn has_docking(&self) -> bool {
        self.0 & DOCKING_FLAGS_MASK != 0
    }

    pub fn has_horizontal_stretch(&self) -> bool {
        self.is_dock_left() && self.is_dock_right()
    }

    pub fn no_horizontal_stretch(&self) -> bool {
        !self.is_dock_left() && !self.is_dock_right()
    }

    pub fn has_vertical_stretch(&self) -> bool {
        self.is_dock_top() && self.is_dock_bottom()
    }

    pub fn no_vertical_stretch(&self) -> bool {
        !self.is_dock_top() && !self.is_dock_bottom()
    }

    pub fn is_dock_left(&self) -> bool {
        self.0 & Docking::Left.value() != 0
    }

    pub fn is_dock_right(&self) -> bool {
        self.0 & Docking::Right.value() != 0
    }

    pub fn is_dock_top(&self) -> bool {
        self.0 & Docking::Top.value() != 0
    }

    pub fn is_dock_bottom(&self) -> bool {
        self.0 & Docking::Bottom.value() != 0
    }
}

impl std::fmt::Debug for DockingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dockings = vec![];
        if self.is_dock_left() {
            dockings.push("Left");
        }
        if self.is_dock_right() {
            dockings.push("Right");
        }
        if self.is_dock_top() {
            dockings.push("Top");
        }
        if self.is_dock_bottom() {
            dockings.push("Bottom");
        }
        if dockings.is_empty() {
            dockings.push("None");
        }

        f.debug_tuple(&format!("{}", dockings.join("|"))).finish()
    }
}

impl From<u8> for DockingType {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
