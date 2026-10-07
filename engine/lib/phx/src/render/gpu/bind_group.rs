//! Bind groups (doc/engine/render-api-v2.md, section 1.3): a set of textures
//! with their samplers and uniform blocks, bound to a group's fixed texture
//! units and block bindings by `pass:setBindGroup`. A uniform entry is a
//! slice of a material parameter arena (`buffer.rs`).

use super::{BufferId, SamplerId, ShaderLayout, TexView, block_binding, texture_unit};
use crate::render::{Renderer, ResourceId, Shader};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BindGroupId(pub u32);

#[derive(Debug, Clone)]
pub enum BindEntry {
    /// A range of a buffer bound to the group's `index`-th uniform block
    /// (binding `group * 4 + index`).
    Uniform {
        index: u8,
        buffer: BufferId,
        offset: u32,
        size: u32,
    },
    Texture {
        /// Index within the group; the unit is `GROUP_FIRST_UNIT[group] + index`.
        index: u8,
        view: TexView,
        sampler: SamplerId,
    },
}

/// A bind group under construction.
pub struct BindGroupDesc {
    shader: ResourceId,
    layout: std::sync::Arc<ShaderLayout>,
    group: u8,
    entries: Vec<BindEntry>,
}

impl BindGroupDesc {
    pub fn entries(&self) -> &[BindEntry] {
        &self.entries
    }

    pub fn group(&self) -> u8 {
        self.group
    }

    pub fn shader(&self) -> ResourceId {
        self.shader
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl BindGroupDesc {
    /// Start a bind group for `shader`'s group `group` (0..3).
    #[bind(name = "Create")]
    pub fn create(shader: &Shader, group: i32) -> BindGroupDesc {
        assert!(
            (0..4).contains(&group),
            "BindGroupDesc.Create: group {group} out of range (0..3)"
        );
        BindGroupDesc {
            shader: shader.resource(),
            layout: shader.layout(),
            group: group as u8,
            entries: Vec::new(),
        }
    }

    /// Bind `view` with `sampler` to the sampler `name` the shader declares
    /// in this group.
    pub fn texture(&mut self, name: &str, view: &TexView, sampler: u32) {
        let Some(decl) = self.layout.texture(name).filter(|t| t.group == self.group) else {
            let declared: Vec<&str> = self
                .layout
                .textures
                .iter()
                .filter(|t| t.group == self.group)
                .map(|t| t.name.as_str())
                .collect();
            panic!(
                "BindGroupDesc:texture: shader has no sampler '{name}' in group {} (declared: {declared:?})",
                self.group
            );
        };
        let entry = BindEntry::Texture {
            index: decl.index,
            view: *view,
            sampler: SamplerId(sampler as u16),
        };
        // Replace an earlier binding of the same sampler.
        self.entries.retain(|e| match e {
            BindEntry::Texture { index, .. } => *index != decl.index,
            BindEntry::Uniform { .. } => true,
        });
        self.entries.push(entry);
    }
}

impl Renderer {
    /// Create the bind group on the render thread. Bind groups made from a
    /// `BindGroupDesc` live until the renderer shuts down; materials own and
    /// free theirs.
    pub fn create_bind_group_from_desc(&mut self, desc: &BindGroupDesc) -> BindGroupId {
        self.create_bind_group_entries(desc.shader, desc.group, desc.entries.clone())
    }

    /// Create a bind group from `entries` and return its id.
    pub fn create_bind_group_entries(
        &mut self,
        shader: ResourceId,
        group: u8,
        entries: Vec<BindEntry>,
    ) -> BindGroupId {
        let id = BindGroupId(self.data.next_bind_group);
        self.data.next_bind_group += 1;
        self.create_bind_group_intern(id, shader, group, entries.into_boxed_slice());
        id
    }
}

/// The GL texture unit of a texture `entry` in `group` (`None` for uniform
/// entries, which bind a block instead).
pub fn entry_unit(group: u8, entry: &BindEntry) -> Option<u32> {
    match entry {
        BindEntry::Texture { index, .. } => Some(texture_unit(group, *index)),
        BindEntry::Uniform { .. } => None,
    }
}

/// The block binding point of a uniform `entry` in `group`.
pub fn entry_block_binding(group: u8, entry: &BindEntry) -> Option<u32> {
    match entry {
        BindEntry::Uniform { index, .. } => Some(block_binding(group, *index)),
        BindEntry::Texture { .. } => None,
    }
}
