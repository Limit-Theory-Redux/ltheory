//! Texture creation, upload and mip generation of the wgpu executor (render
//! API v2, S9): real 1D, 2D, 3D and cube textures with their mip chains,
//! format-agnostic updates, and `GenerateMips` as a chain of blit passes.

use std::borrow::Cow;

use super::*;
use crate::render::{TexDim, TexUsages, convert_floats, f32_to_f16, face_layer};

/// The pipelines, layouts and samplers of the mip blit, created on first use.
#[derive(Debug, Default)]
pub(super) struct MipBlit {
    /// Vertex plus fragment module, per source dimension (`false` = 2D).
    modules: HashMap<bool, wgpu::ShaderModule>,
    /// (3D, filterable) -> bind group layout.
    layouts: HashMap<(bool, bool), wgpu::BindGroupLayout>,
    pipelines: HashMap<(wgpu::TextureFormat, bool), wgpu::RenderPipeline>,
    /// Linear and nearest, clamping.
    samplers: Option<[wgpu::Sampler; 2]>,
}

/// One fullscreen triangle. For a 3D source the instance index is the slice
/// being written, and the fragment samples between the two source slices
/// that average into it.
fn blit_wgsl(is_3d: bool) -> String {
    let (tex_ty, sample) = if is_3d {
        (
            "texture_3d<f32>",
            "let depth = max(textureDimensions(src, 0).z / 2u, 1u);\n    \
             return textureSample(src, smp, vec3<f32>(in.uv, (f32(in.slice) + 0.5) / f32(depth)));",
        )
    } else {
        ("texture_2d<f32>", "return textureSample(src, smp, in.uv);")
    };
    format!(
        "struct VsOut {{\n    @builtin(position) pos: vec4<f32>,\n    @location(0) uv: vec2<f32>,\n    \
         @location(1) @interpolate(flat) slice: u32,\n}};\n\
         @vertex fn vs(@builtin(vertex_index) i: u32, @builtin(instance_index) slice: u32) -> VsOut {{\n    \
         var p = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));\n    \
         var o: VsOut;\n    o.pos = vec4<f32>(p[i], 0.0, 1.0);\n    \
         o.uv = vec2<f32>(p[i].x * 0.5 + 0.5, 0.5 - p[i].y * 0.5);\n    o.slice = slice;\n    return o;\n}}\n\
         @group(0) @binding(0) var src: {tex_ty};\n@group(0) @binding(1) var smp: sampler;\n\
         @fragment fn fs(in: VsOut) -> @location(0) vec4<f32> {{\n    {sample}\n}}\n"
    )
}

fn is_filterable(format: wgpu::TextureFormat, features: wgpu::Features) -> bool {
    !is_float32(format) || features.contains(wgpu::Features::FLOAT32_FILTERABLE)
}

impl MipBlit {
    fn samplers(&mut self, device: &wgpu::Device) -> &[wgpu::Sampler; 2] {
        self.samplers.get_or_insert_with(|| {
            let make = |filter: wgpu::FilterMode| {
                device.create_sampler(&wgpu::SamplerDescriptor {
                    label: Some("phx-mip-blit-sampler"),
                    mag_filter: filter,
                    min_filter: filter,
                    ..Default::default()
                })
            };
            [
                make(wgpu::FilterMode::Linear),
                make(wgpu::FilterMode::Nearest),
            ]
        })
    }

    fn layout(
        &mut self,
        device: &wgpu::Device,
        is_3d: bool,
        filterable: bool,
    ) -> &wgpu::BindGroupLayout {
        self.layouts.entry((is_3d, filterable)).or_insert_with(|| {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("phx-mip-blit-bgl"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable },
                            view_dimension: if is_3d {
                                wgpu::TextureViewDimension::D3
                            } else {
                                wgpu::TextureViewDimension::D2
                            },
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(if filterable {
                            wgpu::SamplerBindingType::Filtering
                        } else {
                            wgpu::SamplerBindingType::NonFiltering
                        }),
                        count: None,
                    },
                ],
            })
        })
    }

    /// Make sure the pipeline for `format` exists.
    fn ensure_pipeline(
        &mut self,
        device: &wgpu::Device,
        features: wgpu::Features,
        format: wgpu::TextureFormat,
        is_3d: bool,
    ) {
        if self.pipelines.contains_key(&(format, is_3d)) {
            return;
        }
        let filterable = is_filterable(format, features);
        self.layout(device, is_3d, filterable);
        let module = self.modules.entry(is_3d).or_insert_with(|| {
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("phx-mip-blit"),
                source: wgpu::ShaderSource::Wgsl(Cow::Owned(blit_wgsl(is_3d))),
            })
        });
        let bgl = &self.layouts[&(is_3d, filterable)];
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("phx-mip-blit-pl"),
            bind_group_layouts: &[Some(bgl)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("phx-mip-blit"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        self.pipelines.insert((format, is_3d), pipeline);
    }
}

impl WgpuCommandExecutor {
    /// The texture and description of any texture-kind resource.
    pub(super) fn texture_and_desc(&self, id: ResourceId) -> Option<(wgpu::Texture, TexDesc)> {
        match self.resources.get(&id)? {
            WgpuResource::Texture { texture, desc } => Some((texture.clone(), *desc)),
            _ => None,
        }
    }

    fn texture_usages(&self, desc: &TexDesc) -> wgpu::TextureUsages {
        let mut usage = self.diag.texture_usage();
        if desc.usage.contains(TexUsages::SAMPLED) {
            usage |= wgpu::TextureUsages::TEXTURE_BINDING;
        }
        if desc.usage.contains(TexUsages::COPY_SRC) {
            usage |= wgpu::TextureUsages::COPY_SRC;
        }
        if desc.usage.contains(TexUsages::COPY_DST) {
            usage |= wgpu::TextureUsages::COPY_DST;
        }
        // The mip blit renders into the levels and samples the previous one.
        let attachment = desc.usage.contains(TexUsages::ATTACHMENT) || desc.mips > 1;
        if attachment && desc.dim != TexDim::D1 && !TexFormat::is_depth(desc.format) {
            usage |= wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        } else if attachment && TexFormat::is_depth(desc.format) {
            usage |= wgpu::TextureUsages::RENDER_ATTACHMENT;
        }
        usage
    }

    /// Create a texture of `desc.dim`: a real `D1`, `D2` or `D3` texture, or a
    /// six-layer 2D texture that is viewed as a cube. `data` (level 0, in the
    /// texture's own format) is uploaded right away.
    pub(super) fn cmd_create_texture(
        &mut self,
        id: ResourceId,
        desc: &TexDesc,
        data: Option<Vec<u8>>,
    ) {
        let (wgpu_format, _) = self.storage_format(desc.format);
        let (dimension, layers) = match desc.dim {
            TexDim::D1 => (wgpu::TextureDimension::D1, 1),
            TexDim::D2 => (wgpu::TextureDimension::D2, 1),
            TexDim::D3 => (wgpu::TextureDimension::D3, desc.size[2]),
            TexDim::Cube => (wgpu::TextureDimension::D2, 6),
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(match desc.dim {
                TexDim::D1 => "phx-tex1d",
                TexDim::D2 => "phx-tex2d",
                TexDim::D3 => "phx-tex3d",
                TexDim::Cube => "phx-texcube",
            }),
            size: wgpu::Extent3d {
                width: desc.size[0],
                height: if desc.dim == TexDim::D1 {
                    1
                } else {
                    desc.size[1]
                },
                depth_or_array_layers: layers,
            },
            // WebGPU: a 1D texture has exactly one level.
            mip_level_count: if desc.dim == TexDim::D1 {
                1
            } else {
                desc.mips.max(1)
            },
            sample_count: 1,
            dimension,
            format: wgpu_format,
            usage: self.texture_usages(desc),
            view_formats: &[],
        });
        self.resources.insert(
            id,
            WgpuResource::Texture {
                texture,
                desc: *desc,
            },
        );
        // A texture created again under an id must not show up through the
        // views and bind groups made for the old one.
        self.bump_generation(id);
        if let Some(bytes) = data {
            self.cmd_update_texture(id, &TexRegion::level(desc, 0), bytes);
        }
    }

    /// Write `data`, tightly packed texels in the texture's own `TexFormat`
    /// layout, to `region`. Formats the adapter cannot hold natively are
    /// converted here (`R32F` and `RGBA32F` in 16F textures without
    /// `FLOAT32_FILTERABLE`).
    pub(super) fn cmd_update_texture(&mut self, id: ResourceId, region: &TexRegion, data: Vec<u8>) {
        let Some((texture, desc)) = self.texture_and_desc(id) else {
            warn!("wgpu: update of unknown texture {id:?}");
            return;
        };
        if region.level >= texture.mip_level_count() {
            warn!(
                "wgpu: update of level {} of {id:?}, which has {} (create it with mips)",
                region.level,
                texture.mip_level_count()
            );
            return;
        }
        let storage: Cow<[u8]> = match (desc.format, self.f32_in_f16(desc.format)) {
            (TexFormat::R32F, true) => Cow::Owned(f32_to_rgba16f_bytes(&data)),
            (TexFormat::RGBA32F, true) => Cow::Owned(f32_to_f16_bytes(&data)),
            _ => Cow::Borrowed(&data[..]),
        };
        let bpp = format_bpp(texture.format());
        let [w, h, d] = region.size;
        if w == 0 || h == 0 || d == 0 {
            // GL tolerates empty uploads; wgpu rejects them.
            return;
        }
        let expected = region.texels() * bpp as usize;
        if storage.len() < expected {
            warn!(
                "wgpu: update of {id:?} needs {expected} bytes for {region:?}, got {}",
                storage.len()
            );
            return;
        }
        self.flush_for_write();
        // `write_texture` repacks rows itself: no 256-byte alignment needed.
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: region.level,
                origin: wgpu::Origin3d {
                    x: region.origin[0],
                    y: region.origin[1],
                    z: region.origin[2],
                },
                aspect: wgpu::TextureAspect::All,
            },
            &storage[..expected],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w * bpp),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: d,
            },
        );
    }

    /// `SetTexel1D/2DByResource`: one texel of level 0, as floats.
    pub(super) fn cmd_set_texel(&mut self, id: ResourceId, x: i32, y: i32, color: [f32; 4]) {
        let Some((texture, desc)) = self.texture_and_desc(id) else {
            warn!("wgpu: texel write for unknown texture {id:?}");
            return;
        };
        if x < 0 || y < 0 || x as u32 >= desc.size[0] || y as u32 >= desc.size[1].max(1) {
            warn!("wgpu: texel write out of bounds for {id:?}: ({x}, {y})");
            return;
        }
        let texel = convert_floats(crate::render::PixelFormat::RGBA, &color, desc.format);
        let texel: Cow<[u8]> = match (desc.format, self.f32_in_f16(desc.format)) {
            (TexFormat::R32F, true) => Cow::Owned(f32_to_rgba16f_bytes(&texel)),
            (TexFormat::RGBA32F, true) => Cow::Owned(f32_to_f16_bytes(&texel)),
            _ => Cow::Owned(texel),
        };
        self.flush_for_write();
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: x as u32,
                    y: y as u32,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            &texel,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: None,
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Replace the single-level texture `id` by one with a full mip chain (same
    /// contents in level 0). `None` if the texture cannot be copied or
    /// rendered to.
    fn grow_mip_chain(
        &mut self,
        id: ResourceId,
        old: &wgpu::Texture,
        desc: &TexDesc,
    ) -> Option<(wgpu::Texture, TexDesc)> {
        let needed = TexUsages::COPY_SRC | TexUsages::COPY_DST;
        if desc.usage.0 & needed != needed {
            warn!(
                "wgpu: GenerateMips of {id:?}: it was created without copy usage, so it cannot get a mip chain"
            );
            return None;
        }
        let grown_desc = desc.with_mips(0);
        if grown_desc.mips <= 1 {
            return None;
        }
        let (format, _) = self.storage_format(desc.format);
        let layers = match desc.dim {
            TexDim::D3 => desc.size[2],
            TexDim::Cube => 6,
            _ => 1,
        };
        let grown = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("phx-texture-grown"),
            size: old.size(),
            mip_level_count: grown_desc.mips,
            sample_count: 1,
            dimension: old.dimension(),
            format,
            usage: self.texture_usages(&grown_desc),
            view_formats: &[],
        });
        let _ = layers;
        self.suspend_pass();
        let encoder = self.encoder();
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: old,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &grown,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            old.size(),
        );
        self.recorded = true;
        self.resources.insert(
            id,
            WgpuResource::Texture {
                texture: grown.clone(),
                desc: grown_desc,
            },
        );
        // Views and bind groups made for the single-level texture are stale.
        self.bump_generation(id);
        Some((grown, grown_desc))
    }

    /// Fill levels 1.. of a texture from level 0, one pass per level and face
    /// (or 3D slice), each sampling the level above with a bilinear (box)
    /// filter. Recorded into the frame's encoder, between the passes around
    /// it. A texture without a chain (`mips` of 1 at creation) has nothing
    /// to fill; create it with `TexDesc::with_mips`.
    pub(super) fn cmd_generate_mips(&mut self, id: ResourceId) {
        let Some((mut texture, mut desc)) = self.texture_and_desc(id) else {
            warn!("wgpu: GenerateMips of unknown texture {id:?}");
            return;
        };
        if desc.dim == TexDim::D1 || TexFormat::is_depth(desc.format) {
            // WebGPU 1D textures have one level, depth textures get no chain.
            return;
        }
        if texture.mip_level_count() <= 1 {
            // GL allocates the chain when it generates it; a wgpu texture has
            // its levels from creation. Make the chain now: a new texture of
            // the same id with all levels and the old base level copied in.
            let Some((grown, grown_desc)) = self.grow_mip_chain(id, &texture, &desc) else {
                return;
            };
            texture = grown;
            desc = grown_desc;
        }
        let levels = texture.mip_level_count();
        if levels <= 1 {
            return;
        }
        let format = texture.format();
        let is_3d = desc.dim == TexDim::D3;
        let filterable = is_filterable(format, self.features);
        let device = self.device.clone();
        self.mip_blit
            .ensure_pipeline(&device, self.features, format, is_3d);
        self.mip_blit.samplers(&device);
        self.suspend_pass();
        let mut encoder = self.encoder.take().unwrap_or_else(|| {
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("phx-encoder"),
            })
        });

        let blit = &self.mip_blit;
        let pipeline = &blit.pipelines[&(format, is_3d)];
        let bgl = &blit.layouts[&(is_3d, filterable)];
        let sampler =
            &blit.samplers.as_ref().expect("created above")[if filterable { 0 } else { 1 }];
        let view_of = |level: u32, layer: u32| {
            texture.create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(if is_3d {
                    wgpu::TextureViewDimension::D3
                } else {
                    wgpu::TextureViewDimension::D2
                }),
                base_mip_level: level,
                mip_level_count: Some(1),
                base_array_layer: if is_3d { 0 } else { layer },
                array_layer_count: if is_3d { None } else { Some(1) },
                ..Default::default()
            })
        };
        let layers = if desc.dim == TexDim::Cube { 6 } else { 1 };
        for level in 1..levels {
            let slices = if is_3d {
                desc.level_size(level)[2]
            } else {
                layers
            };
            for slice in 0..slices {
                let src = view_of(level - 1, slice);
                let dst = view_of(level, slice);
                let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("phx-mip-blit-bg"),
                    layout: bgl,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&src),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(sampler),
                        },
                    ],
                });
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("phx-mip-blit"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &dst,
                        depth_slice: is_3d.then_some(slice),
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    ..Default::default()
                });
                pass.set_pipeline(pipeline);
                pass.set_bind_group(0, &group, &[]);
                pass.draw(0..3, if is_3d { slice..slice + 1 } else { 0..1 });
            }
        }
        self.encoder = Some(encoder);
        self.recorded = true;
    }

    /// The texture, mip level and array layer a 2D or cube-face view addresses.
    fn copy_target(&self, view: &TexView) -> Option<(wgpu::Texture, u32, u32)> {
        let (texture, desc) = self.texture_and_desc(view.tex)?;
        match (desc.dim, view.dim) {
            (TexDim::D2, ViewDim::D2) => Some((texture, view.base_mip as u32, 0)),
            (TexDim::Cube, ViewDim::CubeFace(face)) => {
                Some((texture, view.base_mip as u32, face_layer(face)))
            }
            _ => None,
        }
    }

    /// `CopyTexture` between two 2D or cube-face views, recorded into the
    /// frame's encoder (mip levels a texture was not created with are skipped).
    pub(super) fn cmd_copy_texture(&mut self, src: &TexView, dst: &TexView, size: [u32; 3]) {
        let (Some((src_tex, src_mip, src_layer)), Some((dst_tex, dst_mip, dst_layer))) =
            (self.copy_target(src), self.copy_target(dst))
        else {
            warn!("wgpu: CopyTexture supports 2D and cube-face views only (no-op)");
            return;
        };
        if src_mip >= src_tex.mip_level_count() || dst_mip >= dst_tex.mip_level_count() {
            return;
        }
        self.suspend_pass();
        let encoder = self.encoder();
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &src_tex,
                mip_level: src_mip,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: src_layer,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &dst_tex,
                mip_level: dst_mip,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: dst_layer,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: 1,
            },
        );
        self.recorded = true;
    }
}

/// 32-bit float payload to the 16-bit floats a 16F texture stores.
fn f32_to_f16_bytes(data: &[u8]) -> Vec<u8> {
    data.chunks_exact(4)
        .flat_map(|c| f32_to_f16(f32::from_le_bytes([c[0], c[1], c[2], c[3]])).to_le_bytes())
        .collect()
}

/// An `R32F` upload as RGBA16F texels `(r, 0, 0, 1)`.
fn f32_to_rgba16f_bytes(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() * 2);
    for c in data.chunks_exact(4) {
        out.extend_from_slice(
            &f32_to_f16(f32::from_le_bytes([c[0], c[1], c[2], c[3]])).to_le_bytes(),
        );
        out.extend_from_slice(&[0, 0, 0, 0, 0, 0x3c]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blit_shaders_declare_the_matching_texture_type() {
        assert!(blit_wgsl(false).contains("texture_2d<f32>"));
        assert!(blit_wgsl(true).contains("texture_3d<f32>"));
        assert!(blit_wgsl(true).contains("instance_index"));
    }

    #[test]
    fn blit_shaders_are_valid_wgsl() {
        for is_3d in [false, true] {
            let module = wgpu::naga::front::wgsl::parse_str(&blit_wgsl(is_3d)).expect("parse");
            wgpu::naga::valid::Validator::new(
                wgpu::naga::valid::ValidationFlags::all(),
                wgpu::naga::valid::Capabilities::all(),
            )
            .validate(&module)
            .expect("validate");
        }
    }

    #[test]
    fn only_32f_formats_need_a_feature_to_filter() {
        let none = wgpu::Features::empty();
        assert!(is_filterable(wgpu::TextureFormat::Rgba16Float, none));
        assert!(is_filterable(wgpu::TextureFormat::Rgba8Unorm, none));
        assert!(!is_filterable(wgpu::TextureFormat::Rg32Float, none));
        assert!(is_filterable(
            wgpu::TextureFormat::Rg32Float,
            wgpu::Features::FLOAT32_FILTERABLE
        ));
    }

    #[test]
    fn r32f_upload_expands_to_rgba16f() {
        let bytes = f32_to_rgba16f_bytes(&1.0f32.to_le_bytes());
        assert_eq!(bytes.len(), 8);
        assert_eq!(&bytes[..2], &f32_to_f16(1.0).to_le_bytes());
        assert_eq!(&bytes[6..], &[0, 0x3c]);
    }
}
