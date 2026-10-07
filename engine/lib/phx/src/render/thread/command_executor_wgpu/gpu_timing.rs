//! GPU time per render pass on wgpu: `timestamp_writes` on every wgpu render
//! pass (`Features::TIMESTAMP_QUERY`; writes at the pass boundaries need
//! nothing more).
//!
//! Each frame ring slot owns a query set of `2 * MAX_TIMED_PASSES`
//! timestamps. `SwapBuffers` resolves the used ones into a buffer, copies
//! that into a mappable one and maps it once the frame is submitted; when
//! the slot comes round again (`BeginFrame`, after the GPU has finished its
//! last frame) the mapped bytes are read if the map callback has run. If it
//! has not, that slot skips timing for the frame - nothing ever waits.
//!
//! A pass that a flush point interrupts is several wgpu passes; each gets its
//! own pair of timestamps and they add up under the one label.

use std::sync::atomic::{AtomicU8, Ordering};

use super::*;
use crate::render::thread::gpu_timing::{
    GpuSample, GpuTimingState, MAX_TIMED_PASSES, gpu_timing_enabled,
};

const PENDING: u8 = 0;
const MAPPED: u8 = 1;
const FAILED: u8 = 2;

struct Slot {
    queries: wgpu::QuerySet,
    resolve: wgpu::Buffer,
    read: wgpu::Buffer,
    /// Label id per timed wgpu pass of the slot's frame.
    labels: Vec<u32>,
    /// `read` is being mapped (state set by the callback).
    mapping: Option<Arc<AtomicU8>>,
    /// The slot's frame is not timed (its previous map has not finished).
    skip: bool,
}

pub(super) struct WgpuTimer {
    pub state: GpuTimingState,
    slots: Vec<Slot>,
    /// Nanoseconds per timestamp tick.
    period: f64,
    scratch: Vec<GpuSample>,
}

const BYTES: u64 = (2 * MAX_TIMED_PASSES * 8) as u64;

impl WgpuTimer {
    /// `None` unless the device has `TIMESTAMP_QUERY` and timing is not
    /// turned off with `LTHEORY_GPU_TIMING=0`.
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Option<Self> {
        if !gpu_timing_enabled() || !device.features().contains(wgpu::Features::TIMESTAMP_QUERY) {
            return None;
        }
        let period = queue.get_timestamp_period() as f64;
        if period <= 0.0 {
            return None;
        }
        let slots = (0..crate::render::MAX_FRAMES_IN_FLIGHT)
            .map(|_| Slot {
                queries: device.create_query_set(&wgpu::QuerySetDescriptor {
                    label: Some("phx-gpu-timing"),
                    ty: wgpu::QueryType::Timestamp,
                    count: (2 * MAX_TIMED_PASSES) as u32,
                }),
                resolve: device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("phx-gpu-timing-resolve"),
                    size: BYTES,
                    usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                }),
                read: device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("phx-gpu-timing-read"),
                    size: BYTES,
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
                labels: Vec::with_capacity(MAX_TIMED_PASSES),
                mapping: None,
                skip: false,
            })
            .collect();
        Some(Self {
            state: GpuTimingState::new(true),
            slots,
            period,
            scratch: Vec::with_capacity(MAX_TIMED_PASSES),
        })
    }
}

impl WgpuCommandExecutor {
    /// `BeginFrame` of ring slot `slot` (its previous frame has finished on
    /// the GPU): read the slot's timestamps if they are mapped, never wait.
    pub(super) fn timer_begin_frame(&mut self, slot: usize) {
        let Some(t) = self.timer.as_mut() else {
            return;
        };
        let s = &mut t.slots[slot];
        s.skip = false;
        if let Some(flag) = s.mapping.clone() {
            match flag.load(Ordering::Acquire) {
                PENDING => s.skip = true, // still mapping: leave it alone this round
                state => {
                    s.mapping = None;
                    if state == MAPPED {
                        let n = s.labels.len();
                        t.scratch.clear();
                        if let Ok(view) = s.read.slice(..(n as u64 * 16)).get_mapped_range() {
                            for (i, &label) in s.labels.iter().enumerate() {
                                let word = |k: usize| {
                                    let o = (2 * i + k) * 8;
                                    u64::from_le_bytes(view[o..o + 8].try_into().unwrap())
                                };
                                let (b, e) = (word(0), word(1));
                                t.scratch.push((
                                    label,
                                    (b as f64 * t.period) as u64,
                                    (e as f64 * t.period) as u64,
                                ));
                            }
                        }
                        s.read.unmap();
                        let samples = std::mem::take(&mut t.scratch);
                        t.state.finish_frame(&samples);
                        t.scratch = samples;
                    }
                }
            }
        }
        s.labels.clear();
    }

    /// Reserve a pair of timestamps for a wgpu pass labelled `label_id` in
    /// the running frame; `None` when the frame is not timed or full.
    pub(super) fn timer_alloc(&mut self, label_id: u32) -> Option<(wgpu::QuerySet, u32)> {
        let t = self.timer.as_mut()?;
        let s = &mut t.slots[self.frame_slot];
        if s.skip || s.labels.len() >= MAX_TIMED_PASSES {
            return None;
        }
        let i = s.labels.len() as u32;
        s.labels.push(label_id);
        Some((s.queries.clone(), 2 * i))
    }

    /// The label id of a pass label (0 without a timer).
    pub(super) fn timer_label(&mut self, label: &Arc<str>) -> u32 {
        self.timer.as_mut().map_or(0, |t| t.state.label_id(label))
    }

    /// `SwapBuffers`, before the final submit: resolve the frame's timestamps
    /// into the slot's buffers.
    pub(super) fn timer_resolve(&mut self) {
        let Some(t) = self.timer.as_ref() else {
            return;
        };
        let s = &t.slots[self.frame_slot];
        let n = s.labels.len() as u32;
        if s.skip || n == 0 {
            return;
        }
        let (queries, resolve, read) = (s.queries.clone(), s.resolve.clone(), s.read.clone());
        let encoder = self.encoder();
        encoder.resolve_query_set(&queries, 0..2 * n, &resolve, 0);
        encoder.copy_buffer_to_buffer(&resolve, 0, &read, 0, n as u64 * 16);
        self.recorded = true;
    }

    /// `SwapBuffers`, after the final submit: start mapping the slot's
    /// timestamps.
    pub(super) fn timer_map(&mut self) {
        let Some(t) = self.timer.as_mut() else {
            return;
        };
        let s = &mut t.slots[self.frame_slot];
        let n = s.labels.len() as u64;
        if s.skip || n == 0 {
            return;
        }
        let flag = Arc::new(AtomicU8::new(PENDING));
        let cb = flag.clone();
        s.read
            .slice(..n * 16)
            .map_async(wgpu::MapMode::Read, move |r| {
                cb.store(if r.is_ok() { MAPPED } else { FAILED }, Ordering::Release)
            });
        s.mapping = Some(flag);
    }

    /// The published GPU timings, for `RenderStats`.
    pub(super) fn timer_timings(&self) -> crate::render::GpuTimings {
        self.timer.as_ref().map(|t| t.state.timings().clone()).unwrap_or_default()
    }
}
