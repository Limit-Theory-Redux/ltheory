//! Named stroke looks: values of the `STROKE_LOOK` parameters (core, glow,
//! shade, dark halo) under a name. Lua defines them once (`Gui:defineLook`,
//! `Gui:setLookParam`, wrapped by `UICore.Look.define`), widgets pick one by
//! name (`Gui:setStrokeLook`) and may override single values
//! (`Gui:setStrokeLookParam`). Change a look in one place and every widget
//! that uses it follows.
//!
//! The shape calls from before strokes (`Gui:annulus`, `ring`, `wedge`,
//! `line`) draw with built-in looks named after them, one per blend
//! (`wedge`, `wedgeGlow`, ...), holding the constants of the shaders they used
//! to have. Redefine one of those from Lua to restyle every such widget.

use std::collections::HashMap;

use crate::render::{STROKE_LOOK, STROKE_LOOK_LEN, StrokeLook};

/// A look, by its place in `HmGuiLooks`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HmGuiLookId(u16);

impl HmGuiLookId {
    /// The look of a stroke that names none (the parameter defaults: a plain
    /// line in its colour).
    pub const STROKE: Self = Self(0);
    pub const LINE: Self = Self(1);
    pub const LINE_GLOW: Self = Self(2);
    pub const RING: Self = Self(3);
    pub const RING_GLOW: Self = Self(4);
    pub const WEDGE: Self = Self(5);
    pub const WEDGE_GLOW: Self = Self(6);
    pub const ANNULUS: Self = Self(7);
    pub const ANNULUS_GLOW: Self = Self(8);
}

/// Single look values a widget sets for itself (by slot), on top of its
/// named look.
pub type HmGuiLookOverrides = [Option<f32>; STROKE_LOOK_LEN];

/// The built-in looks, in the order of their ids. Values in `STROKE_LOOK`
/// order: core, coreWidth, glow, glowWidth, boost, shade, halo, haloWidth.
///
/// The `*Glow` looks only add light (shade 0) and are the constants of the
/// shaders those shapes had. The alpha looks of line, ring and wedge are
/// fitted to how those shapes looked when alpha blended: their shaders
/// wrote straight colour times coverage, which the blend multiplied by the
/// coverage again (as `text.glsl` does), so their light fell off with the
/// square of the glow curve while they covered the background with the curve
/// itself. A stroke's light is colour times coverage, so these looks have a
/// tighter glow for the light and a dark halo for the covering: within 10/255
/// of the old light and 16/255 of the old covering.
const BUILT_IN: [(&str, StrokeLook); 9] = [
    ("stroke", [1.0, 0.5, 0.0, 4.0, 1.0, 1.0, 0.0, 8.0]),
    // `line.glsl`: core 0.8 falling by e every 0.5 px, glow 0.2 per 5 px.
    ("line", [0.86, 0.25, 0.12, 2.0, 1.0, 1.0, 0.35, 4.0]),
    ("lineGlow", [0.8, 0.5, 0.2, 5.0, 1.0, 0.0, 0.0, 8.0]),
    // `ring.glsl`: core 0.3, glow 0.4 per 5 px (`RingGlow` adds 0.4 more),
    // colour doubled.
    ("ring", [0.28, 0.7, 0.22, 1.25, 2.0, 1.0, 0.3, 12.0]),
    ("ringGlow", [0.3, 1.0, 0.8, 5.0, 2.0, 0.0, 0.0, 8.0]),
    // `wedge.glsl`: core 0.5, glow 0.4 per 1 / 0.3 px, colour doubled.
    ("wedge", [0.58, 0.75, 0.24, 0.5, 2.0, 1.0, 0.3, 9.0]),
    ("wedgeGlow", [0.5, 1.0, 0.4, 10.0 / 3.0, 2.0, 0.0, 0.0, 8.0]),
    // `annulus.glsl`: a solid band with a soft edge (`AnnulusGlow`: plus a
    // glow of 0.4 per 5 px).
    ("annulus", [1.0, 0.7, 0.0, 5.0, 1.0, 1.0, 0.0, 8.0]),
    ("annulusGlow", [1.0, 0.7, 0.4, 5.0, 1.0, 0.0, 0.0, 8.0]),
];

/// The slot of the look parameter `param`, or a message naming the known ones.
pub fn look_param_slot(param: &str) -> Result<usize, String> {
    STROKE_LOOK.slot(param).ok_or_else(|| {
        format!(
            "unknown look parameter '{param}' (known: {})",
            STROKE_LOOK.names()
        )
    })
}

/// The named looks of one `HmGui`. They outlive frames: define them once.
#[derive(Debug, Clone)]
pub struct HmGuiLooks {
    names: HashMap<String, HmGuiLookId>,
    values: Vec<StrokeLook>,
}

impl HmGuiLooks {
    pub fn new() -> Self {
        let mut looks = Self {
            names: HashMap::new(),
            values: Vec::new(),
        };
        for (name, values) in BUILT_IN {
            let id = looks.insert(name);
            looks.values[id.0 as usize] = values;
        }
        looks
    }

    fn insert(&mut self, name: &str) -> HmGuiLookId {
        let id = HmGuiLookId(self.values.len() as u16);
        self.values.push(STROKE_LOOK.defaults());
        self.names.insert(name.into(), id);
        id
    }

    /// The look called `name`, if it exists.
    pub fn find(&self, name: &str) -> Option<HmGuiLookId> {
        self.names.get(name).copied()
    }

    /// The look called `name`; a new one with the default values if it
    /// doesn't exist yet (the flag says so).
    pub fn get_or_insert(&mut self, name: &str) -> (HmGuiLookId, bool) {
        match self.find(name) {
            Some(id) => (id, false),
            None => (self.insert(name), true),
        }
    }

    /// (Re)define the look `name` with the default values, so values set
    /// next describe it completely.
    pub fn define(&mut self, name: &str) -> HmGuiLookId {
        let (id, _) = self.get_or_insert(name);
        self.values[id.0 as usize] = STROKE_LOOK.defaults();
        id
    }

    /// Set one value of the look `name` (created with the defaults if new).
    pub fn set_param(&mut self, name: &str, param: &str, value: f32) -> Result<(), String> {
        let slot = look_param_slot(param)?;
        let (id, _) = self.get_or_insert(name);
        self.values[id.0 as usize][slot] = value;
        Ok(())
    }

    pub fn values(&self, id: HmGuiLookId) -> &StrokeLook {
        &self.values[id.0 as usize]
    }

    /// The look `id` with a widget's own values on top.
    pub fn resolve(&self, id: HmGuiLookId, overrides: &HmGuiLookOverrides) -> StrokeLook {
        let mut look = *self.values(id);
        for (value, own) in look.iter_mut().zip(overrides) {
            if let Some(own) = own {
                *value = *own;
            }
        }
        look
    }
}

impl Default for HmGuiLooks {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_looks_have_their_ids() {
        let looks = HmGuiLooks::new();
        let ids = [
            ("stroke", HmGuiLookId::STROKE),
            ("line", HmGuiLookId::LINE),
            ("lineGlow", HmGuiLookId::LINE_GLOW),
            ("ring", HmGuiLookId::RING),
            ("ringGlow", HmGuiLookId::RING_GLOW),
            ("wedge", HmGuiLookId::WEDGE),
            ("wedgeGlow", HmGuiLookId::WEDGE_GLOW),
            ("annulus", HmGuiLookId::ANNULUS),
            ("annulusGlow", HmGuiLookId::ANNULUS_GLOW),
        ];
        for (name, id) in ids {
            assert_eq!(looks.find(name), Some(id), "{name}");
        }
        // The plain stroke is the parameter defaults.
        let defaults: StrokeLook = STROKE_LOOK.defaults();
        assert_eq!(looks.values(HmGuiLookId::STROKE), &defaults);
        // Glow twins differ from their alpha look in shade (and a ring's or
        // annulus' glow).
        let shade = STROKE_LOOK.slot("shade").unwrap();
        assert_eq!(looks.values(HmGuiLookId::WEDGE)[shade], 1.0);
        assert_eq!(looks.values(HmGuiLookId::WEDGE_GLOW)[shade], 0.0);
    }

    #[test]
    fn looks_are_defined_by_name() {
        let mut looks = HmGuiLooks::new();
        looks.set_param("hudNeon", "glow", 0.4).unwrap();
        looks.set_param("hudNeon", "glowWidth", 10.0).unwrap();
        looks.set_param("hudNeon", "shade", 0.0).unwrap();
        let id = looks.find("hudNeon").expect("created by its first value");
        let values = looks.values(id);
        assert_eq!(values[STROKE_LOOK.slot("glow").unwrap()], 0.4);
        assert_eq!(values[STROKE_LOOK.slot("glowWidth").unwrap()], 10.0);
        assert_eq!(values[STROKE_LOOK.slot("core").unwrap()], 1.0); // default

        // Defining it again starts from the defaults.
        assert_eq!(looks.define("hudNeon"), id);
        let defaults: StrokeLook = STROKE_LOOK.defaults();
        assert_eq!(looks.values(id), &defaults);

        // Unknown parameters are refused with the list of known ones.
        let err = looks.set_param("hudNeon", "glowStrength", 1.0).unwrap_err();
        assert!(
            err.contains("glowStrength") && err.contains("glowWidth"),
            "{err}"
        );
    }

    #[test]
    fn widgets_override_single_values() {
        let mut looks = HmGuiLooks::new();
        looks.set_param("hudNeon", "glow", 0.4).unwrap();
        let id = looks.find("hudNeon").unwrap();

        let mut overrides: HmGuiLookOverrides = Default::default();
        let glow = STROKE_LOOK.slot("glow").unwrap();
        overrides[glow] = Some(0.8);
        let resolved = looks.resolve(id, &overrides);
        assert_eq!(resolved[glow], 0.8);
        // The named look itself is unchanged.
        assert_eq!(looks.values(id)[glow], 0.4);
        assert_eq!(
            resolved[STROKE_LOOK.slot("core").unwrap()],
            looks.values(id)[STROKE_LOOK.slot("core").unwrap()]
        );
    }
}
