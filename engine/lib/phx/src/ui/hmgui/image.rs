use crate::render::{Tex2D, UIRenderer_Image};

use super::widget::HmGuiWidget;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGuiImage {
    pub widget: HmGuiWidget,
    pub image: *mut Tex2D,
}

pub unsafe extern "C" fn HmGui_DrawImage(e: *mut HmGuiImage) {
    UIRenderer_Image(
        (*e).image,
        (*e).widget.pos.x,
        (*e).widget.pos.y,
        (*e).widget.size.x,
        (*e).widget.size.y,
    );
}
