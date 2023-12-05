use std::path::Path;

use indexmap::IndexMap;
use walkdir::WalkDir;

use super::{HmGuiPropertyId, HmGuiPropertyRegistry, HmGuiPropertyType, HmGuiStyle, HmGuiStyleId};

#[derive(Default)]
pub struct HmGuiStyleRegistry {
    registry: IndexMap<String, HmGuiStyle>,
}

impl HmGuiStyleRegistry {
    pub fn load<F: FnMut(&str) -> Option<(HmGuiPropertyId, HmGuiPropertyType)>>(
        folder_path: &Path,
        mut f: F,
    ) -> Self {
        let dir = WalkDir::new(folder_path);
        let mut registry = IndexMap::new();

        for entry in dir.into_iter().filter_map(|e| e.ok()) {
            let file_path = &entry.path();
            if file_path.is_file() {
                if let Some(theme_name) = file_path
                    .file_stem()
                    .map(|stem| stem.to_str().map(|stem| stem.to_string()))
                    .flatten()
                {
                    registry.insert(theme_name, HmGuiStyle::load(file_path, &mut f));
                }
            }
        }

        Self { registry }
    }

    pub fn merge_to(&self, property_registry: &mut HmGuiPropertyRegistry, style_name: &str) {
        let style = &self.registry[style_name];
        for (id, prop) in &style.properties {
            property_registry.set_property(id, prop);
        }
    }

    pub fn get(&self, id: HmGuiStyleId) -> Option<&HmGuiStyle> {
        self.registry.get_index(*id).map(|(_, s)| s)
    }

    pub fn get_by_name(&self, name: &str) -> Option<&HmGuiStyle> {
        self.registry.get(name)
    }

    pub fn get_id(&self, name: &str) -> Option<HmGuiStyleId> {
        self.registry.get_index_of(name).map(|id| id.into())
    }

    pub fn size(&self) -> usize {
        self.registry.len()
    }
}
