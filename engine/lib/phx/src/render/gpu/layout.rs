//! Shader binding layout (doc/engine/render-api-v2.md, sections 1.3 and 2).
//!
//! GLSL 330 has no `layout(binding=)`, so the shader preprocessor owns the
//! group/binding assignment. `#group N` marks the declarations that follow
//! (uniform blocks and samplers) as belonging to bind group N. The
//! preprocessor strips the directive, records the declarations in a
//! [`ShaderLayout`], and the GL executor applies them at link time
//! (`glUniformBlockBinding` / `glUniform1i`). The wgpu path rewrites the same
//! declarations to `set`/`binding` form.
//!
//! Fixed assignment, identical for every shader:
//!
//! | group | uniform block bindings | texture units |
//! |---|---|---|
//! | 0 frame/view | 0..3 | 0..2 |
//! | 1 material | 4..7 | 3..9 |
//! | 2 draw | 8..11 | 10..11 |
//! | 3 pass inputs | 12..15 | 12..15 |

use std::collections::HashMap;

pub const GROUP_COUNT: usize = 4;
/// First GL texture unit of each group.
pub const GROUP_FIRST_UNIT: [u32; GROUP_COUNT] = [0, 3, 10, 12];
/// Number of texture units each group owns.
pub const GROUP_UNIT_COUNT: [u32; GROUP_COUNT] = [3, 7, 2, 4];
/// Uniform block binding points per group (`binding = group * 4 + k`).
pub const BLOCKS_PER_GROUP: u32 = 4;
/// Total GL texture units the layout uses (the GL 3.3 minimum).
pub const TOTAL_UNITS: u32 = 16;

pub const GROUP_FRAME: u8 = 0;
pub const GROUP_MATERIAL: u8 = 1;
pub const GROUP_DRAW: u8 = 2;
pub const GROUP_INPUTS: u8 = 3;

/// Block binding point of the `k`-th block in `group`.
pub const fn block_binding(group: u8, k: u8) -> u32 {
    group as u32 * BLOCKS_PER_GROUP + k as u32
}

/// Texture unit of the `k`-th sampler in `group`.
pub const fn texture_unit(group: u8, k: u8) -> u32 {
    GROUP_FIRST_UNIT[group as usize] + k as u32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TexDim {
    D1,
    D2,
    D3,
    Cube,
}

impl TexDim {
    pub fn from_sampler_type(ty: &str) -> Option<TexDim> {
        match ty {
            "sampler1D" => Some(TexDim::D1),
            "sampler2D" => Some(TexDim::D2),
            "sampler3D" => Some(TexDim::D3),
            "samplerCube" => Some(TexDim::Cube),
            _ => None,
        }
    }
}

/// One uniform block declared under a `#group`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockDecl {
    pub name: String,
    pub group: u8,
    /// Index within the group, in declaration order.
    pub index: u8,
}

impl BlockDecl {
    pub fn binding(&self) -> u32 {
        block_binding(self.group, self.index)
    }
}

/// One sampler declared under a `#group`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureDecl {
    pub name: String,
    pub group: u8,
    pub index: u8,
    pub dim: TexDim,
}

impl TextureDecl {
    pub fn unit(&self) -> u32 {
        texture_unit(self.group, self.index)
    }
}

/// What the preprocessor recorded for one stage, before the vertex and
/// fragment stages are merged into one program layout.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StageLayout {
    /// `(name, group)` in declaration order.
    pub blocks: Vec<(String, u8)>,
    /// `(name, group, dim)` in declaration order.
    pub textures: Vec<(String, u8, TexDim)>,
    /// Problems found while recording (bad `#group`, mismatched duplicates).
    pub errors: Vec<String>,
}

/// Program-wide layout: the merge of both stages' declarations, with
/// bindings assigned by first appearance (vertex first) within each group.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShaderLayout {
    pub blocks: Vec<BlockDecl>,
    pub textures: Vec<TextureDecl>,
}

impl ShaderLayout {
    /// Merge stage layouts. Returns the layout plus any errors (a group
    /// holding more blocks/samplers than its binding range, or one name
    /// declared with two different groups or dimensions).
    pub fn merge(stages: &[&StageLayout]) -> (ShaderLayout, Vec<String>) {
        let mut layout = ShaderLayout::default();
        let mut errors = Vec::new();
        let mut block_counts = [0u32; GROUP_COUNT];
        let mut texture_counts = [0u32; GROUP_COUNT];

        for stage in stages {
            errors.extend(stage.errors.iter().cloned());
            for (name, group) in &stage.blocks {
                if let Some(existing) = layout.blocks.iter().find(|b| &b.name == name) {
                    if existing.group != *group {
                        errors.push(format!(
                            "uniform block '{name}' declared in group {} and group {group}",
                            existing.group
                        ));
                    }
                    continue;
                }
                let g = *group as usize;
                if block_counts[g] >= BLOCKS_PER_GROUP {
                    errors.push(format!(
                        "group {group} declares more than {BLOCKS_PER_GROUP} uniform blocks ('{name}')"
                    ));
                    continue;
                }
                layout.blocks.push(BlockDecl {
                    name: name.clone(),
                    group: *group,
                    index: block_counts[g] as u8,
                });
                block_counts[g] += 1;
            }
            for (name, group, dim) in &stage.textures {
                if let Some(existing) = layout.textures.iter().find(|t| &t.name == name) {
                    if existing.group != *group || existing.dim != *dim {
                        errors.push(format!(
                            "sampler '{name}' declared as {:?} in group {} and {dim:?} in group {group}",
                            existing.dim, existing.group
                        ));
                    }
                    continue;
                }
                let g = *group as usize;
                if texture_counts[g] >= GROUP_UNIT_COUNT[g] {
                    errors.push(format!(
                        "group {group} declares more than {} samplers ('{name}')",
                        GROUP_UNIT_COUNT[g]
                    ));
                    continue;
                }
                layout.textures.push(TextureDecl {
                    name: name.clone(),
                    group: *group,
                    index: texture_counts[g] as u8,
                    dim: *dim,
                });
                texture_counts[g] += 1;
            }
        }
        (layout, errors)
    }

    pub fn block(&self, name: &str) -> Option<&BlockDecl> {
        self.blocks.iter().find(|b| b.name == name)
    }

    pub fn texture(&self, name: &str) -> Option<&TextureDecl> {
        self.textures.iter().find(|t| t.name == name)
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty() && self.textures.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Reflected block layouts (GL: glGetActiveUniformBlockiv/glGetActiveUniformsiv;
// wgpu: naga's module).
// ---------------------------------------------------------------------------

/// GLSL type of a block member, as far as the engine's FFI types go.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlslType {
    Float,
    Int,
    UInt,
    Bool,
    Vec2,
    Vec3,
    Vec4,
    IVec2,
    IVec3,
    IVec4,
    UVec2,
    UVec3,
    UVec4,
    Mat3,
    Mat4,
    /// Anything else (matrices of other shapes, nested structs). Exposed to
    /// Lua as opaque padding bytes.
    Other,
}

impl GlslType {
    /// Size in bytes of one element when tightly packed (no std140 padding).
    pub fn packed_size(self) -> u32 {
        match self {
            GlslType::Float | GlslType::Int | GlslType::UInt | GlslType::Bool => 4,
            GlslType::Vec2 | GlslType::IVec2 | GlslType::UVec2 => 8,
            GlslType::Vec3 | GlslType::IVec3 | GlslType::UVec3 => 12,
            GlslType::Vec4 | GlslType::IVec4 | GlslType::UVec4 => 16,
            GlslType::Mat3 => 36,
            GlslType::Mat4 => 64,
            GlslType::Other => 0,
        }
    }

    /// FFI type used in the generated Lua struct, with its packed size.
    fn lua_type(self) -> Option<&'static str> {
        Some(match self {
            GlslType::Float => "float",
            GlslType::Int | GlslType::Bool => "int32_t",
            GlslType::UInt => "uint32_t",
            GlslType::Vec2 => "Vec2f",
            GlslType::Vec3 => "Vec3f",
            GlslType::Vec4 => "Vec4f",
            GlslType::IVec2 => "Vec2i",
            GlslType::IVec3 => "Vec3i",
            GlslType::IVec4 => "Vec4i",
            GlslType::UVec2 => "Vec2u",
            GlslType::UVec3 => "Vec3u",
            GlslType::UVec4 => "Vec4u",
            GlslType::Mat4 => "float[16]",
            GlslType::Mat3 | GlslType::Other => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockMember {
    pub name: String,
    pub ty: GlslType,
    pub offset: u32,
    /// Array length; 1 for a non-array member.
    pub count: u32,
    pub array_stride: u32,
    pub matrix_stride: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockLayout {
    pub name: String,
    /// Data size in bytes (`GL_UNIFORM_BLOCK_DATA_SIZE`).
    pub size: u32,
    pub members: Vec<BlockMember>,
}

impl BlockLayout {
    /// The expected member of a fixed block, by name.
    pub fn member(&self, name: &str) -> Option<&BlockMember> {
        self.members.iter().find(|m| m.name == name)
    }

    /// A LuaJIT `ffi.typeof` declaration (an anonymous struct) with the exact
    /// byte layout of this block: members at their std140 offsets, explicit
    /// `_padN` fields for the holes, trailing pad up to the block size.
    /// Anonymous so it can be regenerated on hot reload without colliding
    /// with `ffi.cdef`. Arrays are exposed as raw 4-byte lanes
    /// (`float name[count * stride / 4]`, index by stride).
    pub fn lua_struct(&self) -> String {
        let mut members: Vec<&BlockMember> = self.members.iter().collect();
        members.sort_by_key(|m| m.offset);

        let mut out = String::from("struct { ");
        let mut cursor = 0u32;
        let mut pad = 0u32;
        let mut push_pad = |out: &mut String, bytes: u32| {
            if bytes > 0 {
                out.push_str(&format!("uint8_t _pad{pad}[{bytes}]; "));
                pad += 1;
            }
        };
        for m in members {
            if m.offset < cursor {
                // Overlapping members cannot be expressed; skip (should not
                // happen for std140 blocks).
                continue;
            }
            push_pad(&mut out, m.offset - cursor);
            cursor = m.offset;
            let name = sanitize(&m.name);
            if m.count > 1 {
                let bytes = m.count * m.array_stride.max(m.ty.packed_size());
                out.push_str(&format!("float {name}[{}]; ", bytes / 4));
                cursor += bytes;
                continue;
            }
            match (m.ty.lua_type(), m.ty) {
                (Some(ty), GlslType::Mat4) => {
                    out.push_str(&format!("{} {name}[16]; ", ty.trim_end_matches("[16]")));
                    cursor += 64;
                }
                (Some(ty), t) => {
                    out.push_str(&format!("{ty} {name}; "));
                    cursor += t.packed_size();
                }
                (None, GlslType::Mat3) => {
                    // std140: three vec4 columns.
                    out.push_str(&format!("float {name}[12]; "));
                    cursor += 48;
                }
                (None, _) => {
                    // Unknown type: reserve nothing; the tail pad covers it.
                }
            }
        }
        push_pad(&mut out, self.size.saturating_sub(cursor));
        out.push('}');
        out
    }
}

fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Reflected blocks for a linked shader, by name.
pub fn index_blocks(blocks: &[BlockLayout]) -> HashMap<&str, &BlockLayout> {
    blocks.iter().map(|b| (b.name.as_str(), b)).collect()
}

// ---------------------------------------------------------------------------
// naga reflection (GL-free): used by the wgpu executor to build the same
// `BlockLayout`s, and by the layout-parity test.
// ---------------------------------------------------------------------------

/// Build `BlockLayout`s for every uniform block in a naga module that has a
/// real (non-synthetic) name.
pub fn blocks_from_naga(module: &wgpu::naga::Module) -> Vec<BlockLayout> {
    use wgpu::naga::{AddressSpace, ScalarKind, TypeInner, VectorSize};

    let mut out = Vec::new();
    for (_, var) in module.global_variables.iter() {
        if var.space != AddressSpace::Uniform {
            continue;
        }
        let ty = &module.types[var.ty];
        let TypeInner::Struct { members, span } = &ty.inner else {
            continue;
        };
        // The wgpu path packs loose uniforms into a synthetic `_phx_plain_UBO`.
        let name = ty
            .name
            .clone()
            .or_else(|| var.name.clone())
            .unwrap_or_default();
        if name.starts_with("_phx") || name.is_empty() {
            continue;
        }
        let mut layout_members = Vec::new();
        for m in members {
            let mty = &module.types[m.ty];
            let (glsl, count, stride) = match &mty.inner {
                TypeInner::Scalar(s) => (scalar_type(s.kind), 1, 0),
                TypeInner::Vector { size, scalar } => {
                    let n = match size {
                        VectorSize::Bi => 2,
                        VectorSize::Tri => 3,
                        VectorSize::Quad => 4,
                    };
                    (vector_type(scalar.kind, n), 1, 0)
                }
                TypeInner::Matrix { columns, rows, .. } => {
                    let t = match (columns, rows) {
                        (VectorSize::Quad, VectorSize::Quad) => GlslType::Mat4,
                        (VectorSize::Tri, VectorSize::Tri) => GlslType::Mat3,
                        _ => GlslType::Other,
                    };
                    (t, 1, 0)
                }
                TypeInner::Array { base, size, stride } => {
                    let elem = match &module.types[*base].inner {
                        TypeInner::Scalar(s) => scalar_type(s.kind),
                        TypeInner::Vector { size, scalar } => {
                            let n = match size {
                                VectorSize::Bi => 2,
                                VectorSize::Tri => 3,
                                VectorSize::Quad => 4,
                            };
                            vector_type(scalar.kind, n)
                        }
                        TypeInner::Matrix {
                            columns: VectorSize::Quad,
                            rows: VectorSize::Quad,
                            ..
                        } => GlslType::Mat4,
                        _ => GlslType::Other,
                    };
                    let count = match size {
                        wgpu::naga::ArraySize::Constant(n) => n.get(),
                        _ => 1,
                    };
                    (elem, count, *stride)
                }
                _ => (GlslType::Other, 1, 0),
            };
            let _ = ScalarKind::Float;
            layout_members.push(BlockMember {
                name: m.name.clone().unwrap_or_default(),
                ty: glsl,
                offset: m.offset,
                count,
                array_stride: stride,
                matrix_stride: if glsl == GlslType::Mat4 { 16 } else { 0 },
            });
        }
        // std140 sizes round up to 16 bytes.
        let size = (*span).div_ceil(16) * 16;
        out.push(BlockLayout {
            name,
            size,
            members: layout_members,
        });
    }
    out
}

fn scalar_type(kind: wgpu::naga::ScalarKind) -> GlslType {
    use wgpu::naga::ScalarKind;
    match kind {
        ScalarKind::Float => GlslType::Float,
        ScalarKind::Sint => GlslType::Int,
        ScalarKind::Uint => GlslType::UInt,
        ScalarKind::Bool => GlslType::Bool,
        _ => GlslType::Other,
    }
}

fn vector_type(kind: wgpu::naga::ScalarKind, n: u32) -> GlslType {
    use wgpu::naga::ScalarKind;
    match (kind, n) {
        (ScalarKind::Float, 2) => GlslType::Vec2,
        (ScalarKind::Float, 3) => GlslType::Vec3,
        (ScalarKind::Float, 4) => GlslType::Vec4,
        (ScalarKind::Sint, 2) => GlslType::IVec2,
        (ScalarKind::Sint, 3) => GlslType::IVec3,
        (ScalarKind::Sint, 4) => GlslType::IVec4,
        (ScalarKind::Uint, 2) => GlslType::UVec2,
        (ScalarKind::Uint, 3) => GlslType::UVec3,
        (ScalarKind::Uint, 4) => GlslType::UVec4,
        _ => GlslType::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn units_and_bindings_are_fixed() {
        assert_eq!(block_binding(0, 0), 0);
        assert_eq!(block_binding(1, 0), 4);
        assert_eq!(block_binding(2, 1), 9);
        assert_eq!(block_binding(3, 3), 15);
        assert_eq!(texture_unit(0, 1), 1);
        assert_eq!(texture_unit(1, 0), 3);
        assert_eq!(texture_unit(2, 1), 11);
        assert_eq!(texture_unit(3, 3), 15);
        // The unit ranges tile 0..16 with no overlap.
        let mut next = 0;
        for g in 0..GROUP_COUNT {
            assert_eq!(GROUP_FIRST_UNIT[g], next);
            next += GROUP_UNIT_COUNT[g];
        }
        assert_eq!(next, TOTAL_UNITS);
    }

    #[test]
    fn merge_dedupes_and_numbers_in_declaration_order() {
        let vs = StageLayout {
            blocks: vec![("ViewBlock".into(), 0)],
            textures: vec![("envMap".into(), 0, TexDim::Cube)],
            errors: vec![],
        };
        let fs = StageLayout {
            blocks: vec![("ViewBlock".into(), 0), ("Params".into(), 2)],
            textures: vec![
                ("envMap".into(), 0, TexDim::Cube),
                ("irMap".into(), 0, TexDim::Cube),
                ("src".into(), 3, TexDim::D2),
            ],
            errors: vec![],
        };
        let (layout, errors) = ShaderLayout::merge(&[&vs, &fs]);
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(layout.block("ViewBlock").unwrap().binding(), 0);
        assert_eq!(layout.block("Params").unwrap().binding(), 8);
        assert_eq!(layout.texture("envMap").unwrap().unit(), 0);
        assert_eq!(layout.texture("irMap").unwrap().unit(), 1);
        assert_eq!(layout.texture("src").unwrap().unit(), 12);
    }

    #[test]
    fn merge_reports_overflow_and_conflicts() {
        let fs = StageLayout {
            blocks: vec![],
            textures: (0..4).map(|i| (format!("t{i}"), 2u8, TexDim::D2)).collect(),
            errors: vec![],
        };
        let (_, errors) = ShaderLayout::merge(&[&fs]);
        assert_eq!(errors.len(), 2, "{errors:?}"); // group 2 owns two units

        let a = StageLayout {
            blocks: vec![("B".into(), 1)],
            ..Default::default()
        };
        let b = StageLayout {
            blocks: vec![("B".into(), 2)],
            ..Default::default()
        };
        let (_, errors) = ShaderLayout::merge(&[&a, &b]);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn lua_struct_pads_std140_holes() {
        let block = BlockLayout {
            name: "Params".into(),
            size: 48,
            members: vec![
                BlockMember {
                    name: "color".into(),
                    ty: GlslType::Vec3,
                    offset: 0,
                    count: 1,
                    array_stride: 0,
                    matrix_stride: 0,
                },
                BlockMember {
                    name: "amount".into(),
                    ty: GlslType::Float,
                    offset: 12,
                    count: 1,
                    array_stride: 0,
                    matrix_stride: 0,
                },
                BlockMember {
                    name: "dir".into(),
                    ty: GlslType::Vec2,
                    offset: 16,
                    count: 1,
                    array_stride: 0,
                    matrix_stride: 0,
                },
            ],
        };
        assert_eq!(
            block.lua_struct(),
            "struct { Vec3f color; float amount; Vec2f dir; uint8_t _pad0[24]; }"
        );
    }
}
