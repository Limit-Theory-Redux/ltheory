//! Bind groups (doc/engine/render-api-v2.md, section 1.3): a set of textures
//! with their samplers and uniform blocks, bound to one group of a shader by
//! `pass:setBindGroup`. A uniform entry is a slice of a material parameter
//! arena (`buffer.rs`).

use std::collections::HashMap;

use crossbeam::channel::Sender;
use tracing::info;

use super::{BufferId, Release, SamplerId, ShaderLayout, TexView};
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
        /// Index within the group (see `wgpu_texture_binding`).
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

/// A bind group made from a `BindGroupDesc` (`Renderer:createBindGroup`),
/// owned by its creator. Dropping it releases the group the way a dropped
/// `Material` releases its own: `Renderer::drain_releases`, at the end of the
/// frame after the frame's pass commands went out, unpins its textures and
/// destroys it on the executor, which keeps the GPU object alive for work
/// already submitted.
pub struct BindGroup {
    id: BindGroupId,
    release: Sender<Release>,
}

impl BindGroup {
    pub fn group_id(&self) -> BindGroupId {
        self.id
    }
}

impl Drop for BindGroup {
    fn drop(&mut self) {
        // Fails only once the renderer is gone, and with it the group.
        let _ = self.release.send(Release::BindGroup(self.id));
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl BindGroup {
    /// The group's id, for diagnostics.
    pub fn id(&self) -> u32 {
        self.id.0
    }
}

/// The textures live bind groups sample. A `TexView` is a plain id, so a bind
/// group does not own its textures: when the Lua `Tex*` object behind one is
/// collected, its `ResourceHandle` queues the GPU texture for destruction while
/// the bind group still names it (a default texture would be sampled instead).
/// Every bind group therefore pins its textures here, and the destroy queue
/// holds a dropped texture back until no bind group references it any more.
#[derive(Debug, Default)]
pub struct TexturePins {
    /// Number of live bind groups referencing each texture.
    counts: HashMap<ResourceId, u32>,
    /// The textures each live bind group pinned.
    by_group: HashMap<BindGroupId, Vec<ResourceId>>,
    /// Dropped textures still pinned by a bind group.
    deferred: Vec<ResourceId>,
}

impl TexturePins {
    /// Pin the textures of the new bind group `id`.
    pub fn pin_group(&mut self, id: BindGroupId, entries: &[BindEntry]) {
        let textures: Vec<ResourceId> = entries
            .iter()
            .filter_map(|e| match e {
                BindEntry::Texture { view, .. } => Some(view.tex),
                BindEntry::Uniform { .. } => None,
            })
            .collect();
        if textures.is_empty() {
            return;
        }
        for &tex in &textures {
            *self.counts.entry(tex).or_insert(0) += 1;
        }
        self.by_group.insert(id, textures);
    }

    /// Unpin the textures of the destroyed bind group `id`.
    pub fn unpin_group(&mut self, id: BindGroupId) {
        let Some(textures) = self.by_group.remove(&id) else {
            return;
        };
        for tex in textures {
            if let Some(count) = self.counts.get_mut(&tex) {
                *count -= 1;
                if *count == 0 {
                    self.counts.remove(&tex);
                }
            }
        }
    }

    /// Of the textures dropped since the last call (`dropped`) and those held
    /// back before, the ones no live bind group references: these can be
    /// destroyed now. The rest wait for their bind groups.
    pub fn destroyable(&mut self, dropped: Vec<ResourceId>) -> Vec<ResourceId> {
        if self.counts.is_empty() && self.deferred.is_empty() {
            return dropped;
        }
        let mut now = Vec::with_capacity(dropped.len());
        let mut wait = Vec::new();
        for id in dropped {
            if self.counts.contains_key(&id) {
                info!("texture {id:?} dropped while a bind group samples it: destruction deferred");
                wait.push(id);
            } else {
                now.push(id);
            }
        }
        for id in std::mem::take(&mut self.deferred) {
            if self.counts.contains_key(&id) {
                wait.push(id);
            } else {
                now.push(id);
            }
        }
        self.deferred = wait;
        now
    }

    /// Dropped textures waiting for their bind groups.
    pub fn deferred_count(&self) -> usize {
        self.deferred.len()
    }
}

impl Renderer {
    /// Create the bind group on the render thread, owned by the returned
    /// `BindGroup` (see there for its release).
    pub fn create_bind_group_from_desc(&mut self, desc: &BindGroupDesc) -> BindGroup {
        BindGroup {
            id: self.create_bind_group_entries(desc.shader, desc.group, desc.entries.clone()),
            release: self.data.release_tx.clone(),
        }
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
        self.data.tex_pins.pin_group(id, &entries);
        self.create_bind_group_intern(id, shader, group, entries.into_boxed_slice());
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::ViewDim;

    fn tex_entry(index: u8, tex: u64) -> BindEntry {
        BindEntry::Texture {
            index,
            view: TexView::full(ResourceId(tex), ViewDim::D2, [4, 4]),
            sampler: SamplerId(0),
        }
    }

    #[test]
    fn dropped_texture_waits_for_its_bind_groups() {
        let mut pins = TexturePins::default();
        pins.pin_group(BindGroupId(1), &[tex_entry(0, 10), tex_entry(1, 11)]);
        pins.pin_group(BindGroupId(2), &[tex_entry(0, 10)]);

        // The Lua textures are collected while both groups still sample 10.
        assert_eq!(
            pins.destroyable(vec![ResourceId(10), ResourceId(12)]),
            vec![ResourceId(12)]
        );
        assert_eq!(pins.deferred_count(), 1);

        pins.unpin_group(BindGroupId(1));
        assert!(
            pins.destroyable(Vec::new()).is_empty(),
            "group 2 still samples 10"
        );
        pins.unpin_group(BindGroupId(2));
        assert_eq!(pins.destroyable(Vec::new()), vec![ResourceId(10)]);
        assert_eq!(pins.deferred_count(), 0);
        // 11 was never dropped: unpinning alone destroys nothing.
        assert!(pins.destroyable(Vec::new()).is_empty());
    }

    #[test]
    fn unpinned_textures_pass_straight_through() {
        let mut pins = TexturePins::default();
        pins.pin_group(BindGroupId(1), &[]); // uniform-only groups pin nothing
        pins.unpin_group(BindGroupId(7)); // unknown group: no-op
        assert_eq!(pins.destroyable(vec![ResourceId(3)]), vec![ResourceId(3)]);
    }

    /// A dropped `BindGroup` lets its texture go: until the end-of-frame
    /// release drain the texture is held back, after it nothing pins it.
    #[test]
    fn dropping_a_bind_group_releases_its_texture() {
        let mut r = crate::render::Renderer::new_headless();
        let group = BindGroup {
            id: r.create_bind_group_entries(ResourceId(1), 1, vec![tex_entry(0, 10)]),
            release: r.data.release_tx.clone(),
        };
        // The Lua texture is collected while the group still samples it.
        assert!(r.data.tex_pins.destroyable(vec![ResourceId(10)]).is_empty());

        drop(group);
        r.drain_releases();
        assert_eq!(
            r.data.tex_pins.destroyable(Vec::new()),
            vec![ResourceId(10)]
        );
        assert_eq!(r.data.tex_pins.deferred_count(), 0);
    }
}
