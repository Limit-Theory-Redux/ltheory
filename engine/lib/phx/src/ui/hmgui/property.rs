use std::ops::Deref;

use glam::*;
use internal::ConvertIntoString;

use crate::math::Box3;
use crate::render::{Color, Font};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HmGuiPropertyId(usize);

impl From<usize> for HmGuiPropertyId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl Deref for HmGuiPropertyId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Property information:
/// - property value
/// - ids of the properties this one should be merged into
#[derive(Clone, PartialEq)]
pub struct HmGuiProperty {
    pub value: HmGuiPropertyValue,
    pub map_ids: Vec<HmGuiPropertyId>,
}

phx_macros::define_properties![
    bool, i8, u8, i16, u16, i32, u32, i64, u64, f32, f64, Vec2, Vec3, Vec4, IVec2, IVec3, IVec4,
    UVec2, UVec3, UVec4, DVec2, DVec3, DVec4, Color, Box3, String, Font,
];
