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

macro_rules! decl_property {
    ($($v:ident($ty:ident),)*) => {
        #[derive(Clone)]
        pub enum HmGuiProperty {
            $($v($ty)),*
        }

        impl HmGuiProperty {
            pub fn name(&self) -> &'static str {
                match self {
                    $(Self::$v(_) => stringify!($v),)*
                }
            }

            pub fn get_type(&self) -> HmGuiPropertyType {
                match self {
                    $(Self::$v(_) => HmGuiPropertyType::$v,)*
                }
            }
        }

        $(
            impl From<$ty> for HmGuiProperty {
                fn from(value: $ty) -> Self {
                    Self::$v(value)
                }
            }
        )*

        #[luajit_ffi_gen::luajit_ffi]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum HmGuiPropertyType {
            $($v),*
        }
    };
}

decl_property! {
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
