use glam::Vec2;

use crate::input::Input;
use crate::render::{Tex2D, Tex2D_Free};

use super::{TextContext, TextData};

pub struct TextView {
    data: TextData,
    editable: bool,
    tex: *mut Tex2D,
}

impl TextView {
    pub fn new(data: &TextData, editable: bool) -> Self {
        Self {
            data: data.clone(),
            editable,
            tex: std::ptr::null_mut(),
        }
    }

    pub fn data(&self) -> &TextData {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut TextData {
        &mut self.data
    }

    pub fn set_data(&mut self, text_data: &TextData) {
        // TODO: process input in text data

        self.data.update(text_data);
    }

    pub fn is_editable(&self) -> bool {
        self.editable
    }

    pub fn update_source(&mut self, text_data: &mut TextData) -> bool {
        debug_assert!(self.editable);

        if self.data.is_text_changed() {
            text_data.set_text(self.data.text());
            // TODO: update other text data fields

            self.data.unset_text_changed();

            true
        } else {
            false
        }
    }

    pub fn update(
        &mut self,
        text_ctx: &mut TextContext,
        width: f32,
        scale_factor: f32,
        widget_pos: Vec2,
        input: &Input,
        focused: bool,
        clipboard: &mut String,
    ) -> *mut Tex2D {
        let tex = self.data.render(
            text_ctx,
            width,
            scale_factor,
            widget_pos,
            input,
            self.editable,
            focused,
            clipboard,
        );

        if tex != std::ptr::null_mut() {
            if self.tex != std::ptr::null_mut() {
                unsafe { Tex2D_Free(self.tex) };
            }

            self.tex = tex;
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
