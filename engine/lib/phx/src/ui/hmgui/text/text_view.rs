use glam::Vec2;

use super::{TextContext, TextData};
use crate::input::Input;
use crate::render::Tex2D;

/// Contains text data and rendered text texture.
pub struct TextView {
    data: TextData,
    editable: bool,
    tex: Option<Tex2D>,
}

impl TextView {
    pub fn new(data: &TextData, editable: bool) -> Self {
        Self {
            data: data.clone(),
            editable,
            tex: None,
        }
    }

    pub fn data(&self) -> &TextData {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut TextData {
        &mut self.data
    }

    pub fn set_data(&mut self, text_data: &TextData) {
        self.data.update(text_data);
    }

    pub fn is_editable(&self) -> bool {
        self.editable
    }

    /// In case of text changes, updates user text data with the view one.
    /// Removes `text_changed` flag.
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

    /// Use user input to update text, selection and cursor position.
    /// Rerender text texture if any changes happened.
    pub fn update(
        &mut self,
        text_ctx: &mut TextContext,
        width: f32,
        scale_factor: f32,
        widget_pos: Vec2,
        input: &Input,
        focused: bool,
        clipboard: &mut String,
    ) -> Option<&Tex2D> {
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

        if tex.is_some() {
            self.tex = tex;
        }

        self.tex.as_ref()
    }
}
