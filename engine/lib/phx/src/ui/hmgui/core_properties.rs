use indexmap::IndexMap;

use crate::render::{Color, Font};

use super::{HmGuiProperty, HmGuiPropertyId, HmGuiPropertyInfo};

macro_rules! core_properties {
    ($(($v:ident, $n:literal, $d:expr $(, $m:ident)*),)*) => {
        #[luajit_ffi_gen::luajit_ffi(name = "GuiProperties")]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum HmGuiProperties {
            $($v),*
        }

        // NOTE: it's not possible to implement Deref because of recursive call
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
    (OpacityId,               "opacity",                 1.0f32),
    (BackgroundColorId,       "background-color",        Color::new(0.1, 0.12, 0.13, 1.0)),
    (HighlightColorId,        "highlight-color",         Color::new(0.1, 0.5, 1.0, 1.0)),

    (TextFontId,              "text.font",               Font::load("Rajdhani", 14)),
    (TextColorId,             "text.color",              Color::WHITE),

    (ContainerClipId,         "container.clip",          true),
    (ContainerSpacingId,      "container.spacing",       6.0f32),
    (ContainerColorFrameId,   "container.color-frame",   Color::new(0.1, 0.1, 0.1, 0.5)),
    (ContainerColorPrimaryId, "container.color-primary", Color::new(0.1, 0.5, 1.0, 1.0)),

    (ButtonBorderWidthId,     "button.border-width",     0.0f32),
    (ButtonTextColorId,       "button.text-color",       Color::WHITE, TextColorId),
    (ButtonOpacityId,         "button.opacity",          0.5f32, OpacityId),
    (ButtonBackgroundColorId, "button.background-color", Color::new(0.15, 0.15, 0.15, 0.8), BackgroundColorId),
    (ButtonHighlightColorId,  "button.highlight-color",  Color::new(0.1, 0.5, 1.0, 1.0), HighlightColorId),

    (CheckboxBackgroundColorId, "checkbox.background-color", Color::new(0.3, 0.3, 0.3, 0.5), BackgroundColorId),
    (CheckboxHighlightColorId,  "checkbox.highlight-color",  Color::new(0.3, 0.3, 0.3, 1.0), HighlightColorId),

    (ScrollAreaHScrollShowId, "scroll-area.hscroll.show", true),
    (ScrollAreaVScrollShowId, "scroll-area.vscroll.show", true),
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
