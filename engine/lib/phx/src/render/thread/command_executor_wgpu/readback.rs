//! Texture readback of the wgpu executor (render API v2, S8): `copy_texture_to_
//! buffer` into a mappable staging buffer with the rows padded to the 256-byte
//! alignment, `map_async`, and either `device.poll` until it is mapped (sync) or
//! one non-blocking poll per frame at `BeginFrame` (async). The bytes are
//! unpadded, brought from the storage format of the wgpu texture to the
//! layout of the texture's `TexFormat` (`R32F` and `RGBA32F` live in 16F
//! textures, the surface may be BGRA) and then to the requested format.

use super::*;
use crate::render::{ReadSource, ReadbackSlot, convert_format, f16_to_f32};

/// wgpu requires `bytes_per_row` of a texture-to-buffer copy to be a multiple
/// of this.
const ROW_ALIGN: u32 = 256;

/// How the bytes of the wgpu texture relate to the `TexFormat` layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Storage {
    /// The same layout.
    Native,
    /// `R32F` stored as RGBA16F: the first half of each texel, as an f32.
    R32FInRgba16F,
    /// `RGBA32F` stored as RGBA16F: every half as an f32.
    Rgba32FInRgba16F,
}

#[derive(Debug, Clone, Copy)]
struct ReadLayout {
    /// Bytes of one row in the wgpu texture.
    row_bytes: u32,
    padded_row_bytes: u32,
    rows: u32,
    layers: u32,
    storage: Storage,
    /// The texture's own `TexFormat`.
    native: TexFormat,
    /// The surface stores BGRA: swap red and blue after unpacking.
    bgra: bool,
    /// What the caller asked for.
    format: TexFormat,
}

/// A readback whose staging buffer has not been mapped yet.
#[derive(Debug)]
pub(super) struct WgpuPendingRead {
    staging: wgpu::Buffer,
    /// Set by the `map_async` callback: `Some(true)` mapped, `Some(false)` failed.
    done: Arc<Mutex<Option<bool>>>,
    layout: ReadLayout,
    /// Async reads only: where the pixels go.
    slot: Option<Arc<ReadbackSlot>>,
    /// The submission that holds the copy.
    submission: wgpu::SubmissionIndex,
}

impl WgpuPendingRead {
    fn state(&self) -> Option<bool> {
        self.done.lock().ok().and_then(|state| *state)
    }

    /// The mapped staging buffer as the pixels the caller asked for. The
    /// buffer must be mapped.
    fn finish(&self) -> Option<Vec<u8>> {
        let l = self.layout;
        let Ok(mapped) = self.staging.slice(..).get_mapped_range() else {
            self.staging.unmap();
            return None;
        };
        let mut raw =
            Vec::with_capacity(l.row_bytes as usize * l.rows as usize * l.layers as usize);
        for layer in 0..l.layers as usize {
            let layer_start = layer * l.padded_row_bytes as usize * l.rows as usize;
            for row in 0..l.rows as usize {
                let start = layer_start + row * l.padded_row_bytes as usize;
                raw.extend_from_slice(&mapped[start..start + l.row_bytes as usize]);
            }
        }
        drop(mapped);
        self.staging.unmap();

        let mut native = match l.storage {
            Storage::Native => raw,
            Storage::R32FInRgba16F => raw
                .chunks_exact(8)
                .flat_map(|texel| {
                    f16_to_f32(u16::from_ne_bytes([texel[0], texel[1]])).to_ne_bytes()
                })
                .collect(),
            Storage::Rgba32FInRgba16F => raw
                .chunks_exact(2)
                .flat_map(|half| f16_to_f32(u16::from_ne_bytes([half[0], half[1]])).to_ne_bytes())
                .collect(),
        };
        if l.bgra {
            for texel in native.chunks_exact_mut(4) {
                texel.swap(0, 2);
            }
        }
        Some(convert_format(l.native, &native, l.format).into_owned())
    }
}

impl WgpuCommandExecutor {
    /// Copy `region` of `src` into a staging buffer and ask for it to be
    /// mapped. `None` (after a warning) if the source cannot be read.
    fn begin_read(
        &mut self,
        src: ReadSource,
        region: &TexRegion,
        format: TexFormat,
    ) -> Option<WgpuPendingRead> {
        // The copy reads what the frame has drawn so far.
        self.submit_frame();
        let (device, queue) = (self.device.clone(), self.queue.clone());
        let (texture, native, storage, bgra) = match src {
            ReadSource::Texture(id) => {
                let Some((texture, desc)) = self.texture_and_desc(id) else {
                    warn!("wgpu: read of unknown texture {id:?}");
                    return None;
                };
                if TexFormat::is_depth(desc.format) {
                    warn!("wgpu: depth textures cannot be read back ({id:?})");
                    return None;
                }
                let storage = match (desc.format, self.f32_in_f16(desc.format)) {
                    (TexFormat::R32F, true) => Storage::R32FInRgba16F,
                    (TexFormat::RGBA32F, true) => Storage::Rgba32FInRgba16F,
                    _ => Storage::Native,
                };
                (texture, desc.format, storage, false)
            }
            ReadSource::Backbuffer => {
                // The backbuffer is a GL-convention texture (row 0 is the
                // bottom of the window), so the rows come out as GL's.
                self.ensure_backbuffer();
                let backbuffer = self.backbuffer.as_ref().expect("backbuffer");
                (
                    backbuffer.color.clone(),
                    TexFormat::RGBA8,
                    Storage::Native,
                    false,
                )
            }
        };
        if !texture.usage().contains(wgpu::TextureUsages::COPY_SRC) {
            warn!("wgpu: the texture was not created with copy-source usage");
            return None;
        }
        if region.level >= texture.mip_level_count() {
            warn!(
                "wgpu: read of level {} of a texture with {} levels",
                region.level,
                texture.mip_level_count()
            );
            return None;
        }
        let [w, h, depth] = region.size;
        if w == 0 || h == 0 || depth == 0 {
            return None;
        }

        let texel_bytes = format_bpp(texture.format());
        let row_bytes = w * texel_bytes;
        let padded_row_bytes = row_bytes.div_ceil(ROW_ALIGN) * ROW_ALIGN;
        let size = padded_row_bytes as u64 * h as u64 * depth as u64;
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("phx-readback"),
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("phx-readback-copy"),
        });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: region.level,
                origin: wgpu::Origin3d {
                    x: region.origin[0],
                    y: region.origin[1],
                    z: region.origin[2],
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &staging,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_row_bytes),
                    rows_per_image: Some(h),
                },
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: depth,
            },
        );
        let submission = queue.submit([encoder.finish()]);

        let done = Arc::new(Mutex::new(None));
        let callback_done = done.clone();
        staging
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                if let Ok(mut state) = callback_done.lock() {
                    *state = Some(result.is_ok());
                }
            });
        Some(WgpuPendingRead {
            staging,
            done,
            layout: ReadLayout {
                row_bytes,
                padded_row_bytes,
                rows: h,
                layers: depth,
                storage,
                native,
                bgra,
                format,
            },
            slot: None,
            submission,
        })
    }

    /// `ReadTextureSync`: copy, then poll the device until the buffer is
    /// mapped (5 s at most). Empty if the read failed.
    pub(super) fn cmd_read_texture_sync(
        &mut self,
        src: ReadSource,
        region: &TexRegion,
        format: TexFormat,
    ) -> Vec<u8> {
        let Some(job) = self.begin_read(src, region, format) else {
            return Vec::new();
        };
        let what = format!("{src:?} {region:?} as {format:?}");
        let device = self.device.clone();
        let started = Instant::now();
        // Wait for the GPU in the driver (no spinning): a heavy pass before the
        // read (the occlusion bake takes seconds) is not a failure.
        if let Err(e) = device.poll(wgpu::PollType::Wait {
            submission_index: Some(job.submission.clone()),
            timeout: Some(Duration::from_secs(120)),
        }) {
            warn!(
                "wgpu: readback of {what} did not complete in {:?}: {e:?}",
                started.elapsed()
            );
            return Vec::new();
        }
        // The map callback runs from a poll once the copy is done.
        for _ in 0..1000 {
            match job.state() {
                Some(true) => return job.finish().unwrap_or_default(),
                Some(false) => {
                    warn!("wgpu: readback of {what} failed (map error)");
                    return Vec::new();
                }
                None => {
                    let _ = device.poll(wgpu::PollType::Poll);
                    std::thread::yield_now();
                }
            }
        }
        warn!("wgpu: readback of {what} was never mapped");
        Vec::new()
    }

    /// `ReadbackAsync`: copy and map, then leave the job for `poll_readbacks`.
    pub(super) fn cmd_readback_async(
        &mut self,
        src: ReadSource,
        region: &TexRegion,
        format: TexFormat,
        slot: Arc<ReadbackSlot>,
    ) {
        match self.begin_read(src, region, format) {
            Some(mut job) => {
                job.slot = Some(slot);
                self.pending_reads.push(job);
            }
            None => slot.fail(),
        }
    }

    /// Hand over the readbacks whose buffer has mapped. One non-blocking poll
    /// of the device (which runs the map callbacks), no waiting: called once
    /// per frame at `BeginFrame`.
    pub(super) fn poll_readbacks(&mut self) {
        if self.pending_reads.is_empty() {
            return;
        }
        let _ = self.device.poll(wgpu::PollType::Poll);
        let pending = std::mem::take(&mut self.pending_reads);
        for job in pending {
            match job.state() {
                None => self.pending_reads.push(job),
                Some(ok) => {
                    let data = if ok { job.finish() } else { None };
                    if let Some(slot) = &job.slot {
                        match data {
                            Some(bytes) => slot.complete(bytes),
                            None => slot.fail(),
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_pad_to_the_copy_alignment() {
        assert_eq!(1u32.div_ceil(ROW_ALIGN) * ROW_ALIGN, 256);
        assert_eq!((64u32 * 4).div_ceil(ROW_ALIGN) * ROW_ALIGN, 256);
        assert_eq!((65u32 * 4).div_ceil(ROW_ALIGN) * ROW_ALIGN, 512);
    }
}
