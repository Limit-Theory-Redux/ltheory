use std::path::Path;

use indexmap::IndexMap;

use super::{HmGuiPropertyId, HmGuiPropertyType, HmGuiStyle, HmGuiStyleId, HmGuiStyleRegistry};

#[derive(Default)]
pub struct HmGuiThemeRegistry {
    registry: HmGuiStyleRegistry,
    active_theme: Option<HmGuiStyleId>,
}

impl HmGuiThemeRegistry {
    pub fn load<F: FnMut(&str) -> Option<(HmGuiPropertyId, HmGuiPropertyType)>>(
        folder_path: &Path,
        f: F,
    ) -> Self {
        Self {
            registry: HmGuiStyleRegistry::load(folder_path, f),
            active_theme: None,
        }
    }

    pub fn active_theme(&self) -> Option<&HmGuiStyle> {
        self.active_theme.map(|id| self.registry.get(id)).flatten()
    }

    pub fn set_active_theme(&mut self, name: &str) {
        let id = self.registry.get_id(name);
        assert!(id.is_some(), "Unknown theme: {name}");

        self.active_theme = id;
    }

    pub fn clear_active_theme(&mut self) {
        self.active_theme = None;
    }

    pub fn size(&self) -> usize {
        self.registry.size()
    }
}
