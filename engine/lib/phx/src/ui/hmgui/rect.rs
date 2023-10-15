use glam::Vec4;

use crate::render::UIRenderer_Rect;

use super::widget::HmGuiWidget;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGuiRect {
    pub widget: HmGuiWidget,
    pub color: Vec4,
}

pub unsafe extern "C" fn HmGui_DrawRect(e: *mut HmGuiRect) {
    UIRenderer_Rect(
        (*e).widget.pos.x,
        (*e).widget.pos.y,
        (*e).widget.size.x,
        (*e).widget.size.y,
        (*e).color.x,
        (*e).color.y,
        (*e).color.z,
        (*e).color.w,
        false,
    );
}
