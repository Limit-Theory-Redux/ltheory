use glam::Vec4;

use crate::render::{Font, UIRenderer_Text};

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
    pub fn draw(&self, pos_x: f32, pos_y: f32) {
        // #if HMGUI_DRAW_GROUP_FRAMES
        //   Draw_Color(0.5f, 0.2f, 0.2f, 0.5f);
        //   Draw_Border(1.0f, e->pos.x, e->pos.y, e->size.x, e->size.y);
        //#endif

        unsafe {
            UIRenderer_Text(
                &self.font,
                &self.text,
                pos_x,
                pos_y,
                self.color.x,
                self.color.y,
                self.color.z,
                self.color.w,
            );
        }
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
