use std::{fs::File, path::Path};

use indexmap::IndexMap;
use serde_yaml::Value;
use walkdir::WalkDir;

use super::{
    parse_string, HmGuiPropertyId, HmGuiPropertyRegistry, HmGuiPropertyType, HmGuiStyle,
    HmGuiStyleId,
};

/// Contains a map of style name and style pairs.
/// Map is ordered by insertion.
#[derive(Default)]
pub struct HmGuiStyleRegistry {
    registry: IndexMap<String, HmGuiStyle>,
}

impl HmGuiStyleRegistry {
    /// Load styles from all files in the folder.
    pub fn load<F: FnMut(&str, &str) -> Option<(HmGuiPropertyId, HmGuiPropertyType)>>(
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
                    registry.insert(
                        theme_name.clone(),
                        HmGuiStyle::load(file_path, &theme_name, &mut f),
                    );
                }
            }
        }

        Self { registry }
    }

    /// Load list of styles from a single file.
    pub fn load_map<F: FnMut(&str, &str) -> Option<(HmGuiPropertyId, HmGuiPropertyType)>>(
        file_path: &Path,
        mut f: F,
    ) -> Self {
        let file = File::open(file_path).unwrap_or_else(|err| {
            panic!(
                "Cannot load style map file: {}. Error: {err}",
                file_path.display()
            )
        });
        let root_value: Value = serde_yaml::from_reader(&file).unwrap_or_else(|err| {
            panic!(
                "Cannot parse style map file: {}. Error: {err}",
                file_path.display()
            )
        });

        if root_value.is_null() {
            return Self {
                registry: Default::default(),
            };
        }

        let prop_table = root_value.as_mapping().unwrap_or_else(|| {
            panic!(
                "Cannot parse style map: {}. Expecting map type but was {root_value:?}",
                file_path.display()
            )
        });

        let mut registry = IndexMap::new();

        for (name_value, value) in prop_table.iter() {
            let style_name = parse_string(name_value)
                .unwrap_or_else(|err| panic!("{err}. File: {}", file_path.display()));
            let value = HmGuiStyle::parse_value(&style_name, value, &mut f)
                .unwrap_or_else(|err| panic!("{err}. File: {}", file_path.display()));

            registry.insert(style_name, value);
        }

        Self { registry }
    }

    /// Merge style properties into the property registry.
    pub fn merge_to(&self, property_registry: &mut HmGuiPropertyRegistry, style_name: &str) {
        let style = &self.registry[style_name];
        for (id, prop) in &style.properties {
            property_registry.set_property(id, prop);
        }
    }

    /// Get style by id.
    pub fn get(&self, id: HmGuiStyleId) -> Option<&HmGuiStyle> {
        self.registry.get_index(*id).map(|(_, s)| s)
    }

    /// Get style by name.
    pub fn get_by_name(&self, name: &str) -> Option<&HmGuiStyle> {
        self.registry.get(name)
    }

    /// Get style id by name.
    pub fn get_id(&self, name: &str) -> Option<HmGuiStyleId> {
        self.registry.get_index_of(name).map(|id| id.into())
    }

    /// Return number of registsred styles.
    pub fn size(&self) -> usize {
        self.registry.len()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use glam::*;
    use style::HmGuiStyle;

    use crate::ui::hmgui::style;
    use crate::{
        math::Box3,
        ui::hmgui::{HmGuiProperty, HmGuiPropertyType},
    };

    use super::HmGuiStyleRegistry;

    #[rustfmt::skip]
    const TEST_MAP1: &[(&str, &[(&str, HmGuiPropertyType, HmGuiProperty)])] = &[
        ("style1",&[
            ("prop.bool", HmGuiPropertyType::Bool, HmGuiProperty::Bool(true)),
            ("prop.i8", HmGuiPropertyType::I8, HmGuiProperty::I8(-10)),
            ("prop.u8", HmGuiPropertyType::U8, HmGuiProperty::U8(63)),
            ("prop.i16", HmGuiPropertyType::I16, HmGuiProperty::I16(-400)),
            ("prop.u16", HmGuiPropertyType::U16, HmGuiProperty::U16(1000)),
            ("prop.i32", HmGuiPropertyType::I32, HmGuiProperty::I32(-100000)),
            ("prop.u32", HmGuiPropertyType::U32, HmGuiProperty::U32(630000)),
            ("prop.i64", HmGuiPropertyType::I64, HmGuiProperty::I64(-10)),
            ("prop.u64", HmGuiPropertyType::U64, HmGuiProperty::U64(63)),
        ]),
        ("style2",&[
            ("prop.f32", HmGuiPropertyType::F32, HmGuiProperty::F32(-10.69)),
            ("prop.f64", HmGuiPropertyType::F64, HmGuiProperty::F64(63.132)),
        ]),
        ("style3",&[
            ("prop.vec2", HmGuiPropertyType::Vec2, HmGuiProperty::Vec2(Vec2::new(-10.2, 4.729))),
            ("prop.vec3", HmGuiPropertyType::Vec3, HmGuiProperty::Vec3(Vec3::new(-10.2, 4.729, 0.0))),
            ("prop.vec4", HmGuiPropertyType::Vec4, HmGuiProperty::Vec4(Vec4::new(-10.2, 4.729, 740.0, 44.6))),
            ("prop.ivec2", HmGuiPropertyType::IVec2, HmGuiProperty::IVec2(IVec2::new(-10, 4))),
            ("prop.ivec3", HmGuiPropertyType::IVec3, HmGuiProperty::IVec3(IVec3::new(-10, 4, 0))),
            ("prop.ivec4", HmGuiPropertyType::IVec4, HmGuiProperty::IVec4(IVec4::new(-10, 4, 740, 44))),
            ("prop.uvec2", HmGuiPropertyType::UVec2, HmGuiProperty::UVec2(UVec2::new(10, 4))),
            ("prop.uvec3", HmGuiPropertyType::UVec3, HmGuiProperty::UVec3(UVec3::new(10, 4, 0))),
            ("prop.uvec4", HmGuiPropertyType::UVec4, HmGuiProperty::UVec4(UVec4::new(10, 4, 740, 44))),
            ("prop.dvec2", HmGuiPropertyType::DVec2, HmGuiProperty::DVec2(DVec2::new(-10.2, 4.729))),
            ("prop.dvec3", HmGuiPropertyType::DVec3, HmGuiProperty::DVec3(DVec3::new(-10.2, 4.729, 0.0))),
            ("prop.dvec4", HmGuiPropertyType::DVec4, HmGuiProperty::DVec4(DVec4::new(-10.2, 4.729, 740.0, 44.6))),
        ]),
        ("style4",&[
            ("prop.box3", HmGuiPropertyType::Box3, HmGuiProperty::Box3(Box3::new(Vec3::new(10.2, 4.729, 1.0), Vec3::new(740.0, 44.6, -1.0)))),
        ]),
    ];

    fn test_style(style: &HmGuiStyle, expected: &[(&str, HmGuiPropertyType, HmGuiProperty)]) {
        assert_eq!(style.properties.len(), expected.len());

        for (id, (name, ty, expected)) in expected.iter().enumerate() {
            let actual = style.properties.get(&id.into()).expect(&format!(
                "Cannot find property. {id}/{name}/{ty:?}/{}",
                expected.name()
            ));

            if expected != actual {
                panic!(
                    "Mismatched property: {id}/{name}/{ty:?}/{} - {}",
                    expected.name(),
                    actual.name()
                );
            }
        }
    }

    #[test]
    #[ignore = "randomly failing"]
    fn test_hmgui_load_map() {
        let file_path = PathBuf::from("test_data/styles.yaml");
        let registry = HmGuiStyleRegistry::load_map(&file_path, |style_name, name| {
            if let Some((_, properties)) = TEST_MAP1.iter().find(|(name, _)| *name == style_name) {
                properties
                    .iter()
                    .enumerate()
                    .find(|(_, (n, _, _))| *n == name)
                    .map(|(id, (_, ty, _))| (id.into(), *ty))
            } else {
                None
            }
        });

        for (style_name, style) in registry.registry {
            let (_, properties) = TEST_MAP1
                .iter()
                .find(|(name, _)| *name == style_name)
                .expect("Cannot find a style");

            test_style(&style, properties);
        }
    }
}
