use glam::Vec2;

use crate::render::{Color, Font, TextOutline, UIRenderer};

#[derive(Clone)]
pub struct HmGuiText {
    pub font: Font,
    pub text: String,
    pub color: Color,
    /// Drawn with an outline and halo (`Gui:textOutlined`).
    pub outline: Option<TextOutline>,
}

impl PartialEq for HmGuiText {
    fn eq(&self, other: &Self) -> bool {
        self.font.name() == other.font.name()
            && self.text == other.text
            && self.color == other.color
            && self.outline == other.outline
    }
}

impl HmGuiText {
    pub fn draw(&self, renderer: &mut UIRenderer, pos: Vec2) {
        renderer.text(&self.font, &self.text, pos, self.color, self.outline);
    }

    // For testing.
    #[allow(dead_code)]
    pub(crate) fn dump(&self, ident: usize) {
        let ident_str = crate::ui::hmgui::IDENT.repeat(ident).to_string();

        println!("{ident_str}- text:  {}", self.text);
        println!("{ident_str}- color: {:?}", self.color);
        if let Some(outline) = &self.outline {
            println!("{ident_str}- outline: {outline:?}");
        }
        println!("{ident_str}- font:  {:?}", self.font.name());
    }
}
