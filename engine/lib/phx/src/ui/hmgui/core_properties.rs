use glam::*;
use indexmap::IndexMap;

use crate::render::Font;

use super::HmGuiProperty;

macro_rules! core_properties {
    ($(($v:ident, $n:literal, $d:expr),)*) => {
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

        pub fn register_core_properties() -> IndexMap<String, HmGuiProperty> {
            let mut r = Default::default();

            $(reg(&mut r, $n, $d, HmGuiProperties::$v);)*

            r
        }
    };
}

core_properties! {
    (ContainerSpacingId,      "container.spacing",       6.0f32),
    (ContainerColorFrameId,   "container.color-frame",   Vec4::new(0.1, 0.1, 0.1, 0.5)),
    (ContainerColorPrimaryId, "container.color-primary", Vec4::new(0.1, 0.5, 1.0, 1.0)),
    (TextFontId,              "text.font",               Font::load("Rajdhani", 14)),
    (TextColorId,             "text.color",              Vec4::ONE),
    (ButtonBorderWidthId,     "button.border-width",     0.0f32),
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
