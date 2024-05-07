use indexmap::IndexMap;

use super::{register_core_properties, HmGuiProperty, HmGuiPropertyId, HmGuiPropertyValue};

/// Contains a map of property name and info pairs.
/// Map is ordered by insertion.
#[derive(Clone)]
pub struct HmGuiPropertyRegistry {
    pub registry: IndexMap<String, HmGuiProperty>,
}

impl HmGuiPropertyRegistry {
    /// Create a registry initialized with a core properties.
    pub fn new() -> Self {
        Self {
            registry: register_core_properties(),
        }
    }

    /// Get property id by name.
    pub fn get_id(&self, name: &str) -> HmGuiPropertyId {
        self.registry
            .get_index_of(name)
            .map(|id| id.into())
            .unwrap_or_else(|| panic!("Property {name:?} was not registered"))
    }

    /// Set value of the existing property.
    pub fn set_property(&mut self, id: &HmGuiPropertyId, prop: &HmGuiPropertyValue) {
        assert!(**id < self.registry.len(), "Unknown property id {}", **id);

        assert_eq!(
            self.registry[**id].value.get_type(),
            prop.get_type(),
            "Wrong property type"
        );

        self.registry[**id].value = prop.clone();
    }

    /// Register a new property and return its id.
    pub fn register(
        &mut self,
        name: &str,
        value: HmGuiPropertyValue,
        map_ids: &[HmGuiPropertyId],
    ) -> HmGuiPropertyId {
        assert!(
            !self.registry.contains_key(name),
            "Property {name:?} was already registered"
        );

        let id = self.registry.len();

        self.registry.insert(
            name.into(),
            HmGuiProperty {
                value,
                map_ids: map_ids.into(),
            },
        );

        id.into()
    }
}
