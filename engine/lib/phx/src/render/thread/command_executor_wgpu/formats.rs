//! Pure mappings from engine enums to wgpu: blend, cull, topology, formats,
//! samplers and vertex layouts.

use super::*;

pub(super) fn blend_state(mode: BlendMode) -> Option<wgpu::BlendState> {
    const fn component(src: wgpu::BlendFactor, dst: wgpu::BlendFactor) -> wgpu::BlendComponent {
        wgpu::BlendComponent {
            src_factor: src,
            dst_factor: dst,
            operation: wgpu::BlendOperation::Add,
        }
    }
    match mode {
        BlendMode::Disabled => None,
        BlendMode::Alpha => Some(wgpu::BlendState {
            color: component(
                wgpu::BlendFactor::SrcAlpha,
                wgpu::BlendFactor::OneMinusSrcAlpha,
            ),
            alpha: component(wgpu::BlendFactor::One, wgpu::BlendFactor::OneMinusSrcAlpha),
        }),
        BlendMode::Additive => Some(wgpu::BlendState {
            color: component(wgpu::BlendFactor::One, wgpu::BlendFactor::One),
            alpha: component(wgpu::BlendFactor::One, wgpu::BlendFactor::One),
        }),
        BlendMode::PreMultAlpha => Some(wgpu::BlendState {
            color: component(wgpu::BlendFactor::One, wgpu::BlendFactor::OneMinusSrcAlpha),
            alpha: component(wgpu::BlendFactor::One, wgpu::BlendFactor::OneMinusSrcAlpha),
        }),
    }
}

pub(super) fn cull_mode(face: CullFace) -> Option<wgpu::Face> {
    match face {
        CullFace::None => None,
        CullFace::Back => Some(wgpu::Face::Back),
        CullFace::Front => Some(wgpu::Face::Front),
    }
}

pub(super) fn topology(topology: Topology) -> wgpu::PrimitiveTopology {
    match topology {
        Topology::Points => wgpu::PrimitiveTopology::PointList,
        Topology::Lines => wgpu::PrimitiveTopology::LineList,
        Topology::LineStrip => wgpu::PrimitiveTopology::LineStrip,
        Topology::Triangles => wgpu::PrimitiveTopology::TriangleList,
        Topology::TriangleStrip => wgpu::PrimitiveTopology::TriangleStrip,
    }
}

pub(super) fn compare_function(compare: CompareFn) -> wgpu::CompareFunction {
    match compare {
        CompareFn::Never => wgpu::CompareFunction::Never,
        CompareFn::Less => wgpu::CompareFunction::Less,
        CompareFn::Equal => wgpu::CompareFunction::Equal,
        CompareFn::LessEqual => wgpu::CompareFunction::LessEqual,
        CompareFn::Greater => wgpu::CompareFunction::Greater,
        CompareFn::NotEqual => wgpu::CompareFunction::NotEqual,
        CompareFn::GreaterEqual => wgpu::CompareFunction::GreaterEqual,
        CompareFn::Always => wgpu::CompareFunction::Always,
    }
}

pub(super) fn wrap_mode(mode: TexWrapMode) -> wgpu::AddressMode {
    match mode {
        TexWrapMode::Clamp => wgpu::AddressMode::ClampToEdge,
        // No wgpu equivalent; the nearest behaviour.
        TexWrapMode::MirrorClamp => wgpu::AddressMode::ClampToEdge,
        TexWrapMode::MirrorRepeat => wgpu::AddressMode::MirrorRepeat,
        TexWrapMode::Repeat => wgpu::AddressMode::Repeat,
    }
}

/// `SamplerDesc` to `wgpu::SamplerDescriptor`: filters, wrap on all three axes,
/// anisotropy and the LOD clamps (a sampler without a mip filter reads level
/// `lod_min` only, like GL's non-mipmap min filters).
pub(super) fn sampler_descriptor(desc: &SamplerDesc) -> wgpu::SamplerDescriptor<'static> {
    let filter = |f: SamplerFilter| match f {
        SamplerFilter::Point => wgpu::FilterMode::Nearest,
        SamplerFilter::Linear => wgpu::FilterMode::Linear,
    };
    let mip = match desc.mip {
        MipFilter::None | MipFilter::Point => wgpu::MipmapFilterMode::Nearest,
        MipFilter::Linear => wgpu::MipmapFilterMode::Linear,
    };
    // wgpu: anisotropy needs linear min, mag and mip filtering.
    let all_linear = desc.min == SamplerFilter::Linear
        && desc.mag == SamplerFilter::Linear
        && desc.mip == MipFilter::Linear;
    wgpu::SamplerDescriptor {
        label: Some("phx-sampler"),
        address_mode_u: wrap_mode(desc.wrap[0]),
        address_mode_v: wrap_mode(desc.wrap[1]),
        address_mode_w: wrap_mode(desc.wrap[2]),
        mag_filter: filter(desc.mag),
        min_filter: filter(desc.min),
        mipmap_filter: mip,
        lod_min_clamp: desc.lod_min as f32,
        lod_max_clamp: if desc.mip == MipFilter::None {
            desc.lod_min as f32
        } else if desc.lod_max == 255 {
            32.0
        } else {
            desc.lod_max as f32
        },
        compare: desc.compare.map(compare_function),
        anisotropy_clamp: if all_linear {
            desc.anisotropy.max(1) as u16
        } else {
            1
        },
        border_color: None,
    }
}

/// Vertex attributes of an indexed mesh: the interleaved `position (3)`,
/// `normal (3)`, `uv (2)`, `color (4)` the mesh was built with, at the fixed
/// locations 0 to 3.
pub(super) fn mesh_attributes(format: &VertexFormat) -> Vec<wgpu::VertexAttribute> {
    let mut attrs = Vec::with_capacity(4);
    let mut offset = 0u64;
    let mut push = |format: wgpu::VertexFormat, location: u32, size: u64| {
        attrs.push(wgpu::VertexAttribute {
            format,
            offset,
            shader_location: location,
        });
        offset += size;
    };
    if format.has_position {
        push(wgpu::VertexFormat::Float32x3, 0, 12);
    }
    if format.has_normal {
        push(wgpu::VertexFormat::Float32x3, 1, 12);
    }
    if format.has_uv {
        push(wgpu::VertexFormat::Float32x2, 2, 8);
    }
    if format.has_color {
        push(wgpu::VertexFormat::Float32x4, 3, 16);
    }
    attrs
}

/// Per-vertex attributes of the immediate batcher's 2D vertex (`Imm2DVertex`):
/// position, uv, color and the two shape parameter vectors.
pub(super) const IMM2D_ATTRIBUTES: [wgpu::VertexAttribute; 5] = [
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x2,
        offset: 0,
        shader_location: 0,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x2,
        offset: 8,
        shader_location: 2,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x4,
        offset: 16,
        shader_location: 3,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x4,
        offset: 32,
        shader_location: 11,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x4,
        offset: 48,
        shader_location: 12,
    },
];

/// `Imm3DVertex`: position, uv, color.
pub(super) const IMM3D_ATTRIBUTES: [wgpu::VertexAttribute; 3] = [
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x3,
        offset: 0,
        shader_location: 0,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x2,
        offset: 12,
        shader_location: 2,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x4,
        offset: 20,
        shader_location: 3,
    },
];

/// The built-in unit quad of `DrawFullscreen` (an `ImmVertex`: position,
/// normal, uv, color at locations 0 to 3).
pub(super) const QUAD_ATTRIBUTES: [wgpu::VertexAttribute; 4] = [
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x3,
        offset: 0,
        shader_location: 0,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x3,
        offset: 12,
        shader_location: 1,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x2,
        offset: 24,
        shader_location: 2,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x4,
        offset: 32,
        shader_location: 3,
    },
];

/// Per-instance attributes of `DrawMeshInstanced` (`InstanceData`, 84 bytes):
/// the model matrix columns at 4 to 7, color at 8, scale at 9.
pub(super) const INSTANCE_ATTRIBUTES: [wgpu::VertexAttribute; 6] = [
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x4,
        offset: 0,
        shader_location: 4,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x4,
        offset: 16,
        shader_location: 5,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x4,
        offset: 32,
        shader_location: 6,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x4,
        offset: 48,
        shader_location: 7,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x4,
        offset: 64,
        shader_location: 8,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32,
        offset: 80,
        shader_location: 9,
    },
];

/// The instance index list of `DrawInstancedIndices` (`uint`, location 10).
pub(super) const INSTANCE_INDEX_ATTRIBUTES: [wgpu::VertexAttribute; 1] = [wgpu::VertexAttribute {
    format: wgpu::VertexFormat::Uint32,
    offset: 0,
    shader_location: 10,
}];

/// Formats whose texels are floating point (not clamped by GL, not normalized).
pub(super) fn is_float_format(format: wgpu::TextureFormat) -> bool {
    use wgpu::TextureFormat as F;
    matches!(
        format,
        F::R16Float | F::Rg16Float | F::Rgba16Float | F::R32Float | F::Rg32Float | F::Rgba32Float
    )
}

/// Float32 formats need `FLOAT32_BLENDABLE` to blend.
pub(super) fn is_float32(format: wgpu::TextureFormat) -> bool {
    matches!(
        format,
        wgpu::TextureFormat::R32Float
            | wgpu::TextureFormat::Rg32Float
            | wgpu::TextureFormat::Rgba32Float
    )
}

/// Bytes per texel of the color formats the executor creates.
pub(super) fn format_bpp(format: wgpu::TextureFormat) -> u32 {
    use wgpu::TextureFormat as F;
    match format {
        F::R8Unorm => 1,
        F::Rg8Unorm | F::R16Unorm | F::R16Float => 2,
        F::Rg16Unorm | F::Rg16Float | F::R32Float => 4,
        F::Rg32Float | F::Rgba16Float | F::Rgba16Unorm => 8,
        F::Rgba32Float => 16,
        F::Rgba8Unorm | F::Rgba8UnormSrgb | F::Bgra8Unorm | F::Bgra8UnormSrgb => 4,
        _ => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blend_mode_matches_gl_semantics() {
        assert!(blend_state(BlendMode::Disabled).is_none());
        let add = blend_state(BlendMode::Additive).unwrap();
        assert_eq!(add.color.src_factor, wgpu::BlendFactor::One);
        assert_eq!(add.color.dst_factor, wgpu::BlendFactor::One);
        let alpha = blend_state(BlendMode::Alpha).unwrap();
        assert_eq!(alpha.color.src_factor, wgpu::BlendFactor::SrcAlpha);
        assert_eq!(alpha.color.dst_factor, wgpu::BlendFactor::OneMinusSrcAlpha);
        assert_eq!(alpha.alpha.src_factor, wgpu::BlendFactor::One);
        let premult = blend_state(BlendMode::PreMultAlpha).unwrap();
        assert_eq!(premult.color.src_factor, wgpu::BlendFactor::One);
    }

    #[test]
    fn cull_and_topology_map_directly() {
        assert_eq!(cull_mode(CullFace::None), None);
        assert_eq!(cull_mode(CullFace::Back), Some(wgpu::Face::Back));
        assert_eq!(cull_mode(CullFace::Front), Some(wgpu::Face::Front));
        assert_eq!(
            topology(Topology::TriangleStrip),
            wgpu::PrimitiveTopology::TriangleStrip
        );
    }

    #[test]
    fn mesh_attributes_follow_the_interleaved_layout() {
        let attrs = mesh_attributes(&VertexFormat::default());
        assert_eq!(attrs.len(), 3);
        assert_eq!((attrs[1].shader_location, attrs[1].offset), (1, 12));
        assert_eq!((attrs[2].shader_location, attrs[2].offset), (2, 24));
        let colored = mesh_attributes(&VertexFormat {
            has_color: true,
            stride: 48,
            ..VertexFormat::default()
        });
        assert_eq!((colored[3].shader_location, colored[3].offset), (3, 32));
    }

    #[test]
    fn imm_attribute_offsets_match_the_vertex_structs() {
        assert_eq!(
            IMM2D_ATTRIBUTES.last().unwrap().offset + 16,
            ImmLayout::D2.stride() as u64
        );
        assert_eq!(
            IMM3D_ATTRIBUTES.last().unwrap().offset + 16,
            ImmLayout::D3.stride() as u64
        );
        assert_eq!(
            INSTANCE_ATTRIBUTES.last().unwrap().offset + 4,
            std::mem::size_of::<InstanceData>() as u64
        );
    }
}
