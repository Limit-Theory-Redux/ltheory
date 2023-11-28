use std::ops::Deref;

use glam::*;

use crate::{math::Box3, render::Font};

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

#[derive(Clone)]
pub enum HmGuiProperty {
    Bool(bool),
    I8(i8),
    U8(u8),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    F32(f32),
    F64(f64),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
    IVec2(IVec2),
    IVec3(IVec3),
    IVec4(IVec4),
    UVec2(UVec2),
    UVec3(UVec3),
    UVec4(UVec4),
    DVec2(DVec2),
    DVec3(DVec3),
    DVec4(DVec4),
    Box3(Box3),
    String(String),
    Font(Font),
}

impl HmGuiProperty {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Bool(_) => "Bool",
            Self::I8(_) => "I8",
            Self::U8(_) => "U8",
            Self::I16(_) => "I16",
            Self::U16(_) => "U16",
            Self::I32(_) => "I32",
            Self::U32(_) => "U32",
            Self::I64(_) => "I64",
            Self::U64(_) => "U64",
            Self::F32(_) => "F32",
            Self::F64(_) => "F64",
            Self::Vec2(_) => "Vec2",
            Self::Vec3(_) => "Vec3",
            Self::Vec4(_) => "Vec4",
            Self::IVec2(_) => "IVec2",
            Self::IVec3(_) => "IVec3",
            Self::IVec4(_) => "IVec4",
            Self::UVec2(_) => "UVec2",
            Self::UVec3(_) => "UVec3",
            Self::UVec4(_) => "UVec4",
            Self::DVec2(_) => "DVec2",
            Self::DVec3(_) => "DVec3",
            Self::DVec4(_) => "DVec4",
            Self::Box3(_) => "Box3",
            Self::String(_) => "String",
            Self::Font(_) => "Font",
        }
    }
}

macro_rules! decl_from {
    ($v:ident, $ty:ident) => {
        impl From<$ty> for HmGuiProperty {
            fn from(value: $ty) -> Self {
                Self::$v(value)
            }
        }
    };
}

decl_from!(Bool, bool);
decl_from!(I8, i8);
decl_from!(U8, u8);
decl_from!(I16, i16);
decl_from!(U16, u16);
decl_from!(I32, i32);
decl_from!(U32, u32);
decl_from!(I64, i64);
decl_from!(U64, u64);
decl_from!(F32, f32);
decl_from!(F64, f64);
decl_from!(Vec2, Vec2);
decl_from!(Vec3, Vec3);
decl_from!(Vec4, Vec4);
decl_from!(IVec2, IVec2);
decl_from!(IVec3, IVec3);
decl_from!(IVec4, IVec4);
decl_from!(UVec2, UVec2);
decl_from!(UVec3, UVec3);
decl_from!(UVec4, UVec4);
decl_from!(DVec2, DVec2);
decl_from!(DVec3, DVec3);
decl_from!(DVec4, DVec4);
decl_from!(Box3, Box3);
decl_from!(String, String);
decl_from!(Font, Font);
