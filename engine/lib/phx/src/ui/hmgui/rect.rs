use glam::{Vec2, Vec4};

use crate::render::UIRenderer;

#[derive(Clone, PartialEq)]
pub struct HmGuiRect {
    pub color: Vec4,
}

impl HmGuiRect {
    pub fn draw(&self, renderer: &mut UIRenderer, pos: Vec2, size: Vec2) {
        renderer.rect(pos, size, self.color, false);
    }

    // For testing.
    #[allow(dead_code)]
    pub(crate) fn dump(&self, ident: usize) {
        let ident_str = format!("{}", crate::ui::hmgui::IDENT.repeat(ident));

        println!("{ident_str}- color: {:?}", self.color);
    }
}
