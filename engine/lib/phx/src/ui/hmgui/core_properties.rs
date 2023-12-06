use glam::*;
use indexmap::IndexMap;

use crate::render::Font;

use super::{HmGuiProperty, HmGuiPropertyId, HmGuiPropertyInfo};

macro_rules! core_properties {
    ($(($v:ident, $n:literal, $d:expr $(, $m:ident)*),)*) => {
        #[luajit_ffi_gen::luajit_ffi(name = "GuiProperties")]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum HmGuiProperties {
            $($v),*
        }

        impl HmGuiProperties {
            pub fn id(&self) -> usize {
                *self as _
            }
        }

        pub fn register_core_properties() -> IndexMap<String, HmGuiPropertyInfo> {
            let mut r = Default::default();

            $(reg(&mut r, $n, $d, HmGuiProperties::$v, &[$((HmGuiProperties::$m as usize).into(),)*]);)*

            r
        }
    };
}

// Property id, name, value and optional list of mapped property ids
core_properties! {
    (ContainerSpacingId,      "container.spacing",       6.0f32),
    (ContainerColorFrameId,   "container.color-frame",   Vec4::new(0.1, 0.1, 0.1, 0.5)),
    (ContainerColorPrimaryId, "container.color-primary", Vec4::new(0.1, 0.5, 1.0, 1.0)),
    (TextFontId,              "text.font",               Font::load("Rajdhani", 14)),
    (TextColorId,             "text.color",              Vec4::ONE),
    (ButtonBorderWidthId,     "button.border-width",     0.0f32),
    (ButtonTextColorId,       "button.text-color",       Vec4::ONE, TextColorId),
}

/// Adds a new property to the map.
/// Verifies its expected id and mapped ids.
#[inline]
fn reg<T: Into<HmGuiProperty>>(
    r: &mut IndexMap<String, HmGuiPropertyInfo>,
    name: &str,
    value: T,
    expected_id: HmGuiProperties,
    map_ids: &[HmGuiPropertyId],
) {
    assert!(r.get(name).is_none(), "Property {name:?} already exists");

    let id = r.len();
    assert_eq!(id, expected_id as _, "Wrong property id");

    let property: HmGuiProperty = value.into();

    for map_id in map_ids {
        assert_ne!(**map_id, id, "Property {name:?} maps to itself"); // TODO: check for the circular dependency

        let (map_name, property_info) = r.get_index(**map_id).expect("Unknown pam property");
        assert_eq!(
            property.get_type(),
            property_info.property.get_type(),
            "Wrong {map_name:?} map property type"
        );
    }

    r.insert(
        name.into(),
        HmGuiPropertyInfo {
            property,
            map_ids: map_ids.into(),
        },
    );
}
