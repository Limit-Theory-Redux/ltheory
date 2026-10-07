use glam::Vec3;
use tracing::error;

use crate::math::Matrix;
use crate::render::{
    BatchStats, BindGroupDesc, BlendMode, CmdPrimitiveType, CullFace, GpuHandle, InstanceData,
    LightUboData, MaterialUboData, RenderBatch, RenderPass, RenderPassDesc, Renderer, ResourceId,
    TexCube,
};

// =============================================================================
// FFI-exposed Renderer API
//
// These are thin Lua-facing wrappers: they convert Lua-friendly primitives
// (ints, separate floats, ...) into the native types the per-command methods
// on `Renderer` (defined in `renderer_immediate.rs`/`renderer_threaded.rs`)
// expect, then call those directly. The per-command methods carry an
// `_intern` suffix so these wrappers can keep the plain, Lua-facing name
// without clashing with them by name.
// =============================================================================

#[luajit_ffi_gen::luajit_ffi]
impl Renderer {
    // === Frame Management ===

    /// Begin a new frame
    pub fn begin_frame(&mut self) {
        self.begin_frame_intern();
    }

    /// Flush all queued commands to the render thread
    pub fn flush(&mut self) {
        self.flush_intern();
    }

    /// Synchronize with the render thread (wait for all commands to complete)
    pub fn sync(&mut self) -> bool {
        self.sync_intern()
    }

    // === Batch rendering ===

    pub fn begin_batch(&mut self, view: &Matrix, projection: &Matrix, eye: &Vec3) {
        match &mut self.data.active_batch {
            Some(batch) => batch.reset(view, projection, *eye),
            None => self.data.active_batch = Some(RenderBatch::new(view, projection, *eye)),
        }
    }

    /// `mesh_id`/`shader_id` are `ResourceId`s as plain scalars - obtain them
    /// from `Mesh::resource_id`/`Shader::resource_id` (`mesh:resourceId(r)` /
    /// `shader:resourceId()` in Lua). `user_id` is an opaque caller tag
    /// echoed back by `cull_batch`.
    #[allow(clippy::too_many_arguments)]
    pub fn add_entity(
        &mut self,
        transform: &Matrix,
        bounds_center: &Vec3,
        bounds_radius: f32,
        mesh_id: u64,
        index_count: i32,
        shader_id: u64,
        sort_key: u32,
        user_id: u32,
    ) {
        if let Some(batch) = &mut self.data.active_batch {
            batch.add_entity(
                transform,
                *bounds_center,
                bounds_radius,
                ResourceId(mesh_id),
                index_count,
                ResourceId(shader_id),
                sort_key,
                user_id,
            );
        } else {
            error!("There is no active batch started. Use begin_batch() to start it.");
        }
    }

    /// Add a cull-only entity to the active batch: bounds + sort key, no
    /// mesh/shader to draw. For callers that want frustum culling and sort
    /// ordering from `cull_batch` without going through the (unused) batch
    /// draw path - see `RenderCoreSystem` in Lua, which still applies its
    /// own per-entity material uniforms and issues its own draws.
    ///
    /// `radius < 0.0` is a sentinel meaning "never cull" (e.g. no bounds
    /// source available for this entity).
    pub fn add_cull_entity(
        &mut self,
        bounds_center: &Vec3,
        bounds_radius: f32,
        sort_key: u32,
        user_id: u32,
    ) {
        if let Some(batch) = &mut self.data.active_batch {
            batch.add_cull_entity(*bounds_center, bounds_radius, sort_key, user_id);
        } else {
            error!("There is no active batch started. Use begin_batch() to start it.");
        }
    }

    /// Frustum-cull and sort the active batch, writing survivors' `user_id`s
    /// into `out_indices` in sort-key order. Returns the number written
    /// (never more than `out_indices`'s length). Emits no draw commands and
    /// does not clear the batch - `flush_batch` still works afterward.
    pub fn cull_batch(&mut self, out_indices: &mut [u32]) -> u32 {
        let Some(batch) = &mut self.data.active_batch else {
            error!("There is no active batch started. Use begin_batch() to start it.");
            return 0;
        };

        batch.cull_and_sort();
        let visible = batch.visible();
        let n = visible.len().min(out_indices.len());
        for (dst, &i) in out_indices[..n].iter_mut().zip(visible.iter()) {
            *dst = batch.entities[i as usize].user_id;
        }
        n as u32
    }

    pub fn flush_batch(&mut self) {
        self.process_batch();
    }

    pub fn get_batch_stats(&self) -> Option<&BatchStats> {
        if let Some(batch) = &self.data.active_batch {
            Some(batch.get_stats())
        } else {
            error!("There is no active batch started. Use begin_batch() to start it.");
            None
        }
    }

    // === Frame stats (last completed frame; used by LTHEORY_CAPTURE) ===

    /// Draw calls of the last frame (mesh + immediate + instanced).
    pub fn stats_draw_calls(&mut self) -> u64 {
        let s = self.get_stats();
        s.draw_mesh_calls + s.draw_immediate_calls + s.draw_instanced_calls
    }

    /// Render-thread execute time of the last frame, in microseconds.
    pub fn stats_frame_time_us(&mut self) -> u64 {
        self.get_stats().last_frame_time_us
    }

    /// Time the render thread sat blocked waiting for commands in the last
    /// frame (producer starvation), microseconds.
    pub fn stats_recv_wait_us(&mut self) -> u64 {
        self.get_stats().recv_wait_us
    }

    /// Time the render thread spent blocked in the buffer swap (vsync/GPU
    /// back-pressure) in the last frame, microseconds.
    pub fn stats_present_wait_us(&mut self) -> u64 {
        self.get_stats().present_wait_us
    }

    /// Frames the render thread has completed (to de-duplicate stats samples).
    pub fn stats_frame_count(&mut self) -> u64 {
        self.get_stats().frame_count
    }

    /// Commands the render thread processed in the last frame.
    pub fn stats_commands(&mut self) -> u64 {
        self.get_stats().commands
    }

    /// Time the main thread spent blocked in the last frame end, microseconds.
    pub fn stats_main_wait_us(&self) -> u64 {
        self.get_main_thread_wait_us()
    }

    pub fn stats_vertices(&mut self) -> u64 {
        self.get_stats().vertices_drawn
    }

    // === State Management ===

    /// Set the viewport
    pub fn set_viewport(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.set_viewport_intern(x, y, width, height);
    }

    /// Set the scissor region
    pub fn set_scissor(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.set_scissor_intern(x, y, width, height);
    }

    /// Enable or disable scissor test
    pub fn enable_scissor(&mut self, enable: bool) {
        self.enable_scissor_intern(enable);
    }

    /// Set blend mode (0=Disabled, 1=Alpha, 2=Additive, 3=PreMultAlpha)
    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.set_blend_mode_intern(mode);
    }

    /// Set cull face (0=None, 1=Back, 2=Front)
    pub fn set_cull_face(&mut self, face: CullFace) {
        self.set_cull_face_intern(face);
    }

    /// Enable or disable depth testing
    pub fn set_depth_test(&mut self, enable: bool) {
        self.set_depth_test_intern(enable);
    }

    /// Enable or disable depth writing
    pub fn set_depth_writable(&mut self, enable: bool) {
        self.set_depth_writable_intern(enable);
    }

    /// Set wireframe mode
    pub fn set_wireframe(&mut self, enable: bool) {
        self.set_wireframe_intern(enable);
    }

    // === Shader Operations ===

    /// Bind a shader program
    pub fn bind_shader(&mut self, handle: u32) {
        self.bind_shader_intern(GpuHandle(handle));
    }

    /// Unbind the current shader
    pub fn unbind_shader(&mut self) {
        self.unbind_shader_intern();
    }

    /// Set an integer uniform
    pub fn set_uniform_int(&mut self, location: i32, value: i32) {
        self.set_uniform_int_intern(location, value);
    }

    /// Set a float uniform
    pub fn set_uniform_float(&mut self, location: i32, value: f32) {
        self.set_uniform_float_intern(location, value);
    }

    /// Set a vec2 uniform
    pub fn set_uniform_float2(&mut self, location: i32, x: f32, y: f32) {
        self.set_uniform_float2_intern(location, [x, y]);
    }

    /// Set a vec3 uniform
    pub fn set_uniform_float3(&mut self, location: i32, x: f32, y: f32, z: f32) {
        self.set_uniform_float3_intern(location, [x, y, z]);
    }

    /// Set a vec4 uniform
    pub fn set_uniform_float4(&mut self, location: i32, x: f32, y: f32, z: f32, w: f32) {
        self.set_uniform_float4_intern(location, [x, y, z, w]);
    }

    // === Texture Operations ===

    /// Bind a 2D texture to a slot
    pub fn bind_texture_2d(&mut self, slot: u32, handle: u32) {
        self.bind_texture_2d_intern(slot, GpuHandle(handle));
    }

    /// Bind a 3D texture to a slot
    pub fn bind_texture_3d(&mut self, slot: u32, handle: u32) {
        self.bind_texture_3d_intern(slot, GpuHandle(handle));
    }

    /// Bind a cube texture to a slot
    pub fn bind_texture_cube(&mut self, slot: u32, handle: u32) {
        self.bind_texture_cube_intern(slot, GpuHandle(handle));
    }

    /// Unbind a texture from a slot
    pub fn unbind_texture(&mut self, slot: u32) {
        self.unbind_texture_intern(slot);
    }

    // === Render Passes ===

    /// Begin a render pass on `desc`'s attachments. Only one pass may be open
    /// at a time; end it with `RenderPass:finish()`.
    pub fn begin_pass(&mut self, desc: &RenderPassDesc) -> RenderPass {
        self.begin_pass_intern(desc)
    }

    /// The open pass, for code that records into it without owning it (for
    /// example UI widgets calling `pass:setUiTransform`). It cannot `finish`
    /// the pass. Errors if no pass is open.
    pub fn current_pass(&self) -> RenderPass {
        self.current_pass_intern()
    }

    // === Drawing Operations ===

    /// Draw a mesh
    pub fn draw_mesh(&mut self, vao: u32, index_count: i32) {
        self.draw_mesh_intern(GpuHandle(vao), index_count, CmdPrimitiveType::Triangles);
    }

    /// Draw a mesh with a specific primitive type
    pub fn draw_mesh_primitive(&mut self, vao: u32, index_count: i32, primitive: CmdPrimitiveType) {
        self.draw_mesh_intern(GpuHandle(vao), index_count, primitive);
    }

    /// Draw instanced mesh
    pub fn draw_mesh_instanced(&mut self, vao: u32, index_count: i32, instance_count: i32) {
        self.draw_mesh_instanced_intern(
            GpuHandle(vao),
            index_count,
            instance_count,
            CmdPrimitiveType::Triangles,
        );
    }

    /// Draw instanced with per-instance data (mesh resource id variant).
    pub fn draw_instanced_with_data(
        &mut self,
        mesh_id: u64,
        index_count: i32,
        instances: &[InstanceData],
        primitive: CmdPrimitiveType,
    ) {
        self.draw_instanced_with_data_intern(
            ResourceId(mesh_id),
            index_count,
            instances,
            primitive,
        );
    }

    /// Draw instanced with per-instance u32 INDICES into a static data
    /// texture (texture-fetch instancing, GL 3.3). See
    /// draw_instanced_indices_intern.
    pub fn draw_instanced_indices(
        &mut self,
        mesh_id: u64,
        index_count: i32,
        indices: &[u32],
        primitive: CmdPrimitiveType,
    ) {
        self.draw_instanced_indices_intern(ResourceId(mesh_id), index_count, indices, primitive);
    }

    // === Window Operations ===

    /// Signal resize
    pub fn resize(&mut self, width: u32, height: u32) {
        self.resize_intern(width, height);
    }

    /// Signal swap buffers (frame end)
    pub fn swap_buffers(&mut self) {
        self.swap_buffers_intern();
    }

    // === Frame group (group 0) ===

    /// Set the camera of the passes that begin from now on (and of the open
    /// pass): view and projection matrices and the direction towards the
    /// primary light. Rendering is camera-relative, so the eye is the origin.
    /// Replaces the old shader-variable stack and the camera UBO update.
    pub fn set_camera(&mut self, view: &Matrix, proj: &Matrix, star_dir: &Vec3) {
        self.set_camera_intern(view, proj, *star_dir);
    }

    /// Set the environment cube maps (`envMap` and `irMap` of group 0) of the
    /// passes that begin from now on (and of the open pass). Replaces
    /// the old per-shader `envMap`/`irMap` variables.
    pub fn set_environment(&mut self, env_map: &TexCube, ir_map: &TexCube) {
        self.set_environment_intern(env_map, ir_map);
    }

    // === Binding model objects ===

    /// Create a bind group from `desc`; bind it in a pass with
    /// `pass:setBindGroup(group, id)`.
    pub fn create_bind_group(&mut self, desc: &BindGroupDesc) -> u32 {
        self.create_bind_group_from_desc(desc).0
    }

    /// Create the material UBO on the render thread
    pub fn create_material_ubo(&mut self) {
        self.create_material_ubo_intern();
    }

    /// Update the material UBO with new material properties
    pub fn update_material_ubo(
        &mut self,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
        metallic: f32,
        roughness: f32,
        emission: f32,
    ) {
        let mut data = MaterialUboData::new();
        data.set_color(r, g, b, a);
        data.set_metallic(metallic);
        data.set_roughness(roughness);
        data.set_emission(emission);

        self.update_material_ubo_intern(*data.as_bytes());
    }

    /// Create the light UBO on the render thread
    pub fn create_light_ubo(&mut self) {
        self.create_light_ubo_intern();
    }

    /// Update the light UBO with light properties
    #[allow(clippy::too_many_arguments)]
    pub fn update_light_ubo(
        &mut self,
        pos_x: f32,
        pos_y: f32,
        pos_z: f32,
        radius: f32,
        r: f32,
        g: f32,
        b: f32,
        intensity: f32,
    ) {
        let mut data = LightUboData::new();
        data.set_position(pos_x, pos_y, pos_z);
        data.set_radius(radius);
        data.set_color(r, g, b);
        data.set_intensity(intensity);

        self.update_light_ubo_intern(*data.as_bytes());
    }
}
