//! GPU time per render pass: the backend-independent half.
//!
//! The executors (GL: `glQueryCounter(GL_TIMESTAMP)` pairs, wgpu:
//! `timestamp_writes` on the render passes) record a begin and an end
//! timestamp per pass and read them back a few frames later without ever
//! waiting for them. They hand the finished frame's samples to a
//! [`GpuTimingState`], which sums them per pass label (a label can occur
//! several times in a frame, and a wgpu pass interrupted by a flush point is
//! several GPU passes), smooths them and publishes a fixed-size
//! [`GpuTimings`] that rides in `RenderStats`. See
//! `doc/engine/render-api-v2.md`, "GPU timing".
//!
//! The results describe a frame that finished `MAX_FRAMES_IN_FLIGHT` frames
//! (or a few more) before the frame the rest of the stats describe.

use std::sync::{Arc, Mutex};

/// Distinct pass labels reported per frame (the heaviest ones; the rest only
/// count towards `busy_us`).
pub const MAX_GPU_PASSES: usize = 32;

/// GPU passes timed per frame and ring slot. A frame with more has the later
/// ones untimed.
pub const MAX_TIMED_PASSES: usize = 128;

/// Labels beyond this many distinct ones share the id 0 (`"(other)"`).
const MAX_LABELS: usize = 512;

/// Smoothing factor of the exponential averages, per measured frame.
const SMOOTH: f64 = 0.08;

/// GPU timings of one completed frame.
#[derive(Debug, Clone)]
pub struct GpuTimings {
    /// The backend can time passes and it is enabled (`LTHEORY_GPU_TIMING=0`
    /// disables it). When false every other field is zero ("n/a").
    pub available: bool,
    /// Frames measured so far (changes whenever a new measurement arrives).
    pub measured_frames: u64,
    /// First pass start to last pass end, microseconds: the span the GPU
    /// spent on the frame (including gaps between passes).
    pub total_us: u32,
    /// Exponential average of `total_us`.
    pub total_smooth_us: u32,
    /// Sum of all pass durations (without the gaps).
    pub busy_us: u32,
    /// Entries used in the arrays below, heaviest first.
    pub count: u32,
    /// Label ids (see [`gpu_label`]), the last measured frame's time in
    /// microseconds and its exponential average.
    pub label: [u32; MAX_GPU_PASSES],
    pub us: [u32; MAX_GPU_PASSES],
    pub smooth_us: [u32; MAX_GPU_PASSES],
}

impl Default for GpuTimings {
    fn default() -> Self {
        Self {
            available: false,
            measured_frames: 0,
            total_us: 0,
            total_smooth_us: 0,
            busy_us: 0,
            count: 0,
            label: [0; MAX_GPU_PASSES],
            us: [0; MAX_GPU_PASSES],
            smooth_us: [0; MAX_GPU_PASSES],
        }
    }
}

static LABELS: Mutex<Vec<Arc<str>>> = Mutex::new(Vec::new());

/// The id of a pass label (process-wide). Id 0 is `"(other)"`.
fn intern_label(label: &str) -> u32 {
    let Ok(mut table) = LABELS.lock() else {
        return 0;
    };
    if table.is_empty() {
        table.push(Arc::from("(other)"));
    }
    if let Some(i) = table.iter().position(|l| &**l == label) {
        return i as u32;
    }
    if table.len() >= MAX_LABELS {
        return 0;
    }
    table.push(Arc::from(label));
    (table.len() - 1) as u32
}

/// The pass label of a label id of [`GpuTimings`]; empty if unknown.
pub fn gpu_label(id: u32) -> String {
    LABELS
        .lock()
        .ok()
        .and_then(|t| t.get(id as usize).map(|l| l.to_string()))
        .unwrap_or_default()
}

/// One timed GPU pass: label id and begin/end in nanoseconds on the GPU clock.
pub type GpuSample = (u32, u64, u64);

/// Per-executor accumulation of [`GpuSample`]s into [`GpuTimings`].
pub struct GpuTimingState {
    /// Local label cache (the global table is only locked for new labels).
    cache: Vec<(Arc<str>, u32)>,
    /// Per label id: nanoseconds this frame / smoothed microseconds.
    sum_ns: Vec<u64>,
    smooth: Vec<f64>,
    total_smooth: f64,
    result: GpuTimings,
}

impl GpuTimingState {
    pub fn new(available: bool) -> Self {
        Self {
            cache: Vec::new(),
            sum_ns: Vec::new(),
            smooth: Vec::new(),
            total_smooth: 0.0,
            result: GpuTimings {
                available,
                ..Default::default()
            },
        }
    }

    pub fn timings(&self) -> &GpuTimings {
        &self.result
    }

    pub fn label_id(&mut self, label: &Arc<str>) -> u32 {
        for (l, id) in &self.cache {
            if Arc::ptr_eq(l, label) || **l == **label {
                return *id;
            }
        }
        let id = intern_label(label);
        if self.cache.len() < MAX_LABELS {
            self.cache.push((label.clone(), id));
        }
        id
    }

    /// Fold the samples of one finished frame into the published timings.
    pub fn finish_frame(&mut self, samples: &[GpuSample]) {
        if samples.is_empty() {
            return;
        }
        let ids = samples.iter().map(|s| s.0).max().unwrap_or(0) as usize + 1;
        if self.sum_ns.len() < ids {
            self.sum_ns.resize(ids, 0);
            self.smooth.resize(ids, 0.0);
        }
        let (mut first, mut last, mut busy) = (u64::MAX, 0u64, 0u64);
        for &(id, b, e) in samples {
            let d = e.saturating_sub(b);
            self.sum_ns[id as usize] += d;
            busy += d;
            first = first.min(b);
            last = last.max(e);
        }
        let r = &mut self.result;
        r.measured_frames += 1;
        r.total_us = (last.saturating_sub(first) / 1000).min(u32::MAX as u64) as u32;
        r.busy_us = (busy / 1000).min(u32::MAX as u64) as u32;
        self.total_smooth = if self.total_smooth == 0.0 {
            r.total_us as f64
        } else {
            self.total_smooth + (r.total_us as f64 - self.total_smooth) * SMOOTH
        };
        r.total_smooth_us = self.total_smooth as u32;

        // The heaviest labels, heaviest first (insertion into a short array).
        // `take` leaves 0 behind, so a label that occurs again is skipped.
        let mut n = 0usize;
        for &(id, _, _) in samples {
            let ns = std::mem::take(&mut self.sum_ns[id as usize]);
            if ns == 0 {
                continue;
            }
            let us = (ns / 1000).min(u32::MAX as u64) as u32;
            let s = &mut self.smooth[id as usize];
            *s = if *s == 0.0 { us as f64 } else { *s + (us as f64 - *s) * SMOOTH };
            let mut pos = n;
            while pos > 0 && r.us[pos - 1] < us {
                pos -= 1;
            }
            if pos >= MAX_GPU_PASSES {
                continue;
            }
            let mut i = n.min(MAX_GPU_PASSES - 1);
            while i > pos {
                r.label[i] = r.label[i - 1];
                r.us[i] = r.us[i - 1];
                r.smooth_us[i] = r.smooth_us[i - 1];
                i -= 1;
            }
            r.label[pos] = id;
            r.us[pos] = us;
            r.smooth_us[pos] = *s as u32;
            n = (n + 1).min(MAX_GPU_PASSES);
        }
        r.count = n as u32;
    }
}

/// `LTHEORY_GPU_TIMING=0` turns GPU timing off.
pub fn gpu_timing_enabled() -> bool {
    std::env::var("LTHEORY_GPU_TIMING").map_or(true, |v| v != "0")
}

/// The wgpu features GPU timing needs from `available` (none when timing is
/// off). Timestamps written at pass boundaries need only `TIMESTAMP_QUERY`.
pub fn wgpu_timing_features(available: wgpu::Features) -> wgpu::Features {
    if gpu_timing_enabled() {
        available & wgpu::Features::TIMESTAMP_QUERY
    } else {
        wgpu::Features::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sums_per_label_and_sorts() {
        let mut s = GpuTimingState::new(true);
        let a = s.label_id(&Arc::from("gt-a"));
        let b = s.label_id(&Arc::from("gt-b"));
        assert_ne!(a, b);
        assert_eq!(s.label_id(&Arc::from("gt-a")), a);
        s.finish_frame(&[
            (a, 1_000_000, 2_000_000),
            (b, 2_000_000, 12_000_000),
            (a, 12_000_000, 15_000_000),
        ]);
        let t = s.timings();
        assert_eq!(t.count, 2);
        assert_eq!((t.label[0], t.us[0]), (b, 10_000));
        assert_eq!((t.label[1], t.us[1]), (a, 4_000));
        assert_eq!(t.total_us, 14_000);
        assert_eq!(t.busy_us, 14_000);
        assert_eq!(gpu_label(a), "gt-a");
    }
}
