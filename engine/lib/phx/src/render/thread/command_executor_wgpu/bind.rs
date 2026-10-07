//! Bind groups of the wgpu executor: group layouts interned from what a shader
//! declares, the per-pass state of the four groups, and bind groups created
//! from that state and cached by content.
//!
//! A group's wgpu layout holds the blocks and samplers the shader declares
//! under that `#group` (bindings per `gpu/layout.rs`). The first block of
//! groups 0 and 2 is bound from a ring buffer with a dynamic offset (`SetView`
//! and `SetDraw`); every other block comes from a material arena slice. Two
//! shaders that declare the same things in a group share one layout, so a
//! bind group made for one stays valid for the other.

use super::*;
use crate::render::{TexDim, wgpu_block_binding, wgpu_sampler_binding, wgpu_texture_binding};

/// A uniform block of a group layout.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct BlockSlot {
    pub index: u8,
    pub size: u32,
    pub dynamic: bool,
}

/// A sampler (texture plus sampler object) of a group layout.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct TexSlot {
    pub index: u8,
    pub dim: TexDim,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct GroupLayoutKey {
    blocks: Vec<BlockSlot>,
    textures: Vec<TexSlot>,
}

pub(super) struct GroupLayout {
    pub id: u32,
    pub bgl: wgpu::BindGroupLayout,
    pub blocks: Vec<BlockSlot>,
    pub textures: Vec<TexSlot>,
}

impl GroupLayout {
    pub fn has_dynamic(&self) -> bool {
        self.blocks.iter().any(|b| b.dynamic)
    }
}

/// The first block of the frame (0) and draw (2) groups comes from the
/// uniform ring.
pub(super) fn is_dynamic_block(group: u8, index: u8) -> bool {
    (group == 0 || group == 2) && index == 0
}

/// A created bind group and a serial that identifies it.
#[derive(Clone)]
pub(super) struct BoundBg {
    pub serial: u64,
    pub bg: wgpu::BindGroup,
}

pub(super) struct CachedBg {
    pub bound: BoundBg,
    pub last_used: u64,
}

fn dim_code(dim: TexDim) -> u64 {
    match dim {
        TexDim::D1 => 1,
        TexDim::D2 => 2,
        TexDim::D3 => 3,
        TexDim::Cube => 4,
    }
}

/// What the commands of the open pass have bound to one group so far.
#[derive(Default)]
pub(super) struct GroupState {
    /// Dynamic block: ring `(slot, chunk)` and the offset into it.
    pub ring: Option<(u8, u16)>,
    pub dyn_offset: u32,
    /// Blocks from material arenas (`BindEntry::Uniform`) by block index.
    pub statics: [Option<(BufferId, u32, u32)>; 4],
    /// Samplers by index within the group.
    pub textures: [Option<(TexView, SamplerId)>; 8],
    /// The bind group built from the above for `bound_layout`.
    pub bound: Option<BoundBg>,
    pub bound_layout: u32,
    pub dirty: bool,
}

impl GroupState {
    pub fn new() -> Self {
        Self {
            dirty: true,
            ..Default::default()
        }
    }

    pub fn set_ring(&mut self, slot: u8, chunk: u16, offset: u32) {
        if self.ring != Some((slot, chunk)) {
            self.ring = Some((slot, chunk));
            self.dirty = true;
        }
        self.dyn_offset = offset;
    }

    /// `SetBindGroup`: the group's entries replace what was bound before.
    pub fn set_entries(&mut self, entries: &[BindEntry]) {
        self.statics = Default::default();
        self.textures = Default::default();
        for entry in entries {
            match entry {
                BindEntry::Uniform {
                    index,
                    buffer,
                    offset,
                    size,
                } => {
                    if let Some(slot) = self.statics.get_mut(*index as usize) {
                        *slot = Some((*buffer, *offset, *size));
                    }
                }
                BindEntry::Texture {
                    index,
                    view,
                    sampler,
                } => {
                    if let Some(slot) = self.textures.get_mut(*index as usize) {
                        *slot = Some((*view, *sampler));
                    }
                }
            }
        }
        self.dirty = true;
    }

    pub fn set_texture(&mut self, index: usize, value: Option<(TexView, SamplerId)>) {
        if let Some(slot) = self.textures.get_mut(index) {
            if *slot != value {
                *slot = value;
                self.dirty = true;
            }
        }
    }
}

/// 1x1 textures and a sampler for what a shader declares but nothing bound.
pub(super) struct Defaults {
    pub d1: wgpu::TextureView,
    pub d2: wgpu::TextureView,
    pub d3: wgpu::TextureView,
    pub cube: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    /// A block of zeros for uniform bindings nothing filled.
    pub zero: wgpu::Buffer,
    /// 32 zero bytes the constant vertex attributes read (stride 0).
    pub vertex_zero: wgpu::Buffer,
}

impl WgpuCommandExecutor {
    /// The group layouts of a shader: what it declares per `#group`.
    pub(super) fn shader_group_layouts(
        &mut self,
        layout: &ShaderLayout,
        blocks: &[BlockLayout],
    ) -> [Arc<GroupLayout>; 4] {
        std::array::from_fn(|g| {
            let g = g as u8;
            let mut key = GroupLayoutKey {
                blocks: Vec::new(),
                textures: Vec::new(),
            };
            for decl in layout.blocks.iter().filter(|b| b.group == g) {
                // A block naga dropped (declared, never used) has no binding.
                if let Some(reflected) = blocks.iter().find(|b| b.name == decl.name) {
                    key.blocks.push(BlockSlot {
                        index: decl.index,
                        size: reflected.size,
                        dynamic: is_dynamic_block(g, decl.index),
                    });
                }
            }
            for decl in layout.textures.iter().filter(|t| t.group == g) {
                key.textures.push(TexSlot {
                    index: decl.index,
                    dim: decl.dim,
                });
            }
            key.blocks.sort_by_key(|b| b.index);
            key.textures.sort_by_key(|t| t.index);
            self.group_layout(key)
        })
    }

    fn group_layout(&mut self, key: GroupLayoutKey) -> Arc<GroupLayout> {
        if let Some(layout) = self.group_layouts.get(&key) {
            return layout.clone();
        }
        let visibility = wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT;
        let mut entries = Vec::new();
        for block in &key.blocks {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: wgpu_block_binding(block.index),
                visibility,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: block.dynamic,
                    min_binding_size: wgpu::BufferSize::new(block.size as u64),
                },
                count: None,
            });
        }
        for tex in &key.textures {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: wgpu_texture_binding(tex.index),
                visibility,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: shader::view_dimension(tex.dim),
                    multisampled: false,
                },
                count: None,
            });
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: wgpu_sampler_binding(tex.index),
                visibility,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            });
        }
        let bgl = self
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("phx-group-layout"),
                entries: &entries,
            });
        let layout = Arc::new(GroupLayout {
            id: self.next_layout_id,
            bgl,
            blocks: key.blocks.clone(),
            textures: key.textures.clone(),
        });
        self.next_layout_id += 1;
        self.group_layouts.insert(key, layout.clone());
        layout
    }

    pub(super) fn defaults(&mut self) -> &Defaults {
        if self.defaults.is_none() {
            let make = |dimension: wgpu::TextureDimension,
                        view_dimension: wgpu::TextureViewDimension,
                        layers: u32| {
                let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("phx-default-texture"),
                    size: wgpu::Extent3d {
                        width: 1,
                        height: 1,
                        depth_or_array_layers: layers,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });
                let white = vec![255u8; 4 * layers as usize];
                self.queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &white,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(4),
                        rows_per_image: Some(1),
                    },
                    wgpu::Extent3d {
                        width: 1,
                        height: 1,
                        depth_or_array_layers: layers,
                    },
                );
                texture.create_view(&wgpu::TextureViewDescriptor {
                    dimension: Some(view_dimension),
                    ..Default::default()
                })
            };
            let defaults = Defaults {
                d1: make(
                    wgpu::TextureDimension::D1,
                    wgpu::TextureViewDimension::D1,
                    1,
                ),
                d2: make(
                    wgpu::TextureDimension::D2,
                    wgpu::TextureViewDimension::D2,
                    1,
                ),
                d3: make(
                    wgpu::TextureDimension::D3,
                    wgpu::TextureViewDimension::D3,
                    1,
                ),
                cube: make(
                    wgpu::TextureDimension::D2,
                    wgpu::TextureViewDimension::Cube,
                    6,
                ),
                sampler: self.device.create_sampler(&wgpu::SamplerDescriptor {
                    label: Some("phx-default-sampler"),
                    mag_filter: wgpu::FilterMode::Linear,
                    min_filter: wgpu::FilterMode::Linear,
                    ..Default::default()
                }),
                zero: self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("phx-zero-block"),
                    size: 64 * 1024,
                    usage: wgpu::BufferUsages::UNIFORM,
                    mapped_at_creation: false,
                }),
                vertex_zero: self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("phx-zero-attributes"),
                    size: 32,
                    usage: wgpu::BufferUsages::VERTEX,
                    mapped_at_creation: false,
                }),
            };
            self.defaults = Some(defaults);
        }
        self.defaults.as_ref().expect("created above")
    }

    /// The bind group of group `g` for the state `gs` under `layout`:
    /// created on a cache miss, shared with every state that has the same
    /// content.
    pub(super) fn resolve_group(
        &mut self,
        g: u8,
        gs: &GroupState,
        layout: &Arc<GroupLayout>,
    ) -> BoundBg {
        enum Res {
            Buffer(wgpu::Buffer, u64, u64),
            View(wgpu::TextureView),
            Sampler(wgpu::Sampler),
        }
        let mut key: Vec<u64> = Vec::with_capacity(4 + layout.blocks.len() * 4 + layout.textures.len() * 8);
        key.push(layout.id as u64);
        // (binding, resource) in layout order.
        let mut resources: Vec<(u32, Res)> = Vec::new();
        self.defaults();

        for block in &layout.blocks {
            let binding = wgpu_block_binding(block.index);
            if block.dynamic {
                match gs
                    .ring
                    .and_then(|(slot, chunk)| self.ring_uniform[slot as usize].get(chunk as usize).map(|b| ((slot, chunk), b)))
                {
                    Some(((slot, chunk), buffer)) => {
                        key.extend([1, slot as u64, chunk as u64, block.size as u64]);
                        resources.push((
                            binding,
                            Res::Buffer(buffer.clone(), 0, block.size as u64),
                        ));
                        continue;
                    }
                    None => {}
                }
            } else if let Some((buffer_id, offset, size)) = gs.statics[block.index as usize] {
                if let Some(buffer) = self.buffers.get(&buffer_id) {
                    let size = (size.max(block.size)) as u64;
                    key.extend([2, buffer_id.0 as u64, offset as u64, size]);
                    resources.push((binding, Res::Buffer(buffer.clone(), offset as u64, size)));
                    continue;
                }
            }
            // Nothing was bound: a block of zeros (the GL default is
            // whatever was bound last; zeros are deterministic).
            key.extend([3, block.size as u64]);
            let zero = self.defaults.as_ref().expect("defaults").zero.clone();
            resources.push((binding, Res::Buffer(zero, 0, block.size as u64)));
        }

        for tex in &layout.textures {
            let tex_binding = wgpu_texture_binding(tex.index);
            let bound = gs.textures[tex.index as usize];
            let mut view = None;
            let mut sampler = None;
            if let Some((tv, sampler_id)) = bound {
                if let Some((v, view_key)) = self.sampling_view(&tv, tex.dim) {
                    key.extend(view_key);
                    view = Some(v);
                    sampler = self.samplers.get(&sampler_id).cloned();
                    key.push(sampler_id.0 as u64);
                }
            }
            let (view, sampler) = match (view, sampler) {
                (Some(v), Some(s)) => (v, s),
                _ => {
                    let defaults = self.defaults.as_ref().expect("defaults");
                    key.extend([5, dim_code(tex.dim)]);
                    let v = match tex.dim {
                        TexDim::D1 => defaults.d1.clone(),
                        TexDim::D2 => defaults.d2.clone(),
                        TexDim::D3 => defaults.d3.clone(),
                        TexDim::Cube => defaults.cube.clone(),
                    };
                    (v, defaults.sampler.clone())
                }
            };
            resources.push((tex_binding, Res::View(view)));
            resources.push((wgpu_sampler_binding(tex.index), Res::Sampler(sampler)));
        }

        let frame = self.frame_index;
        if let Some(hit) = self.bg_cache.get_mut(&key) {
            hit.last_used = frame;
            return hit.bound.clone();
        }
        let entries: Vec<wgpu::BindGroupEntry> = resources
            .iter()
            .map(|(binding, res)| wgpu::BindGroupEntry {
                binding: *binding,
                resource: match res {
                    Res::Buffer(buffer, offset, size) => {
                        wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer,
                            offset: *offset,
                            size: wgpu::BufferSize::new(*size),
                        })
                    }
                    Res::View(view) => wgpu::BindingResource::TextureView(view),
                    Res::Sampler(sampler) => wgpu::BindingResource::Sampler(sampler),
                },
            })
            .collect();
        let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(match g {
                0 => "phx-group0",
                1 => "phx-group1",
                2 => "phx-group2",
                _ => "phx-group3",
            }),
            layout: &layout.bgl,
            entries: &entries,
        });
        let bound = BoundBg {
            serial: self.next_bg_serial,
            bg,
        };
        self.next_bg_serial += 1;
        self.bg_cache.insert(
            key,
            CachedBg {
                bound: bound.clone(),
                last_used: frame,
            },
        );
        bound
    }

    /// Drop bind groups nothing has used for a few frames (they hold their
    /// textures alive).
    pub(super) fn sweep_bind_groups(&mut self) {
        let frame = self.frame_index;
        self.bg_cache.retain(|_, c| c.last_used + 4 >= frame);
    }
}
