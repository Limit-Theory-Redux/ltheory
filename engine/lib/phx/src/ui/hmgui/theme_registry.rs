use indexmap::IndexMap;

use super::{HmGuiStyle, HmGuiStyleId};

#[derive(Default)]
pub struct HmGuiThemeRegistry {
    pub registry: IndexMap<String, HmGuiStyle>,
    pub active_theme: Option<HmGuiStyleId>,
}
