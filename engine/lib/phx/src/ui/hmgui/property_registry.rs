use glam::*;
use indexmap::IndexMap;

use crate::{math::Box3, render::Font};

use super::{register_core_properties, HmGuiProperty, HmGuiPropertyId, HmGuiPropertyValue};

/// Contains a map of property name and info pairs.
/// Map is ordered by insertion.
#[derive(Clone)]
pub struct HmGuiPropertyRegistry {
    pub registry: IndexMap<String, HmGuiProperty>,
}

macro_rules! decl_prop_method {
    ($m:ident, $v:ident, $ty:ident) => {
        pub fn $m(&self, id: HmGuiPropertyId) -> $ty {
            let prop = &self.registry[*id].value;
            let HmGuiPropertyValue::$v(value) = prop else {
                panic!("Expected {} but was {:?}", stringify!($v), prop.name())
            };

            *value
        }
    };
}

macro_rules! decl_prop_ref_method {
    ($m:ident, $v:ident, $ty:ident) => {
        pub fn $m(&self, id: HmGuiPropertyId) -> &$ty {
            let prop = &self.registry[*id].value;
            let HmGuiPropertyValue::$v(value) = prop else {
                panic!("Expected {} but was {:?}", stringify!($v), prop.name())
            };

            value
        }
    };
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

    decl_prop_method!(get_bool, Bool, bool);
    decl_prop_method!(get_i8, I8, i8);
    decl_prop_method!(get_u8, U8, u8);
    decl_prop_method!(get_i16, I16, i16);
    decl_prop_method!(get_u16, U16, u16);
    decl_prop_method!(get_i32, I32, i32);
    decl_prop_method!(get_u32, U32, u32);
    decl_prop_method!(get_i64, I64, i64);
    decl_prop_method!(get_u64, U64, u64);
    decl_prop_method!(get_f32, F32, f32);
    decl_prop_method!(get_f64, F64, f64);
    decl_prop_method!(get_vec2, Vec2, Vec2);
    decl_prop_method!(get_vec3, Vec3, Vec3);
    decl_prop_method!(get_vec4, Vec4, Vec4);
    decl_prop_method!(get_ivec2, IVec2, IVec2);
    decl_prop_method!(get_ivec3, IVec3, IVec3);
    decl_prop_method!(get_ivec4, IVec4, IVec4);
    decl_prop_method!(get_uvec2, UVec2, UVec2);
    decl_prop_method!(get_uvec3, UVec3, UVec3);
    decl_prop_method!(get_uvec4, UVec4, UVec4);
    decl_prop_method!(get_dvec2, DVec2, DVec2);
    decl_prop_method!(get_dvec3, DVec3, DVec3);
    decl_prop_method!(get_dvec4, DVec4, DVec4);
    decl_prop_method!(get_box3, Box3, Box3);
    decl_prop_ref_method!(get_string, String, String);
    decl_prop_ref_method!(get_font, Font, Font);
}
