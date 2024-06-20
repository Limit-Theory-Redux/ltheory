use glam::Vec2;

use crate::input::Input;
use crate::render::{Tex2D, Tex2D_Free};

use super::{TextContext, TextData};

pub struct TextView {
    data: TextData,
    editable: bool,
    width: f32,
    dirty: bool,
    tex: *mut Tex2D,
}

impl TextView {
    pub fn new(data: &TextData, editable: bool) -> Self {
        Self {
            data: data.clone(),
            editable,
            width: 0.0,
            dirty: true,
            tex: std::ptr::null_mut(),
        }
    }

    pub fn cursor_rect_pos(&self) -> Vec2 {
        self.data.cursor_rect_pos()
    }

    pub fn cursor_rect_size(&self) -> Vec2 {
        self.data.cursor_rect_size()
    }

    pub fn set_data(&mut self, text_data: &TextData) {
        // TODO: process input in text data

        self.dirty = self.data.update(text_data);
    }

    pub fn update_source(&mut self, text_data: &mut TextData) {
        debug_assert!(self.editable);

        // TODO: process input in text data

        if self.data.is_text_changed() {
            text_data.set_text(self.data.text());
            // TODO: update other text data fields

            self.data.unset_text_changed();
        }
    }

    pub fn update(
        &mut self,
        text_ctx: &mut TextContext,
        width: f32,
        scale_factor: f32,
        widget_pos: Vec2,
        input: Option<&Input>,
        focused: bool,
    ) -> *mut Tex2D {
        if self.width != width {
            self.width = width;
            self.dirty = true;
        }

        // Regenerate texture only if something was changed or text is editable
        if self.dirty || self.data.is_text_changed() || self.editable {
            let tex = self.data.render(
                text_ctx,
                self.width,
                scale_factor,
                widget_pos,
                input,
                self.editable,
                focused,
            );

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
