use indexmap::IndexMap;

use crate::render::{Color, Font};

use super::{HmGuiProperty, HmGuiPropertyId, HmGuiPropertyInfo};

macro_rules! core_properties {
    ($(
        $(#[doc = $doc:literal])*
        ($v:ident, $n:literal, $d:expr $(, $m:ident)*),
    )*) => {
        #[luajit_ffi_gen::luajit_ffi(name = "GuiProperties")]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum HmGuiProperties {
            $(
                #[doc = "Config name: "]
                #[doc = $n]
                $(#[doc = $doc])*
                $v
            ),*
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
    /// Type: f32. Default value: 1
    (Opacity,               "opacity",                 1.0f32),
    /// Type: Color. Default value: Color(0.1, 0.12, 0.13, 1.0)
    (BackgroundColor,       "background-color",        Color::new(0.1, 0.12, 0.13, 1.0)),
    /// Type: Color. Default value: Color(0.1, 0.5, 1.0, 1.0)
    (HighlightColor,        "highlight-color",         Color::new(0.1, 0.5, 1.0, 1.0)),

    /// Type: Font. Default value: Font("Rajdhani", 14)
    (TextFont,              "text.font",               Font::load("Rajdhani", 14)),
    /// Type: Color. Default value: White
    (TextColor,             "text.color",              Color::WHITE),

    /// Type: boolean. Default value: true
    (ContainerClip,         "container.clip",          true),
    /// Type: f32. Default value: 6
    (ContainerSpacing,      "container.spacing",       6.0f32),
    /// Type: Color. Default value: Color(0.1, 0.1, 0.1, 0.5)
    (ContainerColorFrame,   "container.color-frame",   Color::new(0.1, 0.1, 0.1, 0.5)),
    /// Type: Color. Default value: Color(0.1, 0.5, 1.0, 1.0)
    (ContainerColorPrimary, "container.color-primary", Color::new(0.1, 0.5, 1.0, 1.0)),

    /// Type: f32. Default value: 0
    (ButtonBorderWidth,     "button.border-width",     0.0f32),
    /// Type: Color. Default value: Wite. Maps to: TextColor
    (ButtonTextColor,       "button.text-color",       Color::WHITE, TextColor),
    /// Type: f32. Default value: 0.5
    (ButtonOpacity,         "button.opacity",          0.5f32, Opacity),
    /// Type: Color. Default value: Color(0.15, 0.15, 0.15, 0.8). Maps to: BackgroundColor
    (ButtonBackgroundColor, "button.background-color", Color::new(0.15, 0.15, 0.15, 0.8), BackgroundColor),
    /// Type: Color. Default value: Color(0.1, 0.5, 1.0, 1.0). Maps to: HighlightColor
    (ButtonHighlightColor,  "button.highlight-color",  Color::new(0.1, 0.5, 1.0, 1.0), HighlightColor),

    /// Type: Color. Default value: Color(0.3, 0.3, 0.3, 0.5). Maps to: BackgroundColor
    (CheckboxBackgroundColor, "checkbox.background-color", Color::new(0.3, 0.3, 0.3, 0.5), BackgroundColor),
    /// Type: Color. Default value: Color(0.3, 0.3, 0.3, 1.0). Maps to: HighlightColor
    (CheckboxHighlightColor,  "checkbox.highlight-color",  Color::new(0.3, 0.3, 0.3, 1.0), HighlightColor),

    /// Type: boolean. Default value: true
    (ScrollAreaHScrollShow,                   "scroll-area.hscroll.show", true),
    /// Type: boolean. Default value: true
    (ScrollAreaVScrollShow,                   "scroll-area.vscroll.show", true),
    /// Type: f32. Default value: 4
    (ScrollAreaScrollbarLength,               "scroll-area.scrollbar.length", 4f32),
    /// Type: Color. Default value: Color(0.3, 0.3, 0.3, 0.3)
    (ScrollAreaScrollbarBackgroundColor,      "scroll-area.scrollbar.background-color", Color::new(0.3, 0.3, 0.3, 0.3)),
    /// Type: f32. Default value: 20
    (ScrollAreaScrollScale,                   "scroll-area.scroll-scale", 20f32),
    /// Type: u64. Default value: 400
    (ScrollAreaScrollbarVisibilityStableTime, "scroll-area.scrollbar.visibility-stable-time", 400u64),
    /// Type: u64. Default value: 200
    (ScrollAreaScrollbarVisibilityFadeTime,   "scroll-area.scrollbar.visibility-fade-time", 200u64),
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
