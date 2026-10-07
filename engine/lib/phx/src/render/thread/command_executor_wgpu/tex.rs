//! Texture creation, upload and mip generation of the wgpu executor (render
//! API v2, S9): real 1D, 2D, 3D and cube textures with their mip chains,
//! format-agnostic updates, and `GenerateMips` as a chain of blit passes.

use std::borrow::Cow;

use super::*;
use crate::render::{TexDim, TexUsages};

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

fn is_filterable(format: wgpu::TextureFormat) -> bool {
    !matches!(
        format,
        wgpu::TextureFormat::R32Float
            | wgpu::TextureFormat::Rg32Float
            | wgpu::TextureFormat::Rgba32Float
    )
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
    fn ensure_pipeline(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat, is_3d: bool) {
        if self.pipelines.contains_key(&(format, is_3d)) {
            return;
        }
        let filterable = is_filterable(format);
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
    fn texture_and_desc(&self, id: ResourceId) -> Option<(wgpu::Texture, TexDesc)> {
        match self.resources.get(&id)? {
            WgpuGpuResource::Texture1D { texture, desc, .. }
            | WgpuGpuResource::Texture2D { texture, desc, .. }
            | WgpuGpuResource::Texture3D { texture, desc, .. }
            | WgpuGpuResource::TextureCube { texture, desc, .. } => Some((texture.clone(), *desc)),
            _ => None,
        }
    }

    fn texture_usages(desc: &TexDesc) -> wgpu::TextureUsages {
        let mut usage = wgpu::TextureUsages::empty();
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
    /// six-layer 2D texture with a cube view. `data` (level 0, in the
    /// texture's own format) is uploaded right away.
    pub(super) fn cmd_create_texture(
        &mut self,
        id: ResourceId,
        desc: &TexDesc,
        data: Option<Vec<u8>>,
    ) {
        let Some(device) = self.device.clone() else {
            return;
        };
        let (wgpu_format, _bpp, _converted) = Self::tex_format_to_wgpu_with_bpp(desc.format);
        let (dimension, view_dimension, layers) = match desc.dim {
            TexDim::D1 => (
                wgpu::TextureDimension::D1,
                wgpu::TextureViewDimension::D1,
                1,
            ),
            TexDim::D2 => (
                wgpu::TextureDimension::D2,
                wgpu::TextureViewDimension::D2,
                1,
            ),
            TexDim::D3 => (
                wgpu::TextureDimension::D3,
                wgpu::TextureViewDimension::D3,
                desc.size[2],
            ),
            TexDim::Cube => (
                wgpu::TextureDimension::D2,
                wgpu::TextureViewDimension::Cube,
                6,
            ),
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
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
            usage: Self::texture_usages(desc),
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(view_dimension),
            ..Default::default()
        });
        // 32F formats are NOT filterable in WebGPU: a Linear sampler for them
        // fails bind-group validation (the shaders texelFetch them anyway).
        let filter = if is_filterable(wgpu_format) {
            wgpu::FilterMode::Linear
        } else {
            wgpu::FilterMode::Nearest
        };
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("phx-sampler"),
            mag_filter: filter,
            min_filter: filter,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });
        let desc_copy = *desc;
        let resource = match desc.dim {
            TexDim::D1 => WgpuGpuResource::Texture1D {
                texture,
                view,
                sampler,
                desc: desc_copy,
            },
            TexDim::D2 => WgpuGpuResource::Texture2D {
                texture,
                view,
                sampler,
                desc: desc_copy,
            },
            TexDim::D3 => WgpuGpuResource::Texture3D {
                texture,
                view,
                sampler,
                desc: desc_copy,
            },
            TexDim::Cube => WgpuGpuResource::TextureCube {
                texture,
                view,
                sampler,
                desc: desc_copy,
            },
        };
        self.resources.insert(id, resource);
        // A texture created again under an id must not show up through the
        // pipelines and bind groups made for the old one.
        self.bump_generation(id);
        if let Some(bytes) = data {
            self.cmd_update_texture(id, &TexRegion::level(desc, 0), bytes);
        }
    }

    /// Write `data`, tightly packed texels in the texture's own `TexFormat`
    /// layout, to `region`. Formats the wgpu backend stores differently are
    /// converted here (`R32F` and `RGBA32F` live in filterable 16F textures).
    pub(super) fn cmd_update_texture(&mut self, id: ResourceId, region: &TexRegion, data: Vec<u8>) {
        let Some(queue) = self.queue.clone() else {
            return;
        };
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
        let storage: Cow<[u8]> = match desc.format {
            TexFormat::R32F => Cow::Owned(Self::f32_to_rgba16f_bytes(&data)),
            TexFormat::RGBA32F => Cow::Owned(Self::f32_to_f16_bytes(&data)),
            _ => Cow::Borrowed(&data[..]),
        };
        let bpp = Self::format_bpp(texture.format());
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
        // `write_texture` repacks rows itself: no 256-byte alignment needed.
        queue.write_texture(
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

    /// Fill levels 1.. of a texture from level 0, one pass per level and face
    /// (or 3D slice), each sampling the level above with a bilinear (box)
    /// filter. A texture without a chain (`mips` of 1 at creation) has nothing
    /// to fill; create it with `TexDesc::with_mips`.
    pub(super) fn cmd_generate_mips(&mut self, id: ResourceId) {
        let (Some(device), Some(queue)) = (self.device.clone(), self.queue.clone()) else {
            return;
        };
        let Some((texture, desc)) = self.texture_and_desc(id) else {
            warn!("wgpu: GenerateMips of unknown texture {id:?}");
            return;
        };
        if desc.dim == TexDim::D1 || TexFormat::is_depth(desc.format) {
            // WebGPU 1D textures have one level, depth textures get no chain.
            return;
        }
        let levels = texture.mip_level_count();
        if levels <= 1 {
            warn!("wgpu: GenerateMips of {id:?}, which was created without mip levels (no-op)");
            return;
        }
        let format = texture.format();
        let is_3d = desc.dim == TexDim::D3;
        let filterable = is_filterable(format);
        self.mip_blit.ensure_pipeline(&device, format, is_3d);
        let sampler_index = if filterable { 0 } else { 1 };
        self.mip_blit.samplers(&device);
        let blit = &self.mip_blit;
        let pipeline = &blit.pipelines[&(format, is_3d)];
        let bgl = &blit.layouts[&(is_3d, filterable)];
        let sampler = &blit.samplers.as_ref().expect("created above")[sampler_index];

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
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("phx-generate-mips"),
        });
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
        queue.submit([encoder.finish()]);
    }
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
    fn only_32f_formats_are_unfilterable() {
        assert!(is_filterable(wgpu::TextureFormat::Rgba16Float));
        assert!(is_filterable(wgpu::TextureFormat::Rgba8Unorm));
        assert!(!is_filterable(wgpu::TextureFormat::Rg32Float));
    }

    #[test]
    fn mips_force_an_attachment_usage() {
        let plain = TexDesc::d2(8, 8, TexFormat::RGBA8).with_usage(TexUsages(TexUsages::SAMPLED));
        assert!(
            !WgpuCommandExecutor::texture_usages(&plain)
                .contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
        );
        let mipped = plain.with_mips(0);
        assert!(
            WgpuCommandExecutor::texture_usages(&mipped)
                .contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
        );
    }
}
