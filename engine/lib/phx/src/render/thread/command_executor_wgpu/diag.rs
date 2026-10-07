//! Opt-in diagnostics of the wgpu executor, for chasing rendering differences
//! that come and go between runs. Everything here is off unless an
//! environment variable turns it on, and costs nothing then.
//!
//! `LTHEORY_WGPU_DIAG` is a comma-separated list of switches:
//!
//! * `wait`: block until the GPU finished every submission.
//! * `framewait`: block until the GPU finished each frame, at `SwapBuffers`.
//! * `passsubmit`: submit at the end of every render pass.
//! * `nobgcache`: make a new bind group for every resolve (no content cache).
//! * `check`: before every scene-mesh draw that binds a material block, copy
//!   on the GPU the view block, the draw block, the material block and the
//!   mesh's vertices it reads, and compare them with the bytes the CPU
//!   uploaded. A difference is logged as `CHECK MISMATCH`.
//!
//! `LTHEORY_WGPU_DUMP=<frame>` writes the first color attachment (and the
//! depth attachment) of every pass of that frame, raw, into
//! `LTHEORY_WGPU_DUMP_DIR` (default `.`): `f<frame>_<n>_<label>_<w>x<h>_<format>.raw`.
//! Two dumps of the same frame from different runs show the first pass whose
//! output differs.
//!
//! `WGPU_BACKEND` (`vulkan`, `dx12`) picks the backend (`window/wgpu_render.rs`).

use super::pass::PassRt;
use super::*;

#[derive(Default, Clone)]
pub(super) struct Diag {
    pub wait: bool,
    pub frame_wait: bool,
    pub pass_submit: bool,
    pub no_bg_cache: bool,
    pub check: bool,
    pub dump: Option<u64>,
}

impl Diag {
    pub fn from_env() -> Self {
        let list = std::env::var("LTHEORY_WGPU_DIAG").unwrap_or_default();
        let has = |k: &str| list.split(',').any(|s| s.trim() == k);
        let diag = Self {
            wait: has("wait"),
            frame_wait: has("framewait"),
            pass_submit: has("passsubmit"),
            no_bg_cache: has("nobgcache"),
            check: has("check"),
            dump: std::env::var("LTHEORY_WGPU_DUMP")
                .ok()
                .and_then(|f| f.parse().ok()),
        };
        if !list.is_empty() || diag.dump.is_some() {
            warn!("wgpu diagnostics on: '{list}', dump frame {:?}", diag.dump);
        }
        diag
    }

    /// Extra buffer usage the diagnostics copy from.
    pub fn buffer_usage(&self) -> wgpu::BufferUsages {
        if self.check {
            wgpu::BufferUsages::COPY_SRC
        } else {
            wgpu::BufferUsages::empty()
        }
    }

    /// Extra texture usage the diagnostics copy from.
    pub fn texture_usage(&self) -> wgpu::TextureUsages {
        if self.dump.is_some() {
            wgpu::TextureUsages::COPY_SRC
        } else {
            wgpu::TextureUsages::empty()
        }
    }
}

/// A copy of the bytes a draw reads, made on the GPU right before the draw,
/// with the bytes the CPU uploaded for them.
struct Check {
    what: String,
    staging: wgpu::Buffer,
    expected: Vec<u8>,
    done: Arc<Mutex<Option<bool>>>,
}

/// A pass attachment copied for the dump.
struct Dump {
    name: String,
    staging: wgpu::Buffer,
    row_bytes: u32,
    padded_row_bytes: u32,
    rows: u32,
}

/// What the diagnostics keep: CPU copies of uploads, copies in flight.
#[derive(Default)]
pub(super) struct DiagState {
    /// Uniform ring uploads by `(slot, chunk)`.
    ring: HashMap<(usize, u16), Vec<u8>>,
    arenas: HashMap<BufferId, Vec<u8>>,
    meshes: HashMap<ResourceId, Vec<u8>>,
    /// Checks recorded into the open encoder, and submitted ones.
    recorded: Vec<Check>,
    submitted: Vec<Check>,
    compared: u64,
    dumps: Vec<Dump>,
    dump_count: u32,
}

/// Bytes of the `ViewBlock` and the `DrawBlock` (std140).
const VIEW_BLOCK_BYTES: u64 = 448;
const DRAW_BLOCK_BYTES: u64 = 256;

impl WgpuCommandExecutor {
    pub(super) fn diag_ring_upload(&mut self, slot: usize, chunk: u16, offset: u32, data: &[u8]) {
        if !self.diag.check {
            return;
        }
        let shadow = self
            .diag_state
            .ring
            .entry((slot, chunk))
            .or_insert_with(|| vec![0; crate::render::CHUNK_SIZE]);
        shadow[offset as usize..offset as usize + data.len()].copy_from_slice(data);
    }

    pub(super) fn diag_arena_created(&mut self, id: BufferId, size: u32) {
        if self.diag.check {
            self.diag_state.arenas.insert(id, vec![0; size as usize]);
        }
    }

    pub(super) fn diag_arena_write(&mut self, id: BufferId, offset: u32, data: &[u8]) {
        if let Some(shadow) = self.diag_state.arenas.get_mut(&id) {
            shadow[offset as usize..offset as usize + data.len()].copy_from_slice(data);
        }
    }

    pub(super) fn diag_mesh_created(&mut self, id: ResourceId, vertices: &[u8]) {
        if self.diag.check {
            self.diag_state
                .meshes
                .insert(id, pad4(vertices).into_owned());
        }
    }

    /// After a submission: map the checks it carried.
    pub(super) fn diag_submitted(&mut self) {
        for check in self.diag_state.recorded.drain(..) {
            let done = check.done.clone();
            check
                .staging
                .slice(..)
                .map_async(wgpu::MapMode::Read, move |r| {
                    if let Ok(mut d) = done.lock() {
                        *d = Some(r.is_ok());
                    }
                });
            self.diag_state.submitted.push(check);
        }
    }

    /// `check`: copy what the draw of `mesh` is about to read (as the commands
    /// left the pass state) into a staging buffer. Draws without a view block,
    /// a draw block and a material block are skipped.
    pub(super) fn diag_record_check(&mut self, pass: &mut PassRt, mesh: ResourceId) {
        let (Some((vs, vc)), Some((ds, dc)), Some((arena, aoff, asize))) = (
            pass.groups[0].ring,
            pass.groups[2].ring,
            pass.groups[1].statics[0],
        ) else {
            return;
        };
        let (vo, dof) = (
            pass.groups[0].dyn_offset as u64,
            pass.groups[2].dyn_offset as u64,
        );
        let (aoff, asize) = (aoff as u64, (asize as u64).next_multiple_of(4));
        let st = &self.diag_state;
        let (Some(view), Some(draw), Some(arena_bytes)) = (
            st.ring.get(&(vs as usize, vc)),
            st.ring.get(&(ds as usize, dc)),
            st.arenas.get(&arena),
        ) else {
            return;
        };
        let mut expected = Vec::new();
        expected.extend_from_slice(&view[vo as usize..(vo + VIEW_BLOCK_BYTES) as usize]);
        expected.extend_from_slice(&draw[dof as usize..(dof + DRAW_BLOCK_BYTES) as usize]);
        expected.extend_from_slice(&arena_bytes[aoff as usize..(aoff + asize) as usize]);
        let vertices = st.meshes.get(&mesh).cloned().unwrap_or_default();
        let (Some(view_buf), Some(draw_buf), Some(arena_buf), Some(WgpuResource::Mesh(m))) = (
            self.ring_uniform[vs as usize].get(vc as usize).cloned(),
            self.ring_uniform[ds as usize].get(dc as usize).cloned(),
            self.buffers.get(&arena).cloned(),
            self.resources.get(&mesh),
        ) else {
            return;
        };
        let vertex_buf = m.vertex_buffer.clone();
        expected.extend_from_slice(&vertices);

        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("phx-diag-check"),
            size: expected.len() as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        // Copies need the encoder: close the wgpu pass (the draw reopens it).
        pass.rpass = None;
        let encoder = self.encoder();
        let mut at = 0;
        for (src, offset, size) in [
            (&view_buf, vo, VIEW_BLOCK_BYTES),
            (&draw_buf, dof, DRAW_BLOCK_BYTES),
            (&arena_buf, aoff, asize),
            (&vertex_buf, 0, vertices.len() as u64),
        ] {
            if size > 0 {
                encoder.copy_buffer_to_buffer(src, offset, &staging, at, size);
            }
            at += size;
        }
        self.recorded = true;
        let what = format!(
            "f={} pass '{}' {:?} mesh {} view ({vs},{vc})@{vo} draw ({ds},{dc})@{dof} arena {}@{aoff}",
            self.frame_index, pass.label, pass.pipeline, mesh.0, arena.0
        );
        self.diag_state.recorded.push(Check {
            what,
            staging,
            expected,
            done: Arc::new(Mutex::new(None)),
        });
    }

    /// `check`: compare the copies that have mapped (no waiting).
    pub(super) fn diag_poll_checks(&mut self) {
        if self.diag_state.submitted.is_empty() {
            return;
        }
        let _ = self.device.poll(wgpu::PollType::Poll);
        for check in std::mem::take(&mut self.diag_state.submitted) {
            match check.done.lock().ok().and_then(|d| *d) {
                None => self.diag_state.submitted.push(check),
                Some(false) => warn!("CHECK {}: the copy could not be mapped", check.what),
                Some(true) => {
                    if let Ok(got) = check.staging.slice(..).get_mapped_range() {
                        let n = check.expected.len();
                        let bad = (0..n).filter(|&i| got[i] != check.expected[i]).count();
                        if bad > 0 {
                            let first = (0..n).find(|&i| got[i] != check.expected[i]);
                            error!(
                                "CHECK MISMATCH {}: {bad} bytes differ, the first at {first:?} \
                                 (view 0.., draw {VIEW_BLOCK_BYTES}.., material {}.., vertices after)",
                                check.what,
                                VIEW_BLOCK_BYTES + DRAW_BLOCK_BYTES
                            );
                        }
                    }
                    check.staging.unmap();
                    self.diag_state.compared += 1;
                    if self.diag_state.compared % 1000 == 0 {
                        tracing::info!("CHECK {} draws compared", self.diag_state.compared);
                    }
                }
            }
        }
    }

    /// `dump`: copy level `mip` of a 2D `texture` into a staging buffer.
    pub(super) fn diag_record_dump(&mut self, label: &str, texture: &wgpu::Texture, mip: u32) {
        if !texture.usage().contains(wgpu::TextureUsages::COPY_SRC)
            || texture.dimension() != wgpu::TextureDimension::D2
        {
            return;
        }
        let size = texture.size().mip_level_size(mip, texture.dimension());
        let row_bytes = size.width * format_bpp(texture.format());
        let padded_row_bytes = row_bytes.div_ceil(256) * 256;
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("phx-diag-dump"),
            size: padded_row_bytes as u64 * size.height as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let aspect = if texture.format().is_depth_stencil_format() {
            wgpu::TextureAspect::DepthOnly
        } else {
            wgpu::TextureAspect::All
        };
        self.encoder().copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: mip,
                origin: wgpu::Origin3d::ZERO,
                aspect,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &staging,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_row_bytes),
                    rows_per_image: Some(size.height),
                },
            },
            wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
        );
        self.recorded = true;
        self.diag_state.dump_count += 1;
        let name = format!(
            "f{}_{:03}_{}_{}x{}_{:?}",
            self.frame_index,
            self.diag_state.dump_count,
            label.replace(['/', '\\', ' ', '.'], "_"),
            size.width,
            size.height,
            texture.format()
        );
        self.diag_state.dumps.push(Dump {
            name,
            staging,
            row_bytes,
            padded_row_bytes,
            rows: size.height,
        });
    }

    /// `dump`: at the end of the dump frame (submitted), wait for the copies
    /// and write them.
    pub(super) fn diag_save_dumps(&mut self) {
        if self.diag_state.dumps.is_empty() {
            return;
        }
        let dir = std::env::var("LTHEORY_WGPU_DUMP_DIR").unwrap_or_else(|_| ".".into());
        let _ = std::fs::create_dir_all(&dir);
        for dump in std::mem::take(&mut self.diag_state.dumps) {
            let slice = dump.staging.slice(..);
            slice.map_async(wgpu::MapMode::Read, |_| {});
            let _ = self.device.poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(Duration::from_secs(10)),
            });
            let Ok(mapped) = slice.get_mapped_range() else {
                warn!("wgpu dump {}: not mapped", dump.name);
                continue;
            };
            let mut raw = Vec::with_capacity((dump.row_bytes * dump.rows) as usize);
            for row in 0..dump.rows as usize {
                let at = row * dump.padded_row_bytes as usize;
                raw.extend_from_slice(&mapped[at..at + dump.row_bytes as usize]);
            }
            drop(mapped);
            dump.staging.unmap();
            let path = format!("{dir}/{}.raw", dump.name);
            if let Err(e) = std::fs::write(&path, &raw) {
                warn!("wgpu dump {path}: {e}");
            }
        }
    }
}
