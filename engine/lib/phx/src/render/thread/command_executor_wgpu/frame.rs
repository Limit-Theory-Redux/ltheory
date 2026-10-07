//! Frames of the wgpu executor: the encoder and its flush points, the ring
//! buffers, the backbuffer and presentation, and frame pacing.

use std::sync::atomic::Ordering;

use super::*;
use crate::render::{CHUNK_SIZE, MAX_FRAMES_IN_FLIGHT, RingChunk, VERTEX_CHUNK_SIZE};

/// The window's backbuffer is an ordinary texture in the GL convention (see
/// the module docs of `command_executor_wgpu`); `SwapBuffers` flips it onto
/// the swapchain image.
pub(super) const BACKBUFFER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub(super) const BACKBUFFER_DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub(super) struct Backbuffer {
    pub size: (u32, u32),
    pub color: wgpu::Texture,
    pub color_view: wgpu::TextureView,
    pub depth_view: wgpu::TextureView,
}

/// The pipeline that copies the backbuffer onto the swapchain image, upside
/// down, one texel per pixel.
pub(super) struct PresentBlit {
    format: wgpu::TextureFormat,
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
}

const PRESENT_WGSL: &str = "
@group(0) @binding(0) var src: texture_2d<f32>;
@vertex fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    var p = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
    return vec4<f32>(p[i], 0.0, 1.0);
}
@fragment fn fs(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let size = vec2<i32>(textureDimensions(src));
    let p = vec2<i32>(pos.xy);
    // Row 0 of the backbuffer is the bottom row of the window.
    return textureLoad(src, vec2<i32>(min(p.x, size.x - 1), max(size.y - 1 - p.y, 0)), 0);
}
";

impl WgpuCommandExecutor {
    // =====================================================================
    // Encoder
    // =====================================================================

    /// The frame's encoder, created on first use.
    pub(super) fn encoder(&mut self) -> &mut wgpu::CommandEncoder {
        self.encoder.get_or_insert_with(|| {
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("phx-encoder"),
                })
        })
    }

    /// Close the open wgpu pass (it reopens at the next draw, loading what it
    /// drew) so something can be recorded into the encoder.
    pub(super) fn suspend_pass(&mut self) {
        if let Some(pass) = self.pass.as_mut() {
            pass.rpass = None;
        }
    }

    /// Submit what the encoder holds.
    pub(super) fn submit_frame(&mut self) {
        self.suspend_pass();
        if let Some(encoder) = self.encoder.take() {
            if self.recorded {
                let index = self.queue.submit([encoder.finish()]);
                self.frame_counters.submits += 1;
                self.diag_submitted();
                if self.diag.wait {
                    let _ = self.device.poll(wgpu::PollType::Wait {
                        submission_index: Some(index),
                        timeout: Some(Duration::from_secs(10)),
                    });
                }
            }
        }
        self.recorded = false;
    }

    /// Before a `queue.write_*`: those are ordered before the whole next
    /// submission, so whatever was recorded (and may read the old contents) goes first.
    pub(super) fn flush_for_write(&mut self) {
        if self.recorded {
            self.submit_frame();
        }
    }

    // =====================================================================
    // Rings
    // =====================================================================

    /// Write the ring runs into the slot's buffers (created on first use) and
    /// queue their memory for return to the main thread. No fence is needed:
    /// `write_buffer` is ordered after every earlier submission, and the main
    /// thread never reuses a region within a frame.
    pub(super) fn upload_ring(&mut self, slot: usize, chunks: &mut [RingChunk], vertex: bool) {
        for chunk in chunks.iter_mut() {
            if !vertex {
                self.diag_ring_upload(slot, chunk.at.buffer, chunk.at.offset, chunk.data());
            }
            let extra = self.diag.buffer_usage();
            let (buffers, size, usage) = if vertex {
                (
                    &mut self.ring_vertex[slot],
                    VERTEX_CHUNK_SIZE,
                    wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST | extra,
                )
            } else {
                (
                    &mut self.ring_uniform[slot],
                    CHUNK_SIZE,
                    wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST | extra,
                )
            };
            while buffers.len() <= chunk.at.buffer as usize {
                buffers.push(self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(if vertex {
                        "phx-vertex-ring"
                    } else {
                        "phx-uniform-ring"
                    }),
                    size: size as u64,
                    usage,
                    mapped_at_creation: false,
                }));
            }
            let buffer = &buffers[chunk.at.buffer as usize];
            let data = chunk.data();
            if !data.is_empty() {
                self.queue
                    .write_buffer(buffer, chunk.at.offset as u64, &pad4(data));
            }
            self.returned.push(ReturnedChunk {
                vertex,
                bytes: std::mem::take(&mut chunk.bytes),
            });
        }
    }

    // =====================================================================
    // Backbuffer and surface
    // =====================================================================

    pub(super) fn ensure_backbuffer(&mut self) {
        let size = self.surface_size;
        if self.backbuffer.as_ref().is_some_and(|b| b.size == size) {
            return;
        }
        let extent = wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        };
        let color = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("phx-backbuffer"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: BACKBUFFER_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let depth = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("phx-backbuffer-depth"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: BACKBUFFER_DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        self.backbuffer = Some(Backbuffer {
            size,
            color_view: color.create_view(&Default::default()),
            depth_view: depth.create_view(&Default::default()),
            color,
        });
    }

    fn reconfigure_surface(&mut self) {
        let (Some(surface), Some(config)) = (self.surface.as_ref(), self.surface_config.as_mut())
        else {
            return;
        };
        config.width = self.surface_size.0;
        config.height = self.surface_size.1;
        surface.configure(&self.device, config);
    }

    pub(super) fn cmd_resize(&mut self, width: u32, height: u32) {
        let new_size = (width.max(1), height.max(1));
        if new_size == self.surface_size {
            return;
        }
        self.surface_size = new_size;
        self.reconfigure_surface();
    }

    pub(super) fn cmd_set_present_mode(&mut self, mode: PresentMode) {
        let Some(config) = self.surface_config.as_mut() else {
            return;
        };
        let present_mode: wgpu::PresentMode = mode.into();
        if config.present_mode == present_mode {
            return;
        }
        config.present_mode = present_mode;
        self.reconfigure_surface();
        tracing::info!("wgpu: present mode set to {mode:?}");
    }

    fn present_blit(&mut self, format: wgpu::TextureFormat) -> &PresentBlit {
        if self.present.as_ref().is_none_or(|p| p.format != format) {
            let module = self
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("phx-present"),
                    source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(PRESENT_WGSL)),
                });
            let layout = self
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("phx-present-bgl"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    }],
                });
            let pipeline_layout =
                self.device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("phx-present-pl"),
                        bind_group_layouts: &[Some(&layout)],
                        immediate_size: 0,
                    });
            let pipeline = self
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("phx-present"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &module,
                        entry_point: Some("vs"),
                        compilation_options: Default::default(),
                        buffers: &[],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &module,
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
            self.present = Some(PresentBlit {
                format,
                pipeline,
                layout,
            });
        }
        self.present.as_ref().expect("created above")
    }

    // =====================================================================
    // Frame boundaries
    // =====================================================================

    /// A new frame starts in ring slot `slot`: wait until the GPU finished
    /// the frame that last used it (its `on_submitted_work_done`), and pick up
    /// the readbacks that completed.
    pub(super) fn cmd_begin_frame(&mut self, slot: u8) {
        let slot = slot as usize % MAX_FRAMES_IN_FLIGHT;
        self.frame_slot = slot;
        if let Some(done) = self.slot_done[slot].take() {
            let started = Instant::now();
            while !done.load(Ordering::Acquire) {
                let _ = self.device.poll(wgpu::PollType::Poll);
                if started.elapsed() > Duration::from_secs(5) {
                    error!("wgpu: BeginFrame: slot {slot} was not finished by the GPU within 5 s");
                    break;
                }
                std::thread::yield_now();
            }
        }
        self.poll_readbacks();
        self.diag_poll_checks();
        if self.frame_index % 64 == 0 {
            self.sweep_bind_groups();
        }
    }

    /// Present: submit the frame's work, copy the backbuffer onto the
    /// swapchain image and hand the image over.
    pub(super) fn cmd_swap_buffers(&mut self) -> CommandReply {
        // Render time of the frame: receive plus execute, not the present wait.
        let frame_time_us = self.frame_start.elapsed().as_micros() as u64;
        self.pass = None;
        self.submit_frame();
        self.diag_save_dumps();
        self.ensure_backbuffer();

        let present_started = Instant::now();
        let frame = self
            .surface
            .as_ref()
            .and_then(|surface| match surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(frame)
                | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => Some(frame),
                wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                    Some(None).flatten()
                }
                _ => None,
            });
        // An outdated or lost swapchain is reconfigured and the frame skipped.
        if frame.is_none() && self.surface.is_some() {
            self.reconfigure_surface();
        }
        if let Some(frame) = frame {
            let format = frame.texture.format();
            self.present_blit(format);
            let view = frame.texture.create_view(&Default::default());
            let backbuffer = self.backbuffer.as_ref().expect("backbuffer");
            let blit = self.present.as_ref().expect("blit");
            let group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("phx-present-bg"),
                layout: &blit.layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&backbuffer.color_view),
                }],
            });
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("phx-present-encoder"),
                });
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("phx-present"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    ..Default::default()
                });
                pass.set_pipeline(&blit.pipeline);
                pass.set_bind_group(0, &group, &[]);
                pass.draw(0..3, 0..1);
            }
            self.queue.submit([encoder.finish()]);
            self.queue.present(frame);
        }
        let present_wait_us = present_started.elapsed().as_micros() as u64;

        // The GPU's completion of this frame's work frees its ring slot.
        let done = Arc::new(AtomicBool::new(false));
        let flag = done.clone();
        self.queue
            .on_submitted_work_done(move || flag.store(true, Ordering::Release));
        self.slot_done[self.frame_slot] = Some(done);
        if self.diag.frame_wait {
            let _ = self.device.poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(Duration::from_secs(10)),
            });
        }
        let _ = self.device.poll(wgpu::PollType::Poll);

        self.frame_index += 1;
        self.stats.frame_count += 1;
        let c = std::mem::take(&mut self.frame_counters);
        let (mut textures, mut meshes, mut texture_bytes) = (0u64, 0u64, 0u64);
        for r in self.resources.values() {
            match r {
                WgpuResource::Texture { desc, .. } => {
                    textures += 1;
                    let faces = if matches!(desc.dim, crate::render::TexDim::Cube) { 6 } else { 1 };
                    texture_bytes += faces
                        * (0..desc.mips.max(1)).map(|l| desc.level_bytes(l) as u64).sum::<u64>();
                }
                WgpuResource::Mesh(_) => meshes += 1,
                WgpuResource::Shader(_) => {}
            }
        }
        self.last_stats = RenderStats {
            commands_processed: self.stats.commands_processed,
            draw_calls_cumulative: self.stats.draw_calls,
            state_changes_cumulative: self.stats.state_changes + c.state_changes,
            frame_count: self.stats.frame_count,
            last_frame_time_us: frame_time_us,
            recv_wait_us: c.recv_wait_us,
            recv_wait_count: c.recv_wait_count,
            commands: c.commands,
            draw_calls: c.draw_mesh + c.draw_imm + c.draw_instanced,
            state_changes: c.state_changes,
            present_wait_us,
            draw_mesh_calls: c.draw_mesh,
            draw_immediate_calls: c.draw_imm,
            draw_instanced_calls: c.draw_instanced,
            immediate_vertices: c.imm_vertices,
            instanced_data_items: c.instance_items,
            vertices_drawn: c.vertices,
            shader_bind_commands: c.pipeline_binds + c.pipeline_redundant,
            shader_redundant_binds: c.pipeline_redundant,
            shader_distinct_programs: c.pipeline_binds,
            passes: c.passes,
            bind_group_switches: c.bind_group_switches,
            pipelines_cached: self.variants.len() as u64,
            samplers: self.samplers.len() as u64,
            bind_groups: self.bind_groups.len() as u64,
            textures,
            meshes,
            texture_bytes,
            ..self.last_stats.clone()
        };
        self.stats.state_changes += c.state_changes;
        self.frame_start = Instant::now();
        CommandReply::Stats(Box::new(self.last_stats.clone()))
    }

    /// `Flush` (`Renderer:gpuFinish`): submit and wait for the GPU.
    pub(super) fn cmd_flush(&mut self) {
        self.submit_frame();
        if let Err(error) = self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(Duration::from_secs(5)),
        }) {
            warn!("wgpu GPU finish did not complete: {error:?}");
        }
    }

    /// `PacingFence`: answer once the GPU has finished everything submitted
    /// so far (the render thread polls the device for the callback).
    pub(super) fn cmd_pacing_fence(&mut self, fence_id: u64) -> CommandReply {
        let Some(tx) = self.pacing_tx.clone() else {
            return CommandReply::PacingFence(fence_id);
        };
        self.queue.on_submitted_work_done(move || {
            let _ = tx.send(fence_id);
        });
        CommandReply::None
    }
}
