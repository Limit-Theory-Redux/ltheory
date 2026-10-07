//! The scene list (doc/engine/render-api-v2.md, section 3b): everything a scene
//! pass draws, collected once per frame, then culled, sorted and emitted per
//! pass.
//!
//! Lua adds one `SceneTransform` per entity (written in place through a
//! pointer) and one item per mesh. `submit` for a bucket then frustum-culls
//! the bucket's items (a negative scale is the "never cull" sentinel), sorts
//! the survivors by `(pipeline, material, mesh)` (the alpha bucket keeps
//! insertion order, so blended draws do not change which one wins at equal
//! depth), lets the Lua side fill the per-draw values of the survivors that
//! need them, and emits `SetPipeline`/`SetBindGroup`/`SetDraw`/`DrawMesh`
//! with one `DrawBlock` per drawn mesh written straight into the uniform
//! ring. `mWorldIT` is computed once per transform, for survivors only.
//!
//! The split of `submit` into `prepare` and `emit` exists only so the
//! per-draw callbacks (Lua) run between them without a C-to-Lua call:
//! `ffi_ext/SceneList.lua` provides `list:submit(pass, blend, cull)`.

use glam::{Mat4, Vec3};

use super::{BindGroupId, DRAW_USER_FLOATS, DrawBlock, Material, PassCmd, PipelineId};
use crate::render::{BlendMode, CameraRenderData, Mesh, Renderer, ResourceId};
use crate::system::Metric;

/// One entity's transform. `repr(C)`: Lua writes it through a pointer
/// (`ffi_ext/SceneList.lua` declares the same struct).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SceneTransform {
    /// Local to camera-relative world, column-major.
    pub world: [f32; 16],
    /// Cull sphere centre, camera-relative (the body's position).
    pub cx: f32,
    pub cy: f32,
    pub cz: f32,
    /// The body's uniform scale; negative means "no bounds, never cull".
    pub scale: f32,
    /// Index of this transform in the list (set by `add_transform`).
    pub index: u32,
    pub _pad: [u32; 3],
}

#[derive(Debug, Clone, Copy)]
struct SceneItem {
    transform: u32,
    mesh: ResourceId,
    index_count: u32,
    vertex_count: u32,
    pipeline: PipelineId,
    bind_group: BindGroupId,
    material: u32,
    /// Radius of the sphere around the mesh's local origin that contains it,
    /// before the transform's scale.
    local_radius: f32,
}

/// Number of buckets (`BlendMode` discriminants).
const BUCKETS: usize = 4;

/// What `submit` found (since the last `reset`).
#[derive(Debug, Clone, Copy, Default)]
pub struct SceneStats {
    pub submitted: u32,
    pub visible: u32,
    pub culled: u32,
}

pub struct SceneList {
    transforms: Vec<SceneTransform>,
    items: Vec<SceneItem>,
    /// Per-draw values of each item (`drawUser`), zeroed when the item is
    /// added; Lua fills those of the survivors that have a callback.
    users: Vec<[f32; DRAW_USER_FLOATS]>,
    /// Item indices per bucket, in insertion order.
    buckets: [Vec<u32>; BUCKETS],
    /// 1 if the item survived the last `prepare` of its bucket.
    visible: Vec<u8>,
    /// Survivors of the last `prepare`, in draw order.
    order: Vec<u32>,
    /// `mWorldIT` per transform for the current `emit`, computed on first use.
    world_it: Vec<[f32; 16]>,
    world_it_ready: Vec<bool>,
    stats: SceneStats,
}

impl SceneList {
    pub fn new() -> Self {
        Self {
            transforms: Vec::with_capacity(256),
            items: Vec::with_capacity(512),
            users: Vec::with_capacity(512),
            buckets: Default::default(),
            visible: Vec::new(),
            order: Vec::new(),
            world_it: Vec::new(),
            world_it_ready: Vec::new(),
            stats: SceneStats::default(),
        }
    }

    pub fn reset(&mut self) {
        self.transforms.clear();
        self.items.clear();
        self.users.clear();
        for bucket in &mut self.buckets {
            bucket.clear();
        }
        self.visible.clear();
        self.order.clear();
        self.stats = SceneStats::default();
    }

    pub fn push_transform(&mut self) -> &mut SceneTransform {
        let index = self.transforms.len() as u32;
        self.transforms.push(SceneTransform {
            index,
            ..Default::default()
        });
        self.transforms.last_mut().expect("just pushed")
    }

    #[allow(clippy::too_many_arguments)]
    fn push_item(
        &mut self,
        bucket: BlendMode,
        transform: u32,
        mesh: ResourceId,
        index_count: u32,
        vertex_count: u32,
        pipeline: PipelineId,
        bind_group: BindGroupId,
        material: u32,
        local_radius: f32,
    ) -> u32 {
        assert!(
            (transform as usize) < self.transforms.len(),
            "SceneList:addItem: transform {transform} was not added"
        );
        let index = self.items.len() as u32;
        self.items.push(SceneItem {
            transform,
            mesh,
            index_count,
            vertex_count,
            pipeline,
            bind_group,
            material,
            local_radius,
        });
        self.users.push([0.0; DRAW_USER_FLOATS]);
        self.buckets[bucket as usize].push(index);
        index
    }

    /// Cull and sort the items of `bucket`. Returns the number of survivors;
    /// `visible()` says which items they are and `order()` their draw order.
    /// With `cull` false every item survives, in insertion order.
    pub fn prepare(&mut self, bucket: BlendMode, cull: bool, camera: &CameraRenderData) -> u32 {
        let indices = &self.buckets[bucket as usize];
        self.visible.clear();
        self.visible.resize(self.items.len(), 0);
        self.order.clear();

        let total = indices.len() as u32;
        if cull {
            for &i in indices {
                let item = &self.items[i as usize];
                let t = &self.transforms[item.transform as usize];
                // Negative scale: no bounds source, never cull.
                let keep = t.scale < 0.0
                    || camera.sphere_in_frustum(
                        Vec3::new(t.cx, t.cy, t.cz),
                        item.local_radius * t.scale,
                    );
                if keep {
                    self.visible[i as usize] = 1;
                    self.order.push(i);
                }
            }
            // Alpha keeps insertion order: sorting blended draws would change
            // which one wins at equal depth, i.e. change pixels.
            if bucket != BlendMode::Alpha {
                let items = &self.items;
                // Stable: equal keys keep their insertion order.
                self.order.sort_by_key(|&i| {
                    let item = &items[i as usize];
                    (item.pipeline.0, item.material, item.mesh.0)
                });
            }
        } else {
            for &i in indices {
                self.visible[i as usize] = 1;
                self.order.push(i);
            }
        }

        let visible = self.order.len() as u32;
        self.stats.submitted += total;
        self.stats.visible += visible;
        self.stats.culled += total - visible;
        visible
    }

    /// Per-item survivor flags of the last `prepare`.
    pub fn visible(&self) -> &[u8] {
        &self.visible
    }

    pub fn order(&self) -> &[u32] {
        &self.order
    }

    pub fn stats(&self) -> SceneStats {
        self.stats
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    pub fn transform_count(&self) -> usize {
        self.transforms.len()
    }

    /// The per-draw values of all items, `DRAW_USER_FLOATS` floats each.
    pub fn users_ptr(&mut self) -> *mut f32 {
        self.users.as_mut_ptr() as *mut f32
    }

    /// Emit the draws of the last `prepare`.
    pub fn emit(&mut self, r: &mut Renderer) {
        r.pass_require_open("submit");
        self.world_it_ready.clear();
        self.world_it_ready.resize(self.transforms.len(), false);
        self.world_it.resize(self.transforms.len(), [0.0; 16]);

        let mut pipeline = None;
        let mut bind_group = None;
        let (mut draws, mut polys, mut verts) = (0u64, 0u64, 0u64);
        for k in 0..self.order.len() {
            let i = self.order[k] as usize;
            let item = self.items[i];
            if pipeline != Some(item.pipeline) {
                pipeline = Some(item.pipeline);
                // The pipeline resets nothing else; the material's bind group
                // is set for every material change below.
                r.data.encoder.push(PassCmd::SetPipeline(item.pipeline));
            }
            if bind_group != Some(item.bind_group) {
                bind_group = Some(item.bind_group);
                r.data.encoder.push(PassCmd::SetBindGroup {
                    group: super::GROUP_MATERIAL,
                    id: item.bind_group,
                });
            }

            let t = item.transform as usize;
            if !self.world_it_ready[t] {
                let world = Mat4::from_cols_array(&self.transforms[t].world);
                self.world_it[t] = world.inverse().transpose().to_cols_array();
                self.world_it_ready[t] = true;
            }
            let scale = self.transforms[t].scale;
            let block = DrawBlock::new(
                &self.transforms[t].world,
                &self.world_it[t],
                if scale < 0.0 { 1.0 } else { scale },
                &self.users[i],
            );
            let at = r.data.ring.alloc_copy(block.as_bytes());
            r.data.encoder.push(PassCmd::SetDraw {
                at,
                size: DrawBlock::SIZE,
            });
            r.pass_draw(PassCmd::DrawMesh {
                mesh: item.mesh,
                first_index: 0,
                index_count: item.index_count,
            });

            draws += 1;
            polys += item.index_count as u64 / 3;
            verts += item.vertex_count as u64;
        }
        if draws > 0 {
            Metric::add_draws(draws, polys, polys, verts);
        }
    }
}

impl Default for SceneList {
    fn default() -> Self {
        Self::new()
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl SceneList {
    #[bind(name = "Create")]
    pub fn create() -> SceneList {
        SceneList::new()
    }

    /// Forget last frame's transforms and items (capacity is kept).
    #[bind(name = "Reset")]
    pub fn reset_list(&mut self) {
        self.reset();
    }

    /// Add one mesh of the entity whose transform is `transform` (an index
    /// from `addTransform`). Returns the item index. Commits the material if
    /// it has changed since it was last committed, so this must not run
    /// inside an open pass. The bucket is the material's blend mode.
    pub fn add_item(
        &mut self,
        r: &mut Renderer,
        transform: u32,
        mesh: &mut Mesh,
        material: &mut Material,
    ) -> u32 {
        let bind_group = material.ensure_ready(r);
        let pipeline = material.pipeline(r);
        let (mesh_id, index_count) = mesh.resource_and_index_count(r);
        // Sphere around the mesh's local origin that contains the mesh: its
        // own radius plus the distance of its centre from the origin. Exact
        // for origin-centred meshes, conservative otherwise.
        let mut center = glam::Vec3::ZERO;
        mesh.get_center(&mut center);
        let local_radius = center.length() + mesh.get_radius();
        self.push_item(
            material.blend_mode(),
            transform,
            mesh_id,
            index_count,
            mesh.get_vertex_count().max(0) as u32,
            pipeline,
            bind_group,
            material.uid(),
            local_radius,
        )
    }

    /// Cull and sort the items of the bucket `blend`; returns how many
    /// survive. Lua then fills the per-draw values of the survivors that
    /// have a callback and calls `emit`.
    #[bind(name = "Prepare")]
    pub fn prepare_bucket(&mut self, r: &mut Renderer, blend: BlendMode, cull: bool) -> u32 {
        let camera = &r.data.camera;
        let data = CameraRenderData::new(camera.view, camera.proj, Vec3::ZERO);
        self.prepare(blend, cull, &data)
    }

    /// Emit the draws of the last `prepare` into the open pass.
    #[bind(name = "Emit")]
    pub fn emit_bucket(&mut self, r: &mut Renderer) {
        self.emit(r);
    }

    pub fn get_item_count(&self) -> u32 {
        self.items.len() as u32
    }

    pub fn get_transform_count(&self) -> u32 {
        self.transforms.len() as u32
    }

    /// Items submitted since the last reset (all buckets).
    pub fn get_submitted(&self) -> u32 {
        self.stats.submitted
    }

    /// Items that survived culling since the last reset.
    pub fn get_visible(&self) -> u32 {
        self.stats.visible
    }

    /// Items culled since the last reset.
    pub fn get_culled(&self) -> u32 {
        self.stats.culled
    }
}

/// `list:addTransform()` backend: a zeroed transform with its `index` set.
/// The pointer is valid until the next `addTransform` or `reset`. Hand-written
/// because the FFI generator cannot return raw pointers.
#[allow(unsafe_code, non_snake_case, improper_ctypes_definitions)]
#[unsafe(no_mangle)]
pub extern "C" fn SceneList_AddTransform(list: &mut SceneList) -> *mut SceneTransform {
    list.push_transform() as *mut SceneTransform
}

/// Survivor flags (one byte per item) of the last `prepareBucket`.
#[allow(unsafe_code, non_snake_case, improper_ctypes_definitions)]
#[unsafe(no_mangle)]
pub extern "C" fn SceneList_Visible(list: &SceneList) -> *const u8 {
    list.visible.as_ptr()
}

/// The per-draw values of all items: 28 floats (7 `vec4`) per item.
#[allow(unsafe_code, non_snake_case, improper_ctypes_definitions)]
#[unsafe(no_mangle)]
pub extern "C" fn SceneList_Users(list: &mut SceneList) -> *mut f32 {
    list.users_ptr()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Matrix;

    fn camera() -> CameraRenderData {
        let view = Matrix::look_at(&Vec3::ZERO, &Vec3::new(0.0, 0.0, -1.0), &Vec3::Y);
        let proj = Matrix::perspective(90.0, 1.0, 0.1, 1000.0);
        CameraRenderData::new(
            Mat4::from_cols_array(&view.to_cols_array()),
            Mat4::from_cols_array(&proj.to_cols_array()),
            Vec3::ZERO,
        )
    }

    /// Add an entity at `z` (camera looks down -z) with one item.
    fn add(
        list: &mut SceneList,
        bucket: BlendMode,
        z: f32,
        scale: f32,
        pipeline: u32,
        material: u32,
        mesh: u64,
    ) -> u32 {
        let t = list.push_transform();
        t.cz = z;
        t.scale = scale;
        let index = t.index;
        list.push_item(
            bucket,
            index,
            ResourceId(mesh),
            36,
            24,
            PipelineId(pipeline),
            BindGroupId(material),
            material,
            1.0,
        )
    }

    fn ordered(list: &SceneList) -> Vec<u32> {
        list.order().to_vec()
    }

    #[test]
    fn survivors_sort_by_pipeline_material_mesh_stably() {
        let mut list = SceneList::new();
        let op = BlendMode::Disabled;
        // (pipeline, material, mesh): item index = insertion order.
        let a = add(&mut list, op, -10.0, 1.0, 2, 5, 1); // 0
        let b = add(&mut list, op, -10.0, 1.0, 1, 7, 1); // 1
        let c = add(&mut list, op, -10.0, 1.0, 2, 5, 1); // 2 (ties with a)
        let d = add(&mut list, op, -10.0, 1.0, 1, 7, 0); // 3 (mesh 0 first)
        let e = add(&mut list, op, -10.0, 1.0, 1, 6, 9); // 4 (material 6 before 7)
        assert_eq!((a, b, c, d, e), (0, 1, 2, 3, 4));
        assert_eq!(list.prepare(op, true, &camera()), 5);
        assert_eq!(ordered(&list), vec![4, 3, 1, 0, 2]);
    }

    #[test]
    fn alpha_keeps_insertion_order() {
        let mut list = SceneList::new();
        let al = BlendMode::Alpha;
        add(&mut list, al, -10.0, 1.0, 9, 1, 1);
        add(&mut list, al, -10.0, 1.0, 1, 1, 1);
        add(&mut list, al, -10.0, 1.0, 5, 1, 1);
        list.prepare(al, true, &camera());
        assert_eq!(ordered(&list), vec![0, 1, 2]);
    }

    #[test]
    fn items_outside_the_frustum_are_culled() {
        let mut list = SceneList::new();
        let op = BlendMode::Disabled;
        add(&mut list, op, -10.0, 1.0, 0, 0, 0); // in front: visible
        add(&mut list, op, 10.0, 1.0, 0, 0, 0); // behind: culled
        assert_eq!(list.prepare(op, true, &camera()), 1);
        assert_eq!(list.visible(), &[1, 0]);
        let s = list.stats();
        assert_eq!((s.submitted, s.visible, s.culled), (2, 1, 1));
    }

    #[test]
    fn negative_scale_is_never_culled() {
        let mut list = SceneList::new();
        let op = BlendMode::Disabled;
        add(&mut list, op, 1_000_000.0, -1.0, 0, 0, 0);
        assert_eq!(list.prepare(op, true, &camera()), 1);
        assert_eq!(list.stats().culled, 0);
    }

    #[test]
    fn the_radius_scales_with_the_transform() {
        let mut list = SceneList::new();
        let op = BlendMode::Disabled;
        // Centre just behind the camera: a unit sphere is culled, a large
        // one reaches into the frustum.
        add(&mut list, op, 3.0, 1.0, 0, 0, 0);
        add(&mut list, op, 3.0, 100.0, 0, 0, 0);
        list.prepare(op, true, &camera());
        assert_eq!(list.visible(), &[0, 1]);
    }

    #[test]
    fn buckets_are_separate_and_unculled_keeps_insertion_order() {
        let mut list = SceneList::new();
        add(&mut list, BlendMode::Disabled, 10.0, 1.0, 3, 0, 0); // behind, but unculled
        add(&mut list, BlendMode::Additive, -10.0, 1.0, 1, 0, 0);
        add(&mut list, BlendMode::Disabled, -10.0, 1.0, 2, 0, 0);
        assert_eq!(list.prepare(BlendMode::Disabled, false, &camera()), 2);
        assert_eq!(ordered(&list), vec![0, 2]);
        assert_eq!(list.prepare(BlendMode::Additive, true, &camera()), 1);
        assert_eq!(ordered(&list), vec![1]);
    }

    #[test]
    fn reset_keeps_nothing() {
        let mut list = SceneList::new();
        add(&mut list, BlendMode::Disabled, -10.0, 1.0, 0, 0, 0);
        list.reset();
        assert_eq!((list.item_count(), list.transform_count()), (0, 0));
        assert_eq!(list.prepare(BlendMode::Disabled, true, &camera()), 0);
    }
}
