use glam::{Vec2, Vec4};

use crate::render::{Font, UIRenderer};

#[derive(Clone)]
pub struct HmGuiText {
    pub font: Font,
    pub text: String,
    pub color: Vec4,
}

impl PartialEq for HmGuiText {
    fn eq(&self, other: &Self) -> bool {
        self.font.name() == other.font.name()
            && self.text == other.text
            && self.color == other.color
    }
}

impl HmGuiText {
    pub fn draw(&self, renderer: &UIRenderer, pos: Vec2) {
        // #if HMGUI_DRAW_GROUP_FRAMES
        //   Draw_Color(0.5f, 0.2f, 0.2f, 0.5f);
        //   Draw_Border(1.0f, e->pos.x, e->pos.y, e->size.x, e->size.y);
        //#endif

        renderer.text(&self.font, &self.text, pos, self.color);
    }

    // For testing.
    #[allow(dead_code)]
    pub(crate) fn dump(&self, ident: usize) {
        let ident_str = format!("{}", crate::ui::hmgui::IDENT.repeat(ident));

        println!("{ident_str}- text:  {}", self.text);
        println!("{ident_str}- color: {:?}", self.color);
        println!("{ident_str}- font:  {:?}", self.font.name());
    }
}
