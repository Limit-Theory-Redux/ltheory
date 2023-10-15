use glam::Vec2;

use super::group::HmGuiGroup;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum WidgetType {
    Group,
    Text,
    Rect,
    Image,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum LayoutType {
    None,
    Stack,
    Vertical,
    Horizontal,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGuiWidget {
    pub parent: *mut HmGuiGroup,
    pub next: *mut HmGuiWidget,
    pub prev: *mut HmGuiWidget,
    pub hash: u64,
    pub ty: WidgetType,
    pub pos: Vec2,
    pub size: Vec2,
    pub minSize: Vec2,
    pub align: Vec2,
    pub stretch: Vec2,
}

pub unsafe extern "C" fn HmGui_LayoutWidget(e: *mut HmGuiWidget, pos: Vec2, sx: f32, sy: f32) {
    (*e).pos = pos;
    (*e).size = (*e).minSize;
    (*e).size.x += (*e).stretch.x * (sx - (*e).minSize.x);
    (*e).size.y += (*e).stretch.y * (sy - (*e).minSize.y);
    (*e).pos.x += (*e).align.x * (sx - (*e).size.x);
    (*e).pos.y += (*e).align.y * (sy - (*e).size.y);
}
