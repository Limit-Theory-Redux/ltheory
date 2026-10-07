use std::sync::Arc;

use glam::{IVec2, Vec3};

use super::{PASS_FLUSH_LIMIT, PassCmd, PassCommands, PipelineId, SamplerId, TexView, ViewBlock};
use crate::math::Matrix;
use crate::render::{InstanceData, Mesh, Renderer, ScissorUpdate};
use crate::system::{Metric, Profiler};

pub const MAX_COLOR_ATTACHMENTS: usize = 4;

/// What a pass does with an attachment's previous contents at begin.
#[luajit_ffi_gen::luajit_ffi(repr = "u32")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadOp {
    /// Keep the existing contents.
    Load,
    /// Clear to the attachment's clear value.
    Clear,
    /// Contents are undefined; the pass overwrites every pixel it reads back.
    DontCare,
}

/// GL 3.3 has no discard (`glInvalidateFramebuffer` is 4.3), so this is
/// currently informational on GL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreOp {
    Store,
    Discard,
}

#[derive(Debug, Clone, Copy)]
pub struct ColorAttachment {
    pub view: TexView,
    pub load: LoadOp,
    pub clear: [f32; 4],
    pub store: StoreOp,
}

#[derive(Debug, Clone, Copy)]
pub struct DepthAttachment {
    pub view: TexView,
    pub load: LoadOp,
    pub clear: f32,
    pub store: StoreOp,
}

/// Everything a render pass needs to begin. Sent to the executor by value in
/// `RenderCommand::BeginRenderPass`.
#[derive(Debug, Clone)]
pub struct RenderPassDesc {
    pub label: Arc<str>,
    /// Contiguous from index 0.
    pub color: [Option<ColorAttachment>; MAX_COLOR_ATTACHMENTS],
    pub depth: Option<DepthAttachment>,
    /// Draw to the window's default framebuffer instead of attachments.
    pub backbuffer: bool,
    pub back_color: (LoadOp, [f32; 4]),
    pub back_depth: (LoadOp, f32),
    /// Width and height the pass renders at; the viewport is set from it.
    pub extent: [u32; 2],
}

impl RenderPassDesc {
    pub fn new(label: &str) -> Self {
        Self {
            label: Arc::from(label),
            color: [None; MAX_COLOR_ATTACHMENTS],
            depth: None,
            backbuffer: false,
            back_color: (LoadOp::Load, [0.0; 4]),
            back_depth: (LoadOp::Load, 1.0),
            extent: [0, 0],
        }
    }

    /// Backbuffer pass of the given size that keeps its contents.
    pub fn new_backbuffer(label: &str, width: i32, height: i32) -> Self {
        let mut desc = Self::new(label);
        desc.backbuffer = true;
        desc.extent = [width.max(1) as u32, height.max(1) as u32];
        desc
    }

    /// Single color attachment, for the engine's own offscreen passes.
    pub fn with_color(label: &str, view: TexView, load: LoadOp, clear: [f32; 4]) -> Self {
        let mut desc = Self::new(label);
        desc.set_color(0, view, load, clear);
        desc
    }

    pub fn set_color(&mut self, index: usize, view: TexView, load: LoadOp, clear: [f32; 4]) {
        assert!(
            index < MAX_COLOR_ATTACHMENTS,
            "RenderPassDesc '{}': color index {index} out of range (max {MAX_COLOR_ATTACHMENTS})",
            self.label
        );
        self.color[index] = Some(ColorAttachment {
            view,
            load,
            clear,
            store: StoreOp::Store,
        });
        self.extent = view.extent;
    }

    pub fn set_depth(&mut self, view: TexView, load: LoadOp, clear: f32) {
        self.depth = Some(DepthAttachment {
            view,
            load,
            clear,
            store: StoreOp::Store,
        });
        self.extent = view.extent;
    }

    pub fn color_count(&self) -> usize {
        self.color.iter().take_while(|c| c.is_some()).count()
    }

    fn validate(&self) {
        let label = &self.label;
        let count = self.color_count();
        assert!(
            self.color[count..].iter().all(|c| c.is_none()),
            "RenderPassDesc '{label}': color attachments must be contiguous from index 0"
        );
        if self.backbuffer {
            assert!(
                count == 0 && self.depth.is_none(),
                "RenderPassDesc '{label}': a backbuffer pass cannot have texture attachments"
            );
        } else {
            assert!(
                count > 0 || self.depth.is_some(),
                "RenderPassDesc '{label}': no attachments (use backbuffer() for the window)"
            );
            for view in self
                .color
                .iter()
                .flatten()
                .map(|c| c.view)
                .chain(self.depth.iter().map(|d| d.view))
            {
                assert!(
                    view.extent == self.extent,
                    "RenderPassDesc '{label}': attachment extents differ ({:?} vs {:?})",
                    view.extent,
                    self.extent
                );
            }
        }
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl RenderPassDesc {
    #[bind(name = "Create")]
    pub fn create(label: &str) -> RenderPassDesc {
        Self::new(label)
    }

    /// Set color attachment `index` (0-3, contiguous). `r,g,b,a` is the clear
    /// value, used only with `LoadOp.Clear`.
    #[allow(clippy::too_many_arguments)]
    pub fn color(
        &mut self,
        index: i32,
        view: &TexView,
        load: LoadOp,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) {
        self.set_color(index as usize, *view, load, [r, g, b, a]);
    }

    /// Set the depth attachment. `d` is the clear value, used only with
    /// `LoadOp.Clear`.
    pub fn depth(&mut self, view: &TexView, load: LoadOp, d: f32) {
        self.set_depth(*view, load, d);
    }

    /// Target the window's backbuffer (`width` x `height` pixels) instead of
    /// texture attachments.
    #[allow(clippy::too_many_arguments)]
    pub fn backbuffer(
        &mut self,
        width: i32,
        height: i32,
        load: LoadOp,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) {
        self.backbuffer = true;
        self.extent = [width.max(1) as u32, height.max(1) as u32];
        self.back_color = (load, [r, g, b, a]);
    }

    /// Load op for the backbuffer's depth buffer.
    pub fn backbuffer_depth(&mut self, load: LoadOp, d: f32) {
        self.back_depth = (load, d);
    }
}

/// Handle of the one open render pass. `finish` ends it; the other methods
/// record pass commands (see `PassEncoder`).
pub struct RenderPass {
    label: Arc<str>,
    finished: bool,
}

#[luajit_ffi_gen::luajit_ffi]
impl RenderPass {
    pub fn finish(&mut self, r: &mut Renderer) {
        if self.finished {
            panic!(
                "RenderPass '{}': finish() on a pass that was already finished (or a handle from currentPass(), which only borrows the pass)",
                self.label
            );
        }
        self.finished = true;
        r.end_pass_intern();
    }

    /// Bind a pipeline (shader and fixed-function state) for the draws that
    /// follow.
    pub fn set_pipeline(&self, r: &mut Renderer, pipeline: u32) {
        r.pass_record("setPipeline", PassCmd::SetPipeline(PipelineId(pipeline)));
    }

    /// Stage pass input `slot` (0..3, group 3: texture units 12..15). All
    /// staged inputs are bound together before the next draw.
    pub fn set_input(&self, r: &mut Renderer, slot: i32, view: &TexView, sampler: u32) {
        r.pass_require_open("setInput");
        r.data
            .encoder
            .set_input(slot as usize, Some((*view, SamplerId(sampler as u16))));
    }

    /// Unbind pass input `slot`.
    pub fn clear_input(&self, r: &mut Renderer, slot: i32) {
        r.pass_require_open("clearInput");
        r.data.encoder.set_input(slot as usize, None);
    }

    /// Bind a bind group created with `Renderer:createBindGroup` to its
    /// group's units.
    pub fn set_bind_group(&self, r: &mut Renderer, group: i32, bind_group: u32) {
        r.pass_record(
            "setBindGroup",
            PassCmd::SetBindGroup {
                group: group as u8,
                id: super::BindGroupId(bind_group),
            },
        );
    }

    /// Draw `mesh` with the current pipeline.
    pub fn draw_mesh(&self, r: &mut Renderer, mesh: &mut Mesh) {
        r.pass_require_open("drawMesh");
        let (mesh, index_count) = mesh.resource_and_index_count(r);
        r.pass_draw(PassCmd::DrawMesh {
            mesh,
            first_index: 0,
            index_count,
        });
    }

    /// Instanced draw of `mesh`: one instance per `InstanceData` (model matrix,
    /// color, scale, as vertex attributes 4..9; see `instanced.glsl`). The data
    /// is copied into the vertex ring, so the Lua array can be reused at once.
    pub fn draw_mesh_instanced(
        &self,
        r: &mut Renderer,
        mesh: &mut Mesh,
        instances: &[InstanceData],
    ) {
        r.pass_require_open("drawMeshInstanced");
        if instances.is_empty() {
            return;
        }
        let (mesh, index_count) = mesh.resource_and_index_count(r);
        #[allow(unsafe_code)]
        // SAFETY: `InstanceData` is repr(C) plain floats without padding.
        let bytes = unsafe {
            std::slice::from_raw_parts(
                instances.as_ptr() as *const u8,
                std::mem::size_of_val(instances),
            )
        };
        let at = r.data.vertex_ring.alloc_copy(bytes);
        r.pass_draw(PassCmd::DrawMeshInstanced {
            mesh,
            index_count,
            instances: at,
            count: instances.len() as u32,
        });
    }

    /// Texture-fetch instancing: one instance per index, a `uint` attribute
    /// (location 10) that the vertex shader uses to `texelFetch` the instance
    /// transform from a static data texture (see `wvp_instanced_tex.glsl`).
    /// The indices are copied into the vertex ring (4 bytes per instance).
    pub fn draw_instanced_indices(&self, r: &mut Renderer, mesh: &mut Mesh, indices: &[u32]) {
        r.pass_require_open("drawInstancedIndices");
        if indices.is_empty() {
            return;
        }
        let (mesh, index_count) = mesh.resource_and_index_count(r);
        #[allow(unsafe_code)]
        // SAFETY: plain `u32`s viewed as bytes.
        let bytes = unsafe {
            std::slice::from_raw_parts(
                indices.as_ptr() as *const u8,
                std::mem::size_of_val(indices),
            )
        };
        let at = r.data.vertex_ring.alloc_copy(bytes);
        r.pass_draw(PassCmd::DrawInstancedIndices {
            mesh,
            index_count,
            indices: at,
            count: indices.len() as u32,
        });
    }

    /// Draw the built-in unit quad (pipeline vertex layout `Fullscreen`),
    /// scaled to the viewport by the vertex shader.
    pub fn draw_fullscreen(&self, r: &mut Renderer) {
        r.pass_draw(PassCmd::DrawFullscreen);
    }

    /// Restrict drawing to a sub-rectangle of the target. The UI projection
    /// follows the new size, like the viewport stack of old did.
    pub fn set_viewport(&self, r: &mut Renderer, x: i32, y: i32, width: i32, height: i32) {
        r.pass_set_viewport([x, y, width, height]);
    }

    pub fn set_scissor(&self, r: &mut Renderer, x: i32, y: i32, width: i32, height: i32) {
        r.pass_record(
            "setScissor",
            PassCmd::SetScissor(Some([x, y, width, height])),
        );
    }

    pub fn clear_scissor(&self, r: &mut Renderer) {
        r.pass_record("clearScissor", PassCmd::SetScissor(None));
    }

    /// Replace the model-view part of the UI transform (`mWorldViewUI`) for
    /// the draws that follow.
    pub fn set_ui_transform(&self, r: &mut Renderer, transform: &Matrix) {
        r.pass_set_ui_transform(transform.to_cols_array());
    }
}

/// `pass:alloc(T)` backend: `size` zeroed bytes of uniform ring staging that
/// become the group-2 block of the next draw. The pointer is valid until the
/// next draw is recorded (the pass auto-flushes only at an `alloc` entry or
/// after a draw), so write the fields, then draw. Hand-written because the
/// FFI generator cannot return raw pointers; the Lua side casts the result
/// to `T*` (see `ffi_ext/RenderPass.lua`).
#[allow(unsafe_code, non_snake_case, improper_ctypes_definitions)]
#[unsafe(no_mangle)]
pub extern "C" fn RenderPass_Alloc(_pass: &RenderPass, r: &mut Renderer, size: u32) -> *mut u8 {
    r.pass_alloc(size)
}

impl Renderer {
    /// Begin a render pass: bind the attachments, size the viewport, apply
    /// the load ops, and set up the pass's group-0 state (view block and
    /// environment). Only one pass may be open at a time.
    pub fn begin_pass_intern(&mut self, desc: &RenderPassDesc) -> RenderPass {
        Profiler::begin("RenderPass_Begin");
        if let Some(open) = &self.data.pass.open {
            panic!(
                "beginPass('{}'): pass '{open}' is still open (passes cannot nest; call finish() first)",
                desc.label
            );
        }
        desc.validate();

        Metric::FBOSwap.inc();
        self.begin_render_pass(Box::new(desc.clone()));

        let viewport = [0, 0, desc.extent[0] as i32, desc.extent[1] as i32];
        let camera = self.data.camera;
        let pass = &mut self.data.pass;
        pass.open = Some(desc.label.clone());
        pass.extent = desc.extent;
        pass.is_window = desc.backbuffer;
        pass.viewport = viewport;
        pass.view = ViewBlock::new(&camera, viewport, desc.backbuffer);

        self.pass_emit_view();
        self.pass_emit_environment();
        self.sync_scissor();
        Profiler::end();

        RenderPass {
            label: desc.label.clone(),
            finished: false,
        }
    }

    pub fn end_pass_intern(&mut self) {
        Profiler::begin("RenderPass_End");
        if self.data.pass.open.take().is_none() {
            panic!("RenderPass finish(): no pass is open");
        }
        Metric::FBOSwap.inc();
        // Hand the recorded commands over before the pass ends.
        self.flush_pass_encoder();
        self.end_render_pass();
        Profiler::end();
    }

    /// A handle on the open pass for code that records into it without owning
    /// it (it cannot `finish` it).
    pub fn current_pass_intern(&self) -> RenderPass {
        let label = self
            .data
            .pass
            .open
            .clone()
            .unwrap_or_else(|| panic!("currentPass(): no render pass is open"));
        RenderPass {
            label,
            finished: true,
        }
    }

    /// Size of the open pass's viewport, or of the last pass's target when
    /// none is open (what `ClipRect` and screen capture measure against).
    pub fn target_size(&self) -> IVec2 {
        let pass = &self.data.pass;
        if pass.open.is_some() {
            IVec2::new(pass.viewport[2], pass.viewport[3])
        } else {
            IVec2::new(pass.extent[0] as i32, pass.extent[1] as i32)
        }
    }

    pub(crate) fn pass_require_open(&self, what: &str) {
        if self.data.pass.open.is_none() {
            panic!("RenderPass:{what}: the pass is not open (it was finished, or never began)");
        }
    }

    /// Record `cmd` into the open pass.
    pub(crate) fn pass_record(&mut self, what: &str, cmd: PassCmd) {
        self.pass_require_open(what);
        self.data.encoder.push(cmd);
    }

    /// Record a draw: pending inputs go out first, and a full buffer flushes.
    pub(crate) fn pass_draw(&mut self, cmd: PassCmd) {
        self.pass_require_open("draw");
        let encoder = &mut self.data.encoder;
        if encoder.inputs_dirty {
            encoder.inputs_dirty = false;
            let inputs = Box::new(encoder.inputs);
            encoder.push(PassCmd::SetInputs(inputs));
        }
        encoder.push(cmd);
        if encoder.len() >= PASS_FLUSH_LIMIT {
            self.flush_pass_encoder();
        }
    }

    pub(crate) fn pass_alloc(&mut self, size: u32) -> *mut u8 {
        self.pass_require_open("alloc");
        if self.data.encoder.len() >= PASS_FLUSH_LIMIT {
            self.flush_pass_encoder();
        }
        let (at, ptr) = self.data.ring.alloc(size);
        self.data.encoder.push(PassCmd::SetDraw { at, size });
        ptr
    }

    /// Send everything recorded so far, plus the uniform bytes it
    /// references, to the executor.
    pub(crate) fn flush_pass_encoder(&mut self) {
        if self.data.encoder.is_empty()
            && !self.data.ring.has_pending()
            && !self.data.vertex_ring.has_pending()
        {
            return;
        }
        // Ring memory the executor finished with comes back before the next
        // chunk is needed.
        self.reclaim_chunks();
        let uniforms = self.data.ring.take_pending();
        let vertices = self.data.vertex_ring.take_pending();
        let cmds = self.data.encoder.take();
        let slot = self.data.ring.slot();
        self.send_pass_commands(Box::new(PassCommands {
            slot,
            uniforms,
            vertices,
            cmds,
        }));
    }

    /// Upload the pass's current `ViewBlock` and bind it. Passes of a frame
    /// that share a camera, size and UI transform share one upload.
    fn pass_emit_view(&mut self) {
        let view = self.data.pass.view;
        let frame = self.data.frame_index;
        let at = match self.data.last_view {
            Some((last, at, last_frame))
                if last_frame == frame && last.as_bytes() == view.as_bytes() =>
            {
                at
            }
            _ => {
                let at = self.data.ring.alloc_copy(view.as_bytes());
                self.data.last_view = Some((view, at, frame));
                at
            }
        };
        self.data.encoder.push(PassCmd::SetView {
            block: at,
            size: ViewBlock::SIZE,
        });
    }

    /// Every pass binds the environment: cheap on the executor (its unit
    /// cache skips unchanged binds) and robust against anything else that
    /// touches the group-0 units.
    fn pass_emit_environment(&mut self) {
        let env = &self.data.environment;
        let cmd = PassCmd::SetEnvironment {
            env_map: env.env_map.as_ref().map(|t| t.resource_id()),
            ir_map: env.ir_map.as_ref().map(|t| t.resource_id()),
        };
        self.data.encoder.push(cmd);
    }

    fn pass_set_viewport(&mut self, viewport: [i32; 4]) {
        self.pass_require_open("setViewport");
        let is_window = self.data.pass.is_window;
        self.data.pass.viewport = viewport;
        self.data.pass.view.set_viewport(viewport, is_window);
        self.data.encoder.push(PassCmd::SetViewport(viewport));
        self.pass_emit_view();
        self.sync_scissor();
    }

    fn pass_set_ui_transform(&mut self, m: [f32; 16]) {
        self.pass_require_open("setUiTransform");
        self.data.pass.view.m_world_view_ui = m;
        self.pass_emit_view();
    }

    /// Set the camera every pass that begins from now on (and the open pass)
    /// renders with: view and projection matrices and the direction to the
    /// primary light. Rendering is camera-relative, so the eye stays at the
    /// origin.
    pub fn set_camera_intern(&mut self, view: &Matrix, proj: &Matrix, star_dir: Vec3) {
        let camera = &mut self.data.camera;
        camera.view = glam::Mat4::from_cols_array(&view.to_cols_array());
        camera.proj = glam::Mat4::from_cols_array(&proj.to_cols_array());
        camera.star_dir = star_dir;
        if self.data.pass.open.is_some() {
            let pass = &mut self.data.pass;
            let rebuilt = ViewBlock::new(&self.data.camera, pass.viewport, pass.is_window);
            let ui = pass.view.m_world_view_ui;
            pass.view = ViewBlock {
                m_world_view_ui: ui,
                ..rebuilt
            };
            self.pass_emit_view();
        }
    }

    /// Set the environment cube maps (`envMap`, `irMap`; group 0) for the
    /// passes that begin from now on and the open pass.
    pub fn set_environment_intern(
        &mut self,
        env_map: &crate::render::TexCube,
        ir_map: &crate::render::TexCube,
    ) {
        self.data.environment.env_map = Some(env_map.clone());
        self.data.environment.ir_map = Some(ir_map.clone());
        if self.data.pass.open.is_some() {
            self.pass_emit_environment();
        }
    }

    /// Re-apply the `ClipRect` scissor to the open pass if it differs from
    /// what the GPU currently has. The GL scissor is global state, so this
    /// compares against the last update sent, not against the pass.
    pub fn sync_scissor(&mut self) {
        if self.data.pass.open.is_none() {
            return;
        }
        let size = self.target_size();
        let want = self.data.clip_rect.desired(size);
        if self.data.clip_emitted == Some(want) {
            return;
        }
        self.data.clip_emitted = Some(want);
        match want {
            ScissorUpdate::Disable => self.enable_scissor_intern(false),
            ScissorUpdate::Set {
                x,
                y,
                width,
                height,
            } => {
                self.enable_scissor_intern(true);
                self.set_scissor_intern(x, y, width, height);
            }
        }
    }

    /// Frame boundary: the uniform ring moves on to the next slot.
    pub(crate) fn pass_end_frame(&mut self) {
        if let Some(open) = &self.data.pass.open {
            panic!("end of frame with render pass '{open}' still open");
        }
        self.flush_pass_encoder();
        self.data.frame_index += 1;
        let frame = self.data.frame_index;
        self.data.ring.begin_frame(frame);
        self.data.vertex_ring.begin_frame(frame);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::CHUNK_SIZE;

    /// The executor hands ring memory back after uploading it, in both
    /// backends: a chunk that filled up comes back whole and is reused.
    #[test]
    fn ring_chunks_come_back_from_the_executor() {
        let mut r = Renderer::new_headless();
        r.data.ring.alloc(CHUNK_SIZE as u32 - 256);
        r.data.ring.alloc(1024); // fills chunk 0, opens chunk 1
        r.data.vertex_ring.alloc_copy(&[1u8; 64]);
        r.flush_pass_encoder();

        // A blocking round trip: the executor has processed the commands.
        assert!(r.sync());
        r.reclaim_chunks();
        assert_eq!(r.data.ring.spare_chunks(), 1, "the full uniform chunk");
        assert_eq!(
            r.data.vertex_ring.spare_chunks(),
            0,
            "a small run is not a chunk"
        );

        // The next chunk that fills up reuses the returned memory.
        r.data.ring.alloc(CHUNK_SIZE as u32 - 256);
        r.data.ring.alloc(1024);
        assert_eq!(r.data.ring.spare_chunks(), 0);
    }
}
