use crate::render::{Tex2D, Tex2D_Free};

use super::{TextContext, TextData};

pub struct TextView {
    data: TextData,
    width: f32,
    dirty: bool,
    tex: *mut Tex2D,
}

impl TextView {
    pub fn new(data: &TextData) -> Self {
        Self {
            data: data.clone(),
            width: 0.0,
            dirty: true,
            tex: std::ptr::null_mut(),
        }
    }

    pub fn set_data(&mut self, text_data: &TextData) {
        self.dirty = self.data.update(text_data);
    }

    pub fn update(
        &mut self,
        text_ctx: &mut TextContext,
        width: f32,
        scale_factor: f32,
    ) -> *mut Tex2D {
        if self.width != width {
            self.width = width;
            self.dirty = true;
        }

        // Regenerate texture only if something was changed
        if self.dirty {
            let tex = self.data.render(text_ctx, self.width, scale_factor);

            if self.tex != std::ptr::null_mut() {
                unsafe { Tex2D_Free(self.tex) };
            }

            self.tex = tex;
            self.dirty = false;
        }

        self.tex
    }
}

impl Drop for TextView {
    fn drop(&mut self) {
        if self.tex != std::ptr::null_mut() {
            unsafe { Tex2D_Free(self.tex) };
        }
    }
}
