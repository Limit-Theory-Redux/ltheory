use glam::Vec4;
use internal::*;

use crate::render::{Font, UIRenderer_Text};

use super::widget::HmGuiWidget;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGuiText {
    pub widget: HmGuiWidget,
    pub font: *mut Font,
    pub text: *const libc::c_char,
    pub color: Vec4,
}

pub unsafe extern "C" fn HmGui_FreeText(e: *mut HmGuiText) {
    StrFree((*e).text);
    MemFree(e as *const _);
}

pub unsafe extern "C" fn HmGui_DrawText(e: *mut HmGuiText) {
    // #if HMGUI_DRAW_GROUP_FRAMES
    //   Draw_Color(0.5f, 0.2f, 0.2f, 0.5f);
    //   Draw_Border(1.0f, e->pos.x, e->pos.y, e->size.x, e->size.y);
    //#endif

    UIRenderer_Text(
        (*e).font,
        (*e).text,
        (*e).widget.pos.x,
        (*e).widget.pos.y + (*e).widget.minSize.y,
        (*e).color.x,
        (*e).color.y,
        (*e).color.z,
        (*e).color.w,
    );
}
