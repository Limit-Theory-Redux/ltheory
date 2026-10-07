use glam::{IVec2, IVec3, IVec4, Vec2, Vec3, Vec4};

use super::{Tex1D, Tex2D, Tex3D, TexCube};
use crate::math::Matrix;

/// A value `ShaderState` and the legacy `Shader:set*` path send to a uniform.
#[derive(Clone)]
pub enum ShaderVarData {
    Float(f32),
    Float2(Vec2),
    Float3(Vec3),
    Float4(Vec4),
    Int(i32),
    Int2(IVec2),
    Int3(IVec3),
    Int4(IVec4),
    Matrix(Matrix),
    Tex1D(Tex1D),
    Tex2D(Tex2D),
    Tex3D(Tex3D),
    TexCube(TexCube),
}

impl ShaderVarData {
    /// Whether two values are identical for uniform-dedup purposes. Only pure
    /// value types (float/int/matrix) can be deduplicated: sampler types have
    /// side effects (texture-unit allocation) and are always re-sent.
    pub fn same_value(&self, other: &ShaderVarData) -> bool {
        match (self, other) {
            (ShaderVarData::Float(a), ShaderVarData::Float(b)) => a == b,
            (ShaderVarData::Float2(a), ShaderVarData::Float2(b)) => a == b,
            (ShaderVarData::Float3(a), ShaderVarData::Float3(b)) => a == b,
            (ShaderVarData::Float4(a), ShaderVarData::Float4(b)) => a == b,
            (ShaderVarData::Int(a), ShaderVarData::Int(b)) => a == b,
            (ShaderVarData::Int2(a), ShaderVarData::Int2(b)) => a == b,
            (ShaderVarData::Int3(a), ShaderVarData::Int3(b)) => a == b,
            (ShaderVarData::Int4(a), ShaderVarData::Int4(b)) => a == b,
            (ShaderVarData::Matrix(a), ShaderVarData::Matrix(b)) => {
                a.to_cols_array() == b.to_cols_array()
            }
            // Samplers and mismatched types: never dedup
            _ => false,
        }
    }
}
