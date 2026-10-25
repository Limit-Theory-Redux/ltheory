//! The Rust half of a material (doc/engine/render-api-v2.md, section 3a).
//!
//! A `Material` owns what the GPU needs to draw with one set of material
//! parameters: a slice of a parameter arena holding the `MaterialParams`
//! block, the group-1 bind group (that slice plus the material's textures and
//! samplers) and the pipeline template (shader and fixed-function state).
//! The Lua `Material` wraps it with the typed parameter struct and the
//! per-draw callback (`script/Shared/Rendering/Material.lua`).
//!
//! The parameters are written once per change: Lua fills the struct behind
//! `Material_Params`, then `commit()` sends one `WriteBuffer` (and recreates
//! the bind group if a texture changed). Nothing is written per draw.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use crossbeam::channel::Sender;
use tracing::warn;

use super::{
    ArenaSlice, BindEntry, BindGroupId, BlockLayout, GROUP_MATERIAL, GROUP_UNIT_COUNT,
    PipelineDesc, PipelineId, Release, SamplerId, ShaderLayout, TexDim, TexView, ViewDim,
};
use crate::render::{BlendMode, CompareFn, CullFace, DepthState, Renderer, ResourceId, Shader};

/// Name of the group-1 uniform block that holds a material's parameters.
pub const MATERIAL_PARAMS_BLOCK: &str = "MaterialParams";

static NEXT_UID: AtomicU32 = AtomicU32::new(1);

pub struct Material {
    shader: Shader,
    layout: Arc<ShaderLayout>,
    template: PipelineDesc,
    /// The bucket the material draws in (its blend mode).
    blend: BlendMode,
    /// Index of `MaterialParams` among the group-1 blocks, if the shader has
    /// one.
    params_index: Option<u8>,
    /// The reflected layout of `MaterialParams` the CPU copy follows (what a
    /// hot reload copies fields from, by name).
    block: Option<BlockLayout>,
    /// The CPU copy of the block. Heap-allocated, so the pointer Lua holds
    /// stays valid for the life of the material.
    params: Box<[u8]>,
    slice: Option<ArenaSlice>,
    /// Texture and sampler per group-1 sampler declaration.
    textures: [Option<(TexView, SamplerId)>; GROUP_UNIT_COUNT[GROUP_MATERIAL as usize] as usize],
    bind_group: Option<BindGroupId>,
    /// Textures or the slice changed since the bind group was made.
    dirty: bool,
    /// The pipeline for the shader's current resource and generation.
    pipeline: Option<(ResourceId, u32, PipelineId)>,
    uid: u32,
    release: Sender<Release>,
    warned_missing: bool,
}

fn dim_matches(decl: TexDim, view: ViewDim) -> bool {
    matches!(
        (decl, view),
        (TexDim::D1, ViewDim::D1)
            | (TexDim::D2, ViewDim::D2)
            | (TexDim::D3, ViewDim::D3)
            | (TexDim::Cube, ViewDim::Cube)
    )
}

impl Material {
    pub fn new(
        r: &mut Renderer,
        shader: &Shader,
        blend: BlendMode,
        cull: CullFace,
        depth_test: bool,
        depth_write: bool,
    ) -> Material {
        let layout = shader.layout();
        let params_index = layout
            .block(MATERIAL_PARAMS_BLOCK)
            .filter(|b| b.group == GROUP_MATERIAL)
            .map(|b| b.index);
        let block = Self::params_block(shader, params_index);
        let params_size = block.as_ref().map_or(0, |b| b.size);
        let mut template = PipelineDesc::for_shader(shader);
        template.blend = blend;
        template.cull = cull;
        template.depth = DepthState {
            test: depth_test,
            write: depth_write,
            compare: CompareFn::LessEqual,
        };
        Material {
            shader: shader.clone(),
            layout,
            template,
            blend,
            params_index,
            block,
            params: vec![0u8; params_size as usize].into_boxed_slice(),
            slice: None,
            textures: Default::default(),
            bind_group: None,
            dirty: true,
            pipeline: None,
            uid: NEXT_UID.fetch_add(1, Ordering::Relaxed),
            release: r.data.release_tx.clone(),
            warned_missing: false,
        }
    }

    /// The reflected `MaterialParams` block, if the shader has one in group 1.
    fn params_block(shader: &Shader, params_index: Option<u8>) -> Option<BlockLayout> {
        params_index?;
        shader
            .blocks()
            .iter()
            .find(|b| b.name == MATERIAL_PARAMS_BLOCK)
            .cloned()
    }

    /// Unique id, the middle part of the scene sort key.
    pub fn uid(&self) -> u32 {
        self.uid
    }

    pub fn blend_mode(&self) -> BlendMode {
        self.blend
    }

    /// The pipeline of this material: the template with the shader's
    /// current resource and generation (a hot reload gives it new ones).
    pub fn pipeline(&mut self, r: &mut Renderer) -> PipelineId {
        let resource = self.shader.resource();
        let generation = self.shader.generation();
        if let Some((cached_for, cached_generation, id)) = self.pipeline {
            if cached_for == resource && cached_generation == generation {
                return id;
            }
        }
        self.template.shader = resource;
        self.template.shader_generation = generation;
        let id = r.get_pipeline(&self.template);
        self.pipeline = Some((resource, generation, id));
        id
    }

    /// The shader was hot reloaded: take its new layout. The parameter copy
    /// follows the new `MaterialParams` block (members are copied from the old
    /// one by name, new ones are zero), the textures are kept by sampler name
    /// (one the shader no longer declares, or declares with another
    /// dimension, is dropped), and the arena slice and the bind group go back
    /// to be remade by the next `commit`. The pointer of `params()` changes:
    /// recast it. Not allowed inside a pass.
    pub fn refresh_shader(&mut self, r: &mut Renderer) -> RefreshReport {
        debug_assert!(
            r.data.pass.open.is_none(),
            "Material:refreshShader() inside an open render pass: resource writes are not allowed there"
        );
        let layout = self.shader.layout();
        let params_index = layout
            .block(MATERIAL_PARAMS_BLOCK)
            .filter(|b| b.group == GROUP_MATERIAL)
            .map(|b| b.index);
        let block = Self::params_block(&self.shader, params_index);

        // Parameters, by member name.
        let mut params = vec![0u8; block.as_ref().map_or(0, |b| b.size) as usize];
        let blocks = match (&self.block, &block) {
            (Some(old), Some(new)) => old.copy_members(&self.params, new, &mut params),
            (None, Some(new)) => super::CopyReport {
                added: new.members.iter().map(|m| m.name.clone()).collect(),
                ..Default::default()
            },
            (Some(old), None) => super::CopyReport {
                dropped: old.members.iter().map(|m| m.name.clone()).collect(),
                ..Default::default()
            },
            (None, None) => Default::default(),
        };
        let layout_changed = self.block.as_ref().map(BlockLayout::layout_hash)
            != block.as_ref().map(BlockLayout::layout_hash);

        // Textures, by sampler name.
        let mut by_name: HashMap<String, (TexView, SamplerId)> = HashMap::new();
        for decl in self
            .layout
            .textures
            .iter()
            .filter(|t| t.group == GROUP_MATERIAL)
        {
            if let Some(t) = self.textures[decl.index as usize] {
                by_name.insert(decl.name.clone(), t);
            }
        }
        let mut textures: [Option<(TexView, SamplerId)>;
            GROUP_UNIT_COUNT[GROUP_MATERIAL as usize] as usize] = Default::default();
        let mut textures_dropped = Vec::new();
        for decl in layout.textures.iter().filter(|t| t.group == GROUP_MATERIAL) {
            match by_name.remove(&decl.name) {
                Some((view, sampler)) if dim_matches(decl.dim, view.dim) => {
                    textures[decl.index as usize] = Some((view, sampler));
                }
                Some(_) => textures_dropped.push(decl.name.clone()),
                None => {}
            }
        }
        textures_dropped.extend(by_name.into_keys());

        // What the GPU had of the old layout goes back; `commit` makes it anew.
        if let Some(id) = self.bind_group.take() {
            let _ = self.release.send(Release::BindGroup(id));
        }
        if let Some(slice) = self.slice.take() {
            let _ = self.release.send(Release::Slice(slice));
        }
        self.layout = layout;
        self.params_index = params_index;
        self.block = block;
        self.params = params.into_boxed_slice();
        self.textures = textures;
        self.template.shader = self.shader.resource();
        self.template.shader_generation = self.shader.generation();
        self.pipeline = None;
        self.dirty = true;
        self.warned_missing = false;

        RefreshReport {
            layout_changed,
            copied: blocks.copied.join(","),
            added: blocks.added.join(","),
            dropped: blocks.dropped.join(","),
            textures_dropped: textures_dropped.join(","),
        }
    }

    /// The bind group, committing first if the material was never committed
    /// or changed since.
    pub fn ensure_ready(&mut self, r: &mut Renderer) -> BindGroupId {
        if self.dirty || self.bind_group.is_none() {
            self.commit_intern(r, true);
        }
        self.bind_group.expect("commit creates the bind group")
    }

    /// `warn_missing`: complain about samplers without a texture (when the
    /// material is about to be drawn, not while it is still being set up).
    fn commit_intern(&mut self, r: &mut Renderer, warn_missing: bool) {
        debug_assert!(
            r.data.pass.open.is_none(),
            "Material:commit() inside an open render pass: resource writes are not allowed there"
        );
        let mut entries = Vec::new();
        if let Some(index) = self.params_index {
            if self.slice.is_none() {
                self.slice = Some(r.alloc_material_slice(self.params.len() as u32));
            }
            let slice = self.slice.expect("just allocated");
            entries.push(BindEntry::Uniform {
                index,
                buffer: slice.buffer,
                offset: slice.offset,
                size: self.params.len() as u32,
            });
            r.write_buffer(slice.buffer, slice.offset, self.params.to_vec());
        }
        if self.dirty || self.bind_group.is_none() {
            let mut missing = Vec::new();
            for decl in self
                .layout
                .textures
                .iter()
                .filter(|t| t.group == GROUP_MATERIAL)
            {
                match self.textures[decl.index as usize] {
                    Some((view, sampler)) => entries.push(BindEntry::Texture {
                        index: decl.index,
                        view,
                        sampler,
                    }),
                    None => missing.push(decl.name.clone()),
                }
            }
            if warn_missing && !missing.is_empty() && !self.warned_missing {
                self.warned_missing = true;
                warn!(
                    "Material of shader {}: no texture set for {missing:?}",
                    self.shader.name()
                );
            }
            let id = r.create_bind_group_entries(self.shader.resource(), GROUP_MATERIAL, entries);
            if let Some(old) = self.bind_group.replace(id) {
                let _ = self.release.send(Release::BindGroup(old));
            }
            self.dirty = false;
        }
    }
}

/// What `Material::refresh_shader` did, as comma separated names.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RefreshReport {
    /// The `MaterialParams` layout hash differs from before.
    pub layout_changed: bool,
    /// Members carried over from the old block.
    pub copied: String,
    /// Members of the new block that start at zero.
    pub added: String,
    /// Members of the old block the new one lacks.
    pub dropped: String,
    /// Textures that could not be kept.
    pub textures_dropped: String,
}

impl Drop for Material {
    fn drop(&mut self) {
        if let Some(id) = self.bind_group.take() {
            let _ = self.release.send(Release::BindGroup(id));
        }
        if let Some(slice) = self.slice.take() {
            let _ = self.release.send(Release::Slice(slice));
        }
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl Material {
    /// A material drawing with `shader`. `blend` picks the scene bucket and
    /// the blending of the pipeline; the other arguments complete its state.
    #[bind(name = "Create")]
    pub fn create(
        r: &mut Renderer,
        shader: &Shader,
        blend: BlendMode,
        cull: CullFace,
        depth_test: bool,
        depth_write: bool,
    ) -> Material {
        Material::new(r, shader, blend, cull, depth_test, depth_write)
    }

    /// The shader was hot reloaded: adopt its layout (see
    /// `Material::refresh_shader`). Returns a one-line report,
    /// `changed=<0|1> copied=<names> added=<names> dropped=<names> textures_dropped=<names>`.
    /// Recast `params()` afterwards (the parameter copy moved), then `commit()`.
    #[bind(name = "RefreshShader")]
    pub fn refresh_shader_report(&mut self, r: &mut Renderer) -> String {
        let report = self.refresh_shader(r);
        format!(
            "changed={} copied={} added={} dropped={} textures_dropped={}",
            report.layout_changed as u8,
            report.copied,
            report.added,
            report.dropped,
            report.textures_dropped
        )
    }

    /// Size in bytes of the `MaterialParams` block (0 if the shader has none).
    pub fn get_params_size(&self) -> u32 {
        self.params.len() as u32
    }

    /// Bind `view` to the group-1 sampler `name` the shader declares, sampled
    /// with `sampler`. Takes effect at the next `commit` (the bind group is
    /// recreated). The caller keeps the texture alive.
    pub fn set_texture(&mut self, name: &str, view: &TexView, sampler: u32) {
        let Some(decl) = self
            .layout
            .texture(name)
            .filter(|t| t.group == GROUP_MATERIAL)
        else {
            let declared: Vec<&str> = self
                .layout
                .textures
                .iter()
                .filter(|t| t.group == GROUP_MATERIAL)
                .map(|t| t.name.as_str())
                .collect();
            panic!(
                "Material:setTexture: shader {} has no sampler '{name}' in group 1 (declared: {declared:?})",
                self.shader.name()
            );
        };
        if !dim_matches(decl.dim, view.dim) {
            panic!(
                "Material:setTexture: sampler '{name}' of shader {} is a {:?} sampler, got a {:?} view",
                self.shader.name(),
                decl.dim,
                view.dim
            );
        }
        self.textures[decl.index as usize] = Some((*view, SamplerId(sampler as u16)));
        self.dirty = true;
    }

    /// Send the parameters to the GPU (one `WriteBuffer`) and recreate the
    /// bind group if a texture changed. Not allowed while a pass is open.
    pub fn commit(&mut self, r: &mut Renderer) {
        self.commit_intern(r, false);
    }

    /// The scene bucket (blend mode) the material draws in.
    pub fn get_blend(&self) -> BlendMode {
        self.blend
    }
}

/// `material:params()` backend: the CPU copy of the `MaterialParams` block.
/// Hand-written because the FFI generator cannot return raw pointers; the Lua
/// side casts it to the type of `shader:blockType('MaterialParams')`.
#[allow(unsafe_code, non_snake_case, improper_ctypes_definitions)]
#[unsafe(no_mangle)]
pub extern "C" fn Material_Params(material: &mut Material) -> *mut u8 {
    material.params.as_mut_ptr()
}

impl Renderer {
    /// A slice of a material parameter arena for a block of `size` bytes,
    /// creating a new arena buffer if needed.
    pub fn alloc_material_slice(&mut self, size: u32) -> ArenaSlice {
        let (slice, created) = self.data.arenas.alloc(size);
        if let Some(buffer) = created {
            self.create_buffer(buffer, super::ARENA_SIZE);
        }
        slice
    }

    /// Take back what dropped materials released: parameter slices go back to
    /// the arenas and their bind groups are destroyed on the executor. Called
    /// once per frame.
    pub fn drain_releases(&mut self) {
        let mut groups = Vec::new();
        while let Ok(release) = self.data.release_rx.try_recv() {
            match release {
                Release::Slice(slice) => self.data.arenas.release(slice),
                Release::BindGroup(id) => {
                    self.data.tex_pins.unpin_group(id);
                    groups.push(id);
                }
            }
        }
        if !groups.is_empty() {
            self.destroy_bind_groups(groups);
        }
    }
}
