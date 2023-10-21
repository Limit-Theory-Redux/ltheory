use glam::{Vec2, Vec4};

use crate::render::UIRenderer_Rect;

use super::{HmGui, HmGuiWidget, Rf};

#[derive(Clone, PartialEq)]
pub struct HmGuiRect {
    pub color: Vec4,
}

impl HmGuiRect {
    pub fn draw(&self, pos: Vec2, size: Vec2) {
        unsafe {
            UIRenderer_Rect(
                pos.x,
                pos.y,
                size.x,
                size.y,
                self.color.x,
                self.color.y,
                self.color.z,
                self.color.w,
                false,
            );
        }
    }

    // For testing.
    #[allow(dead_code)]
    pub(crate) fn dump(&self, ident: usize) {
        let ident_str = format!("{}", crate::ui::hmgui::IDENT.repeat(ident));

        println!("{ident_str}- color: {:?}", self.color);
    }
}
