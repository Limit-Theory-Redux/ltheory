use glam::*;
use indexmap::IndexMap;

use crate::{math::Box3, render::Font};

use super::{HmGuiProperty, HmGuiPropertyId};

#[luajit_ffi_gen::luajit_ffi(name = "GuiProperties")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HmGuiProperties {
    ContainerSpacingId,
    ContainerColorFrameId,
    ContainerColorPrimaryId,
    TextFontId,
    TextColorId,
    ButtonBorderWidthId,
}

impl HmGuiProperties {
    pub fn id(&self) -> usize {
        *self as _
    }
}

pub struct HmGuiPropertyRegistry {
    pub registry: IndexMap<String, HmGuiProperty>,
}

macro_rules! decl_prop_method {
    ($m:ident, $v:ident, $ty:ident) => {
        pub fn $m(&self, id: HmGuiPropertyId) -> $ty {
            let prop = &self.registry[*id];
            let HmGuiProperty::$v(value) = prop else {
                panic!("Expected {} but was {:?}", stringify!($v), prop.name())
            };

            *value
        }
    };
}

macro_rules! decl_prop_ref_method {
    ($m:ident, $v:ident, $ty:ident) => {
        pub fn $m(&self, id: HmGuiPropertyId) -> &$ty {
            let prop = &self.registry[*id];
            let HmGuiProperty::$v(value) = prop else {
                panic!("Expected {} but was {:?}", stringify!($v), prop.name())
            };

            value
        }
    };
}

impl HmGuiPropertyRegistry {
    #[rustfmt::skip]
    pub fn new() -> Self {
        let mut r = Default::default();

        reg(&mut r, "container.spacing", 6.0f32, HmGuiProperties::ContainerSpacingId);
        reg(&mut r, "container.color-frame", Vec4::new(0.1, 0.1, 0.1, 0.5), HmGuiProperties::ContainerColorFrameId);
        reg(&mut r, "container.color-primary", Vec4::new(0.1, 0.5, 1.0, 1.0), HmGuiProperties::ContainerColorPrimaryId);
        reg(&mut r, "text.font", Font::load("Rajdhani", 14), HmGuiProperties::TextFontId);
        reg(&mut r, "text.color", Vec4::ONE, HmGuiProperties::TextColorId);
        reg(&mut r, "button.border-width", 0.0f32, HmGuiProperties::ButtonBorderWidthId);

        Self { registry: r }
    }

    pub fn get_id(&self, name: &str) -> HmGuiPropertyId {
        self.registry
            .get_index_of(name)
            .map(|id| id.into())
            .unwrap_or_else(|| panic!("Property {name:?} was not registered"))
    }

    pub fn register(&mut self, name: &str, value: HmGuiProperty) -> HmGuiPropertyId {
        assert!(
            !self.registry.contains_key(name),
            "Property {name:?} was already registered"
        );

        let id = self.registry.len();

        self.registry.insert(name.into(), value);

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

#[inline]
fn reg<T: Into<HmGuiProperty>>(
    r: &mut IndexMap<String, HmGuiProperty>,
    name: &str,
    value: T,
    expected_id: HmGuiProperties,
) {
    assert!(r.get(name).is_none(), "Property {name:?} already exists");

    let id = r.len();
    assert_eq!(id, expected_id as _, "Wrong property id");

    r.insert(name.into(), value.into());
}
