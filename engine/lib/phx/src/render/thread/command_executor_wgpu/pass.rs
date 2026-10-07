//! Render passes, pipelines and draws of the wgpu executor.
//!
//! `BeginRenderPass` opens a `wgpu::RenderPass` on the frame's encoder with
//! the load ops of the description; `PassCommands` record into it; and
//! `EndRenderPass` closes it. State the commands set (pipeline, bind group
//! contents, viewport, scissor) lives in [`PassRt`] and is applied lazily at
//! the draw, so a pass that a flush point interrupted can be reopened (with
//! `LoadOp::Load`) and carry on.

use super::bind::{GroupState, is_dynamic_block};
use super::*;
use crate::render::{MAX_COLOR_ATTACHMENTS, Samplers, StoreOp, face_layer};

/// Bytes of one entry of an instance index list (`uint`).
const INSTANCE_INDEX_SIZE: u64 = 4;

/// Cache key of a texture view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct ViewKey {
    pub tex: u64,
    pub generation: u32,
    /// 0 sampling D1, 1 D2, 2 D3, 3 Cube, 4 face (D2), 5 attachment of a slice (D3).
    pub kind: u8,
    pub base_mip: u8,
    pub mip_count: u8,
    pub layer: u16,
}

/// The vertex layout a draw feeds a pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Flavor {
    /// An indexed mesh: `(stride, attribute flags)`.
    Mesh(u32, u8),
    /// The mesh plus `InstanceData` per instance.
    MeshInstanced(u32, u8),
    /// The mesh plus an instance index list.
    MeshIndices(u32, u8),
    Fullscreen,
    Imm2D,
    Imm3D,
}

impl Flavor {
    /// Vertex buffer slot of the constant-attribute defaults.
    fn defaults_slot(self) -> u32 {
        match self {
            Flavor::MeshInstanced(..) | Flavor::MeshIndices(..) => 2,
            _ => 1,
        }
    }
}

fn format_flags(format: &VertexFormat) -> u8 {
    format.has_position as u8
        | (format.has_normal as u8) << 1
        | (format.has_uv as u8) << 2
        | (format.has_color as u8) << 3
}

/// A pipeline variant: the `PipelineId` made for one pass signature and one
/// vertex layout.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct VariantKey {
    pub pipeline: PipelineId,
    pub shader_generation: u32,
    pub colors: [Option<wgpu::TextureFormat>; MAX_COLOR_ATTACHMENTS],
    pub depth: Option<wgpu::TextureFormat>,
    pub flavor: Flavor,
}

/// The unit quad `DrawFullscreen` draws, an `ImmVertex` layout like the one GL
/// uses, as two triangles (the fan of the GL path).
pub(super) struct Quad {
    pub vertices: wgpu::Buffer,
    pub indices: wgpu::Buffer,
}

/// One attachment of the open pass.
pub(super) struct AttachRt {
    pub view: wgpu::TextureView,
    pub depth_slice: Option<u32>,
    pub format: wgpu::TextureFormat,
    pub load: LoadOp,
    pub clear: [f32; 4],
}

/// The open pass: attachments, the wgpu pass (absent while a flush point has
/// it closed) and the state the commands have set.
pub(super) struct PassRt {
    pub label: Arc<str>,
    pub colors: Vec<AttachRt>,
    pub depth: Option<AttachRt>,
    pub extent: [u32; 2],
    pub rpass: Option<wgpu::RenderPass<'static>>,
    /// The pass has been opened once: reopening loads what it drew.
    pub opened: bool,
    pub groups: [GroupState; 4],
    /// What is set on the wgpu pass per group: bind group serial and dynamic offset.
    pub set: [Option<(u64, u32)>; 4],
    pub pipeline: Option<PipelineId>,
    pub variant: Option<VariantKey>,
    pub layouts: [u32; 4],
    pub viewport: [i32; 4],
    pub scissor: Option<[i32; 4]>,
    pub slot: usize,
    /// The viewport is empty after clamping: draws do nothing (as in GL).
    pub no_viewport: bool,
}

impl PassRt {
    fn formats(
        &self,
    ) -> (
        [Option<wgpu::TextureFormat>; MAX_COLOR_ATTACHMENTS],
        Option<wgpu::TextureFormat>,
    ) {
        let mut colors = [None; MAX_COLOR_ATTACHMENTS];
        for (i, c) in self.colors.iter().enumerate() {
            colors[i] = Some(c.format);
        }
        (colors, self.depth.as_ref().map(|d| d.format))
    }
}

fn to_color(c: [f32; 4]) -> wgpu::Color {
    wgpu::Color {
        r: c[0] as f64,
        g: c[1] as f64,
        b: c[2] as f64,
        a: c[3] as f64,
    }
}

impl WgpuCommandExecutor {
    // =====================================================================
    // Views
    // =====================================================================

    /// A view for sampling `tv` as a texture of the dimension `dim` the shader
    /// declares, honouring the view's mip range, cube face and layer. The key
    /// words identify it for the bind group cache.
    pub(super) fn sampling_view(
        &mut self,
        tv: &TexView,
        dim: crate::render::TexDim,
    ) -> Option<(wgpu::TextureView, [u64; 6])> {
        use crate::render::TexDim;
        let (texture, desc) = self.texture_and_desc(tv.tex)?;
        // One face of a cube can be sampled as a 2D texture.
        let face = match (desc.dim, dim, tv.dim) {
            (TexDim::Cube, TexDim::D2, ViewDim::CubeFace(face)) => Some(face_layer(face)),
            _ => None,
        };
        if desc.dim != dim && face.is_none() {
            self.warn_once(format!(
                "texture {:?} is a {:?} texture but is bound to a {:?} sampler",
                tv.tex, desc.dim, dim
            ));
            return None;
        }
        let levels = texture.mip_level_count();
        let base = (tv.base_mip as u32).min(levels - 1);
        let count = if tv.mip_count == 0 {
            levels - base
        } else {
            (tv.mip_count as u32).min(levels - base)
        };
        let (view_dimension, layer, layers, kind) = match (dim, face) {
            (TexDim::D2, Some(face)) => (wgpu::TextureViewDimension::D2, face, Some(1), 4u8),
            (TexDim::Cube, _) => (wgpu::TextureViewDimension::Cube, 0, None, 3),
            (TexDim::D1, _) => (wgpu::TextureViewDimension::D1, 0, None, 0),
            (TexDim::D3, _) => (wgpu::TextureViewDimension::D3, 0, None, 2),
            (TexDim::D2, None) => (wgpu::TextureViewDimension::D2, 0, None, 1),
        };
        let key = ViewKey {
            tex: tv.tex.0,
            generation: self.generation_of(tv.tex),
            kind,
            base_mip: base as u8,
            mip_count: count as u8,
            layer: layer as u16,
        };
        let view = match self.views.get(&key) {
            Some(v) => v.clone(),
            None => {
                let v = texture.create_view(&wgpu::TextureViewDescriptor {
                    label: Some("phx-sampling-view"),
                    dimension: Some(view_dimension),
                    base_mip_level: base,
                    mip_level_count: Some(count),
                    base_array_layer: layer,
                    array_layer_count: layers,
                    ..Default::default()
                });
                self.views.insert(key, v.clone());
                v
            }
        };
        Some((
            view,
            [
                4,
                key.tex,
                key.generation as u64,
                key.kind as u64,
                (key.base_mip as u64) << 8 | key.mip_count as u64,
                key.layer as u64,
            ],
        ))
    }

    /// The view `tv` renders to (one mip level, one cube face or volume
    /// slice) and the volume slice to pass as `depth_slice`.
    fn attachment_view(
        &mut self,
        tv: &TexView,
    ) -> Option<(wgpu::TextureView, Option<u32>, wgpu::TextureFormat)> {
        use crate::render::TexDim;
        let (texture, desc) = self.texture_and_desc(tv.tex)?;
        let format = texture.format();
        let base = (tv.base_mip as u32).min(texture.mip_level_count() - 1);
        let (dimension, layer, depth_slice, kind) = match (desc.dim, tv.dim) {
            (TexDim::D2, _) => (wgpu::TextureViewDimension::D2, 0, None, 1u8),
            (TexDim::Cube, ViewDim::CubeFace(face)) => {
                (wgpu::TextureViewDimension::D2, face_layer(face), None, 4)
            }
            (TexDim::D3, ViewDim::D2Layer(z)) => {
                (wgpu::TextureViewDimension::D3, 0, Some(z as u32), 5)
            }
            (dim, view) => {
                self.warn_once(format!(
                    "{:?} of a {dim:?} texture is not an attachable view",
                    view
                ));
                return None;
            }
        };
        let key = ViewKey {
            tex: tv.tex.0,
            generation: self.generation_of(tv.tex),
            kind,
            base_mip: base as u8,
            mip_count: 1,
            layer: layer as u16,
        };
        let view = match self.views.get(&key) {
            Some(v) => v.clone(),
            None => {
                let v = texture.create_view(&wgpu::TextureViewDescriptor {
                    label: Some("phx-attachment-view"),
                    dimension: Some(dimension),
                    base_mip_level: base,
                    mip_level_count: Some(1),
                    base_array_layer: layer,
                    array_layer_count: if desc.dim == TexDim::D3 {
                        None
                    } else {
                        Some(1)
                    },
                    ..Default::default()
                });
                self.views.insert(key, v.clone());
                v
            }
        };
        Some((view, depth_slice, format))
    }

    // =====================================================================
    // Pass begin / end
    // =====================================================================

    pub(super) fn cmd_begin_render_pass(&mut self, desc: &RenderPassDesc) {
        self.pass = None;
        let mut colors = Vec::new();
        let mut depth = None;
        let extent;
        if desc.backbuffer {
            self.ensure_backbuffer();
            let bb = self.backbuffer.as_ref().expect("backbuffer");
            extent = [bb.size.0, bb.size.1];
            colors.push(AttachRt {
                view: bb.color_view.clone(),
                depth_slice: None,
                format: super::frame::BACKBUFFER_FORMAT,
                load: desc.back_color.0,
                clear: desc.back_color.1,
            });
            depth = Some(AttachRt {
                view: bb.depth_view.clone(),
                depth_slice: None,
                format: super::frame::BACKBUFFER_DEPTH_FORMAT,
                load: desc.back_depth.0,
                clear: [desc.back_depth.1, 0.0, 0.0, 0.0],
            });
        } else {
            extent = desc.extent;
            for attachment in desc.color.iter().flatten() {
                let Some((view, depth_slice, format)) = self.attachment_view(&attachment.view)
                else {
                    warn!(
                        "wgpu: pass '{}' has a color attachment that is not available",
                        desc.label
                    );
                    return;
                };
                colors.push(AttachRt {
                    view,
                    depth_slice,
                    format,
                    load: attachment.load,
                    clear: attachment.clear,
                });
            }
            if let Some(d) = &desc.depth {
                let Some((view, _, format)) = self.attachment_view(&d.view) else {
                    warn!(
                        "wgpu: pass '{}' has a depth attachment that is not available",
                        desc.label
                    );
                    return;
                };
                depth = Some(AttachRt {
                    view,
                    depth_slice: None,
                    format,
                    load: d.load,
                    clear: [d.clear, 0.0, 0.0, 0.0],
                });
            }
            // A store op of `Discard` is informational (GL 3.3 cannot discard):
            // later passes may sample the attachment.
            let _ = StoreOp::Discard;
        }
        let mut pass = PassRt {
            label: desc.label.clone(),
            colors,
            depth,
            extent,
            rpass: None,
            opened: false,
            groups: std::array::from_fn(|_| GroupState::new()),
            set: [None; 4],
            pipeline: None,
            variant: None,
            layouts: [0; 4],
            viewport: [0, 0, extent[0] as i32, extent[1] as i32],
            scissor: None,
            slot: self.frame_slot,
            no_viewport: false,
        };
        self.open_rpass(&mut pass);
        self.pass = Some(pass);
    }

    pub(super) fn cmd_end_render_pass(&mut self) {
        // Dropping the pass ends it; the encoder stays open for the frame.
        self.pass = None;
    }

    /// Open (or reopen) the wgpu pass of `pass`. The first opening applies the
    /// load ops of the description; later ones keep what the pass drew.
    pub(super) fn open_rpass(&mut self, pass: &mut PassRt) {
        let first = !pass.opened;
        let color_ops = |a: &AttachRt| wgpu::Operations {
            load: if first && a.load == LoadOp::Clear {
                wgpu::LoadOp::Clear(to_color(a.clear))
            } else {
                // `DontCare` keeps the old contents, like GL.
                wgpu::LoadOp::Load
            },
            store: wgpu::StoreOp::Store,
        };
        let color_attachments: Vec<Option<wgpu::RenderPassColorAttachment>> = pass
            .colors
            .iter()
            .map(|a| {
                Some(wgpu::RenderPassColorAttachment {
                    view: &a.view,
                    depth_slice: a.depth_slice,
                    resolve_target: None,
                    ops: color_ops(a),
                })
            })
            .collect();
        let depth_stencil_attachment =
            pass.depth
                .as_ref()
                .map(|d| wgpu::RenderPassDepthStencilAttachment {
                    view: &d.view,
                    depth_ops: Some(wgpu::Operations {
                        load: if first && d.load == LoadOp::Clear {
                            wgpu::LoadOp::Clear(d.clear[0])
                        } else {
                            wgpu::LoadOp::Load
                        },
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                });
        let encoder = self.encoder();
        let rpass = encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&pass.label),
                color_attachments: &color_attachments,
                depth_stencil_attachment,
                ..Default::default()
            })
            .forget_lifetime();
        pass.rpass = Some(rpass);
        pass.opened = true;
        // A new wgpu pass starts with nothing set.
        pass.set = [None; 4];
        pass.variant = None;
        self.recorded = true;
        Self::apply_viewport(pass);
        Self::apply_scissor(pass);
    }

    fn apply_viewport(pass: &mut PassRt) {
        let Some(rpass) = pass.rpass.as_mut() else {
            return;
        };
        let [x, y, w, h] = pass.viewport;
        let [ew, eh] = [pass.extent[0] as i32, pass.extent[1] as i32];
        let (x0, y0) = (x.max(0), y.max(0));
        let (x1, y1) = ((x + w).min(ew), (y + h).min(eh));
        if x1 <= x0 || y1 <= y0 {
            pass.no_viewport = true;
            return;
        }
        pass.no_viewport = false;
        rpass.set_viewport(
            x0 as f32,
            y0 as f32,
            (x1 - x0) as f32,
            (y1 - y0) as f32,
            0.0,
            1.0,
        );
    }

    fn apply_scissor(pass: &mut PassRt) {
        let Some(rpass) = pass.rpass.as_mut() else {
            return;
        };
        let [ew, eh] = [pass.extent[0] as i32, pass.extent[1] as i32];
        let (x0, y0, x1, y1) = match pass.scissor {
            Some([x, y, w, h]) => (x.max(0), y.max(0), (x + w).min(ew), (y + h).min(eh)),
            None => (0, 0, ew, eh),
        };
        let (w, h) = ((x1 - x0).max(0), (y1 - y0).max(0));
        rpass.set_scissor_rect(x0.min(ew) as u32, y0.min(eh) as u32, w as u32, h as u32);
    }

    // =====================================================================
    // Pass commands
    // =====================================================================

    pub(super) fn cmd_pass_commands(&mut self, commands: &mut PassCommands) {
        let slot = commands.slot as usize % crate::render::MAX_FRAMES_IN_FLIGHT;
        // Uploads first: everything the commands reference is in these.
        self.upload_ring(slot, &mut commands.uniforms, false);
        self.upload_ring(slot, &mut commands.vertices, true);
        let Some(mut pass) = self.pass.take() else {
            self.warn_once("PassCommands outside a render pass".into());
            return;
        };
        pass.slot = slot;
        for cmd in &commands.cmds {
            self.run_pass_cmd(&mut pass, cmd);
        }
        self.pass = Some(pass);
    }

    fn run_pass_cmd(&mut self, pass: &mut PassRt, cmd: &PassCmd) {
        match cmd {
            PassCmd::SetPipeline(id) => {
                pass.pipeline = Some(*id);
                self.frame_counters.state_changes += 1;
            }
            PassCmd::SetBindGroup { group, id } => {
                let Some(stored) = self.bind_groups.get(id) else {
                    warn!("wgpu: SetBindGroup: bind group {id:?} was never created");
                    return;
                };
                debug_assert_eq!(stored.group, *group);
                if let Some(gs) = pass.groups.get_mut(*group as usize) {
                    gs.set_entries(&stored.entries);
                }
            }
            PassCmd::SetView { block, .. } => {
                pass.groups[0].set_ring(pass.slot as u8, block.buffer, block.offset);
            }
            PassCmd::SetEnvironment { env_map, ir_map } => {
                let sampler = Samplers::LinearMipClamp.id();
                for (index, id) in [(0usize, env_map), (1, ir_map)] {
                    let value = id.map(|id| (TexView::full(id, ViewDim::Cube, [1, 1]), sampler));
                    pass.groups[0].set_texture(index, value);
                }
            }
            PassCmd::SetDraw { at, .. } => {
                pass.groups[2].set_ring(pass.slot as u8, at.buffer, at.offset);
            }
            PassCmd::SetInputs(inputs) => {
                for (i, input) in inputs.iter().enumerate() {
                    pass.groups[3].set_texture(i, *input);
                }
            }
            PassCmd::SetViewport(v) => {
                pass.viewport = *v;
                Self::apply_viewport(pass);
            }
            PassCmd::SetScissor(rect) => {
                pass.scissor = *rect;
                Self::apply_scissor(pass);
            }
            PassCmd::DrawMesh {
                mesh,
                first_index,
                index_count,
            } => {
                let Some(WgpuResource::Mesh(m)) = self.resources.get(mesh) else {
                    warn!("wgpu: DrawMesh: mesh {mesh:?} not found");
                    return;
                };
                let (vb, ib) = (m.vertex_buffer.clone(), m.index_buffer.clone());
                let flavor = Flavor::Mesh(m.vertex_format.stride, format_flags(&m.vertex_format));
                if !self.prepare_draw(pass, flavor) {
                    return;
                }
                let defaults = self.defaults().vertex_zero.clone();
                let rp = pass.rpass.as_mut().expect("prepared");
                rp.set_vertex_buffer(0, vb.slice(..));
                rp.set_vertex_buffer(flavor.defaults_slot(), defaults.slice(..32));
                rp.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                rp.draw_indexed(*first_index..first_index + index_count, 0, 0..1);
                self.frame_counters.draw_mesh += 1;
                self.frame_counters.vertices += *index_count as u64;
                self.count_draw();
            }
            PassCmd::DrawFullscreen => {
                if !self.prepare_draw(pass, Flavor::Fullscreen) {
                    return;
                }
                self.ensure_quad();
                let quad = self.quad.as_ref().expect("quad");
                let defaults = self
                    .defaults
                    .as_ref()
                    .expect("defaults")
                    .vertex_zero
                    .clone();
                let rp = pass.rpass.as_mut().expect("prepared");
                rp.set_vertex_buffer(0, quad.vertices.slice(..));
                rp.set_vertex_buffer(1, defaults.slice(..32));
                rp.set_index_buffer(quad.indices.slice(..), wgpu::IndexFormat::Uint32);
                rp.draw_indexed(0..6, 0, 0..1);
                self.frame_counters.draw_imm += 1;
                self.frame_counters.vertices += 4;
                self.count_draw();
            }
            PassCmd::DrawImm {
                layout,
                vertices,
                count,
            } => {
                let flavor = match layout {
                    ImmLayout::D2 => Flavor::Imm2D,
                    ImmLayout::D3 => Flavor::Imm3D,
                };
                let Some(buffer) = self.ring_vertex[pass.slot]
                    .get(vertices.buffer as usize)
                    .cloned()
                else {
                    warn!(
                        "wgpu: DrawImm: vertex ring chunk {} is missing",
                        vertices.buffer
                    );
                    return;
                };
                if !self.prepare_draw(pass, flavor) {
                    return;
                }
                let defaults = self
                    .defaults
                    .as_ref()
                    .expect("defaults")
                    .vertex_zero
                    .clone();
                let rp = pass.rpass.as_mut().expect("prepared");
                let start = vertices.offset as u64;
                let end = start + *count as u64 * layout.stride() as u64;
                rp.set_vertex_buffer(0, buffer.slice(start..end));
                rp.set_vertex_buffer(1, defaults.slice(..32));
                rp.draw(0..*count, 0..1);
                self.frame_counters.draw_imm += 1;
                self.frame_counters.imm_vertices += *count as u64;
                self.frame_counters.vertices += *count as u64;
                self.count_draw();
            }
            PassCmd::DrawMeshInstanced {
                mesh,
                index_count,
                instances,
                count,
            } => {
                let Some(WgpuResource::Mesh(m)) = self.resources.get(mesh) else {
                    warn!("wgpu: DrawMeshInstanced: mesh {mesh:?} not found");
                    return;
                };
                let (vb, ib) = (m.vertex_buffer.clone(), m.index_buffer.clone());
                let flavor =
                    Flavor::MeshInstanced(m.vertex_format.stride, format_flags(&m.vertex_format));
                let Some(buffer) = self.ring_vertex[pass.slot]
                    .get(instances.buffer as usize)
                    .cloned()
                else {
                    warn!(
                        "wgpu: DrawMeshInstanced: vertex ring chunk {} is missing",
                        instances.buffer
                    );
                    return;
                };
                if !self.prepare_draw(pass, flavor) {
                    return;
                }
                let defaults = self
                    .defaults
                    .as_ref()
                    .expect("defaults")
                    .vertex_zero
                    .clone();
                let rp = pass.rpass.as_mut().expect("prepared");
                let start = instances.offset as u64;
                let end = start + *count as u64 * std::mem::size_of::<InstanceData>() as u64;
                rp.set_vertex_buffer(0, vb.slice(..));
                rp.set_vertex_buffer(1, buffer.slice(start..end));
                rp.set_vertex_buffer(flavor.defaults_slot(), defaults.slice(..32));
                rp.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                rp.draw_indexed(0..*index_count, 0, 0..*count);
                self.frame_counters.draw_instanced += 1;
                self.frame_counters.instance_items += *count as u64;
                self.frame_counters.vertices += *index_count as u64 * *count as u64;
                self.count_draw();
            }
            PassCmd::DrawInstancedIndices {
                mesh,
                index_count,
                indices,
                count,
            } => {
                let Some(WgpuResource::Mesh(m)) = self.resources.get(mesh) else {
                    warn!("wgpu: DrawInstancedIndices: mesh {mesh:?} not found");
                    return;
                };
                let (vb, ib) = (m.vertex_buffer.clone(), m.index_buffer.clone());
                let flavor =
                    Flavor::MeshIndices(m.vertex_format.stride, format_flags(&m.vertex_format));
                let Some(buffer) = self.ring_vertex[pass.slot]
                    .get(indices.buffer as usize)
                    .cloned()
                else {
                    warn!(
                        "wgpu: DrawInstancedIndices: vertex ring chunk {} is missing",
                        indices.buffer
                    );
                    return;
                };
                if !self.prepare_draw(pass, flavor) {
                    return;
                }
                let defaults = self
                    .defaults
                    .as_ref()
                    .expect("defaults")
                    .vertex_zero
                    .clone();
                let rp = pass.rpass.as_mut().expect("prepared");
                let start = indices.offset as u64;
                let end = start + *count as u64 * INSTANCE_INDEX_SIZE;
                rp.set_vertex_buffer(0, vb.slice(..));
                rp.set_vertex_buffer(1, buffer.slice(start..end));
                rp.set_vertex_buffer(flavor.defaults_slot(), defaults.slice(..32));
                rp.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                rp.draw_indexed(0..*index_count, 0, 0..*count);
                self.frame_counters.draw_instanced += 1;
                self.frame_counters.instance_items += *count as u64;
                self.frame_counters.vertices += *index_count as u64 * *count as u64;
                self.count_draw();
            }
        }
    }

    fn count_draw(&mut self) {
        self.stats.draw_calls += 1;
    }

    fn ensure_quad(&mut self) {
        if self.quad.is_some() {
            return;
        }
        let vertex = |x: f32, y: f32| -> [f32; 12] {
            // position, normal, uv, color
            [x, y, 0.0, 0.0, 0.0, 0.0, x, y, 1.0, 1.0, 1.0, 1.0]
        };
        let mut bytes: Vec<u8> = Vec::new();
        for v in [
            vertex(0.0, 0.0),
            vertex(0.0, 1.0),
            vertex(1.0, 1.0),
            vertex(1.0, 0.0),
        ] {
            bytes.extend(v.iter().flat_map(|f| f.to_le_bytes()));
        }
        let vertices = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("phx-quad-vb"),
            size: bytes.len() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(&vertices, 0, &bytes);
        // The fan (0, 1, 2, 3) as two triangles.
        let index_bytes: Vec<u8> = [0u32, 1, 2, 0, 2, 3]
            .iter()
            .flat_map(|i| i.to_le_bytes())
            .collect();
        let indices = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("phx-quad-ib"),
            size: index_bytes.len() as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(&indices, 0, &index_bytes);
        self.quad = Some(Quad { vertices, indices });
    }

    // =====================================================================
    // Draw preparation: pipeline variant, bind groups
    // =====================================================================

    /// Everything a draw needs before its draw call: the wgpu pass open, the
    /// pipeline variant for this pass and vertex layout set, and the four
    /// bind groups current. `false` if the draw cannot happen (and is
    /// skipped).
    fn prepare_draw(&mut self, pass: &mut PassRt, flavor: Flavor) -> bool {
        let Some(pipeline) = pass.pipeline else {
            self.warn_once(format!("pass '{}': a draw without a pipeline", pass.label));
            return false;
        };
        let Some(desc) = self.pipeline_descs.get(&pipeline) else {
            warn!("wgpu: pipeline {pipeline:?} was never created");
            return false;
        };
        let (colors, depth) = pass.formats();
        let key = VariantKey {
            pipeline,
            shader_generation: self.generation_of(desc.shader),
            colors,
            depth,
            flavor,
        };
        let shader_id = desc.shader;
        let Some(variant) = self.variant(&key) else {
            return false;
        };
        let groups = match self.resources.get(&shader_id) {
            Some(WgpuResource::Shader(s)) => s.groups.clone(),
            _ => return false,
        };

        if pass.rpass.is_none() {
            self.open_rpass(pass);
        }
        if pass.no_viewport {
            return false;
        }

        if pass.variant.as_ref() != Some(&key) {
            // Bind groups below the first group whose layout changed stay valid.
            let first = (0..4)
                .find(|&g| pass.layouts[g] != groups[g].id)
                .unwrap_or(4);
            for g in first..4 {
                pass.set[g] = None;
            }
            pass.layouts = std::array::from_fn(|g| groups[g].id);
            pass.rpass.as_mut().expect("open").set_pipeline(&variant);
            pass.variant = Some(key);
            self.frame_counters.pipeline_binds += 1;
        } else {
            self.frame_counters.pipeline_redundant += 1;
        }

        for g in 0..4u8 {
            let layout = &groups[g as usize];
            let gi = g as usize;
            if pass.groups[gi].dirty
                || pass.groups[gi].bound_layout != layout.id
                || pass.groups[gi].bound.is_none()
            {
                let bound = self.resolve_group(g, &pass.groups[gi], layout);
                let gs = &mut pass.groups[gi];
                gs.bound = Some(bound);
                gs.bound_layout = layout.id;
                gs.dirty = false;
            }
            let gs = &pass.groups[gi];
            let bound = gs.bound.as_ref().expect("resolved");
            let offset = if layout.has_dynamic() && gs.ring.is_some() {
                gs.dyn_offset
            } else {
                0
            };
            if pass.set[gi] != Some((bound.serial, offset)) {
                let offsets: &[u32] = if layout.has_dynamic() {
                    &[offset][..]
                } else {
                    &[]
                };
                pass.rpass
                    .as_mut()
                    .expect("open")
                    .set_bind_group(g as u32, &bound.bg, offsets);
                pass.set[gi] = Some((bound.serial, offset));
            }
        }
        let _ = is_dynamic_block;
        true
    }

    /// The pipeline for `key`, built on first use.
    fn variant(&mut self, key: &VariantKey) -> Option<wgpu::RenderPipeline> {
        if let Some(p) = self.variants.get(key) {
            return Some(p.clone());
        }
        let desc = self.pipeline_descs.get(&key.pipeline)?.clone();
        let pipeline = self.build_variant(key, &desc)?;
        self.variants.insert(key.clone(), pipeline.clone());
        Some(pipeline)
    }

    /// The fragment module of `shader` for targets of `colors`: outputs that
    /// go to a fixed-point format are clamped to 0..1 like GL does before
    /// blending (see `shader::wrap_fragment_clamp`).
    fn fragment_module(
        &mut self,
        shader_id: ResourceId,
        colors: &[Option<wgpu::TextureFormat>; MAX_COLOR_ATTACHMENTS],
    ) -> Option<wgpu::ShaderModule> {
        let mut mask = 0u32;
        for (i, format) in colors.iter().enumerate() {
            if format.is_some_and(|f| !is_float_format(f)) {
                mask |= 1 << i;
            }
        }
        let shader = match self.resources.get(&shader_id) {
            Some(WgpuResource::Shader(s)) => s,
            _ => return None,
        };
        let mask = shader
            .fs_named_outputs
            .iter()
            .fold(0, |m, o| m | (mask & (1 << o.location)));
        if mask == 0 {
            return Some(shader.fs.clone());
        }
        let key = (shader_id, self.generation_of(shader_id), mask);
        if let Some(module) = self.fs_clamped.get(&key) {
            return Some(module.clone());
        }
        let wrapped = shader::wrap_fragment_clamp(&shader.fs_code, &shader.fs_named_outputs, mask)
            .and_then(|code| shader::parse_stage(wgpu::naga::ShaderStage::Fragment, &code));
        let module = match wrapped {
            Ok(module) => self.shader_module("phx-fragment-clamped", module),
            Err(e) => {
                error!("wgpu: clamped fragment variant of {shader_id:?} failed: {e}");
                return None;
            }
        };
        self.fs_clamped.insert(key, module.clone());
        Some(module)
    }

    fn build_variant(
        &mut self,
        key: &VariantKey,
        desc: &PipelineDesc,
    ) -> Option<wgpu::RenderPipeline> {
        let fs_module = self.fragment_module(desc.shader, &key.colors)?;
        let shader = match self.resources.get(&desc.shader) {
            Some(WgpuResource::Shader(s)) => s,
            _ => {
                warn!(
                    "wgpu: pipeline {:?}: shader {:?} not found",
                    key.pipeline, desc.shader
                );
                return None;
            }
        };

        // Vertex buffers of the flavor, then the defaults for every input
        // nothing supplies.
        struct Vb {
            stride: u64,
            step: wgpu::VertexStepMode,
            attrs: Vec<wgpu::VertexAttribute>,
        }
        let mesh_attrs = |stride: u32, flags: u8| {
            let format = VertexFormat {
                has_position: flags & 1 != 0,
                has_normal: flags & 2 != 0,
                has_uv: flags & 4 != 0,
                has_color: flags & 8 != 0,
                stride,
            };
            Vb {
                stride: stride as u64,
                step: wgpu::VertexStepMode::Vertex,
                attrs: mesh_attributes(&format),
            }
        };
        let mut buffers: Vec<Vb> = match key.flavor {
            Flavor::Mesh(stride, flags) => vec![mesh_attrs(stride, flags)],
            Flavor::MeshInstanced(stride, flags) => vec![
                mesh_attrs(stride, flags),
                Vb {
                    stride: std::mem::size_of::<InstanceData>() as u64,
                    step: wgpu::VertexStepMode::Instance,
                    attrs: INSTANCE_ATTRIBUTES.to_vec(),
                },
            ],
            Flavor::MeshIndices(stride, flags) => vec![
                mesh_attrs(stride, flags),
                Vb {
                    stride: INSTANCE_INDEX_SIZE,
                    step: wgpu::VertexStepMode::Instance,
                    attrs: INSTANCE_INDEX_ATTRIBUTES.to_vec(),
                },
            ],
            Flavor::Fullscreen => vec![Vb {
                stride: std::mem::size_of::<crate::render::ImmVertex>() as u64,
                step: wgpu::VertexStepMode::Vertex,
                attrs: QUAD_ATTRIBUTES.to_vec(),
            }],
            Flavor::Imm2D => vec![Vb {
                stride: ImmLayout::D2.stride() as u64,
                step: wgpu::VertexStepMode::Vertex,
                attrs: IMM2D_ATTRIBUTES.to_vec(),
            }],
            Flavor::Imm3D => vec![Vb {
                stride: ImmLayout::D3.stride() as u64,
                step: wgpu::VertexStepMode::Vertex,
                attrs: IMM3D_ATTRIBUTES.to_vec(),
            }],
        };
        let mut defaults = Vec::new();
        for input in &shader.vs_inputs {
            let supplied = buffers
                .iter()
                .any(|b| b.attrs.iter().any(|a| a.shader_location == input.location));
            if !supplied {
                let integer = matches!(
                    input.default,
                    wgpu::VertexFormat::Uint32
                        | wgpu::VertexFormat::Uint32x2
                        | wgpu::VertexFormat::Uint32x3
                        | wgpu::VertexFormat::Uint32x4
                        | wgpu::VertexFormat::Sint32
                        | wgpu::VertexFormat::Sint32x2
                        | wgpu::VertexFormat::Sint32x3
                        | wgpu::VertexFormat::Sint32x4
                );
                // The shared 32-byte zero buffer: `(0, 0, 0, 0)` (GL's default
                // attribute is `(0, 0, 0, 1)`; nothing reads the w of one).
                defaults.push(wgpu::VertexAttribute {
                    format: input.default,
                    offset: if integer { 16 } else { 0 },
                    shader_location: input.location,
                });
            }
        }
        buffers.push(Vb {
            stride: 0,
            step: wgpu::VertexStepMode::Vertex,
            attrs: defaults,
        });
        let layouts: Vec<Option<wgpu::VertexBufferLayout>> = buffers
            .iter()
            .map(|b| {
                Some(wgpu::VertexBufferLayout {
                    array_stride: b.stride,
                    step_mode: b.step,
                    attributes: &b.attrs,
                })
            })
            .collect();

        // Color targets: one per attachment of the pass. A target the shader
        // does not write masks every channel (GL leaves it undefined).
        let can_blend_f32 = self.features.contains(wgpu::Features::FLOAT32_BLENDABLE);
        let blend = blend_state(desc.blend);
        let targets: Vec<Option<wgpu::ColorTargetState>> = key
            .colors
            .iter()
            .enumerate()
            .filter_map(|(i, format)| format.map(|f| (i, f)))
            .map(|(i, format)| {
                Some(wgpu::ColorTargetState {
                    format,
                    blend: if is_float32(format) && !can_blend_f32 {
                        None
                    } else {
                        blend
                    },
                    write_mask: if shader.fs_outputs.contains(&(i as u32)) {
                        wgpu::ColorWrites::ALL
                    } else {
                        wgpu::ColorWrites::empty()
                    },
                })
            })
            .collect();

        // GL: with the depth test off nothing is written either.
        let depth_stencil = key.depth.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: Some(desc.depth.test && desc.depth.write),
            depth_compare: Some(if desc.depth.test {
                compare_function(desc.depth.compare)
            } else {
                wgpu::CompareFunction::Always
            }),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        });

        let strip = matches!(desc.topology, Topology::LineStrip | Topology::TriangleStrip);
        let polygon_mode = if desc.polygon == crate::render::PolygonMode::Line
            && self.features.contains(wgpu::Features::POLYGON_MODE_LINE)
        {
            wgpu::PolygonMode::Line
        } else {
            wgpu::PolygonMode::Fill
        };
        let pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("phx-pipeline"),
                layout: Some(&shader.pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader.vs,
                    entry_point: Some("main"),
                    compilation_options: Default::default(),
                    buffers: &layouts,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &fs_module,
                    entry_point: Some("main"),
                    compilation_options: Default::default(),
                    targets: &targets,
                }),
                primitive: wgpu::PrimitiveState {
                    topology: topology(desc.topology),
                    strip_index_format: strip.then_some(wgpu::IndexFormat::Uint32),
                    // The vertex stage mirrors y (see the module docs), which
                    // swaps the winding.
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: cull_mode(desc.cull),
                    polygon_mode,
                    ..Default::default()
                },
                depth_stencil,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });
        Some(pipeline)
    }
}
