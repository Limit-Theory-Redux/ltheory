use indexmap::IndexMap;

use crate::render::{Color, Font};

use super::{HmGuiProperty, HmGuiPropertyId, HmGuiPropertyValue};

// Property id, name, value and optional list of mapped property ids
phx_macros::core_properties! {
    (Opacity,               "opacity",                 1.0f32),
    (BorderColor,           "border-color",            Color::TRANSPARENT),
    (BackgroundColor,       "background-color",        Color::TRANSPARENT),
    (HighlightColor,        "highlight-color",         Color::TRANSPARENT),
    (TextFont,              "text.font",               Font::load("Rajdhani", 14)),
    (TextColor,             "text.color",              Color::WHITE),

    (ContainerClip,    "container.clip",    true),
    (ContainerSpacing, "container.spacing", 0.0f32),

    (ButtonRectOpacity,         "button.rect.opacity",          0.5f32, Opacity),
    (ButtonRectBorderColor,     "button.rect.border-color",     Color::TRANSPARENT, BorderColor),
    (ButtonRectBackgroundColor, "button.rect.background-color", Color::new(0.15, 0.15, 0.15, 0.8), BackgroundColor),
    (ButtonRectHighlightColor,  "button.rect.highlight-color",  Color::new(0.1, 0.5, 1.0, 1.0), HighlightColor),
    (ButtonTextOpacity,         "button.text.opacity",          0.5f32, Opacity),
    (ButtonTextBackgroundColor, "button.text.background-color", Color::TRANSPARENT, BackgroundColor),
    (ButtonTextHighlightColor,  "button.text.highlight-color",  Color::TRANSPARENT, HighlightColor),
    (ButtonTextFont,            "button.text.font",             Font::load("Rajdhani", 14), TextFont),
    (ButtonTextColor,           "button.text.color",            Color::WHITE, TextColor),

    (CheckboxRectOpacity,          "checkbox.rect.opacity",          0.5f32, Opacity),
    (CheckboxRectBorderColor,      "checkbox.rect.border-color",     Color::TRANSPARENT, BorderColor),
    (CheckboxRectBackgroundColor,  "checkbox.rect.background-color", Color::new(0.3, 0.3, 0.3, 0.5), BackgroundColor),
    (CheckboxRectHighlightColor,   "checkbox.rect.highlight-color",  Color::new(0.3, 0.3, 0.3, 1.0), HighlightColor),
    (CheckboxTextOpacity,          "checkbox.text.opacity",          0.5f32, Opacity),
    (CheckboxTextBackgroundColor,  "checkbox.text.background-color", Color::TRANSPARENT, BackgroundColor),
    (CheckboxTextHighlightColor,   "checkbox.text.highlight-color",  Color::TRANSPARENT, HighlightColor),
    (CheckboxTextFont,             "checkbox.text.font",             Font::load("Rajdhani", 14), TextFont),
    (CheckboxTextColor,            "checkbox.text.color",            Color::WHITE, TextColor),
    (CheckboxClickAreaBorderColor,     "checkbox.click-area.border-color",     Color::new(0.1, 0.1, 0.1, 0.5), BorderColor),
    (CheckboxClickAreaBackgroundColor, "checkbox.click-area.background-color", Color::TRANSPARENT, BackgroundColor),
    (CheckboxClickAreaHighlightColor,  "checkbox.click-area.highlight-color",  Color::TRANSPARENT, HighlightColor),
    (CheckboxClickAreaSelectedColor,   "checkbox.click-area.selected-color",   Color::new(0.1, 0.5, 1.0, 1.0)),

    (ScrollAreaHScrollShow,                   "scroll-area.hscroll.show", true),
    (ScrollAreaVScrollShow,                   "scroll-area.vscroll.show", true),
    (ScrollAreaScrollScale,                   "scroll-area.scroll-scale", 20f32),
    (ScrollAreaScrollbarLength,               "scroll-area.scrollbar.length", 4f32),
    (ScrollAreaScrollbarBackgroundColor,      "scroll-area.scrollbar.background-color", Color::new(0.3, 0.3, 0.3, 0.3)),
    (ScrollAreaScrollbarVisibilityFading,     "scroll-area.scrollbar.visibility-fading", true),
    /// Time in milliseconds for how long scrollbar is visible fading
    (ScrollAreaScrollbarVisibilityStableTime, "scroll-area.scrollbar.visibility-stable-time", 400u64),
    /// Time in milliseconds for how long scrollbar is fading
    (ScrollAreaScrollbarVisibilityFadeTime,   "scroll-area.scrollbar.visibility-fade-time", 200u64),
    (ScrollAreaScrollbarKnobColor,            "scroll-area.scrollbar.knob-color",   Color::new(0.1, 0.1, 0.1, 0.5)),
}

/// Adds a new property to the map.
/// Verifies its expected id and mapped ids.
#[inline]
fn reg<T: Into<HmGuiPropertyValue>>(
    r: &mut IndexMap<String, HmGuiProperty>,
    name: &str,
    value: T,
    expected_id: HmGuiProperties,
    map_ids: &[HmGuiPropertyId],
) {
    assert!(r.get(name).is_none(), "Property {name:?} already exists");

    let id = r.len();
    assert_eq!(id, expected_id as _, "Wrong property id");

    let property: HmGuiPropertyValue = value.into();

    for map_id in map_ids {
        assert_ne!(**map_id, id, "Property {name:?} maps to itself"); // TODO: check for the circular dependency

        let (map_name, property_info) = r.get_index(**map_id).expect("Unknown pam property");
        assert_eq!(
            property.get_type(),
            property_info.value.get_type(),
            "Wrong {map_name:?} map property type"
        );
    }

    r.insert(
        name.into(),
        HmGuiProperty {
            value: property,
            map_ids: map_ids.into(),
        },
    );
}
