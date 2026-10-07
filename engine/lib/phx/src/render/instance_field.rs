//! Bulk culler for static instance fields (asteroid belts and rings).
//!
//! This is the per-frame work that used to be a serial Lua loop in
//! `AsteroidBeltRenderer` (see `doc/engine/parallel-recording.md`, D1).
//! The field is partitioned into angular chunks at generation time. Per
//! frame each chunk is culled by its centroid (render distance, view cone),
//! then each asteroid is LOD-selected by projected screen size (sub-pixel
//! asteroids are dropped) and its index appended to a per-LOD list. The
//! lists are drawn with `DrawInstancedIndices` (texture-fetch instancing).
//!
//! All distance and LOD comparisons are done in `f64`, over the same
//! expressions in the same order as the Lua loop, so the LOD choices are
//! bit-identical to it.
//!
//! Threading: the chunks may be split over a small rayon pool. Workers only
//! fill their own per-chunk lists; the calling thread joins them and merges
//! in chunk order, applying the draw cap in that order, so the output is
//! identical for any worker count. The vertex ring is only touched by the
//! caller (`draw`).

use std::sync::{Arc, Mutex};

use rayon::prelude::*;

use crate::render::{Mesh, RenderPass, Renderer};

/// Number of LOD levels.
pub const LOD_COUNT: usize = 8;

/// Squared screen-size thresholds (pixels) per LOD, 0 = most detailed.
/// Bands halve each level (32, 16, 8, 4, 2, 1, 0.5, 0.25 px). An asteroid
/// below the last one is sub-pixel and culled entirely.
const LOD_PX_MIN_SQ: [f64; LOD_COUNT] = [1024.0, 256.0, 64.0, 16.0, 4.0, 1.0, 0.25, 0.0625];

/// A view cone cutoff: chunks whose centroid is more than ~107 degrees
/// behind the camera are skipped.
const CONE_COS_MIN: f64 = -0.3;

/// Default minimum field size to split over workers.
const DEFAULT_PARALLEL_MIN: usize = 20_000;

/// Per-frame inputs of the culler.
#[derive(Clone, Copy)]
struct CullParams {
    eye: [f64; 3],
    origin: [f64; 3],
    fwd: [f64; 3],
    fwd_len_sq: f64,
    px_per_unit_sq: f64,
    render_dist_sq: f64,
    lod_count: usize,
}

/// One chunk's survivors, in iteration order: asteroid index and LOD.
#[derive(Default)]
struct ChunkOut {
    index: Vec<u32>,
    lod: Vec<u8>,
}

/// The immutable, shared part of the field.
struct FieldData {
    pos: Vec<[f32; 3]>,
    scales: Vec<f32>,
    /// `chunks + 1` prefix offsets into `chunk_indices`.
    chunk_offsets: Vec<u32>,
    /// Asteroid indices (0-based), grouped by chunk.
    chunk_indices: Vec<u32>,
    chunk_centroids: Vec<[f32; 3]>,
}

impl FieldData {
    fn chunk_count(&self) -> usize {
        self.chunk_centroids.len()
    }

    /// Cull one chunk into `out`, stopping at `limit` survivors.
    fn cull_chunk(
        &self,
        c: usize,
        p: &CullParams,
        limit: usize,
        spawned: &[u8],
        out: &mut ChunkOut,
    ) {
        out.index.clear();
        out.lod.clear();
        let begin = self.chunk_offsets[c] as usize;
        let end = self.chunk_offsets[c + 1] as usize;
        if begin == end || limit == 0 {
            return;
        }

        // Centroid eye-distance (world coords = origin + centroid).
        let cen = self.chunk_centroids[c];
        let rcx = p.origin[0] + cen[0] as f64 - p.eye[0];
        let rcy = p.origin[1] + cen[1] as f64 - p.eye[1];
        let rcz = p.origin[2] + cen[2] as f64 - p.eye[2];
        let c_dist_sq = rcx * rcx + rcy * rcy + rcz * rcz;
        if c_dist_sq > p.render_dist_sq {
            return;
        }
        if p.fwd_len_sq > 1e-9 && c_dist_sq > 1e-9 {
            let inv_dist = 1.0 / c_dist_sq.sqrt();
            let dot = (rcx * p.fwd[0] + rcy * p.fwd[1] + rcz * p.fwd[2]) * inv_dist
                / p.fwd_len_sq.sqrt();
            if dot < CONE_COS_MIN {
                return;
            }
        }

        for &i in &self.chunk_indices[begin..end] {
            if out.index.len() >= limit {
                break;
            }
            let iu = i as usize;
            if spawned[iu] != 0 {
                continue;
            }
            let q = self.pos[iu];
            let rx = p.origin[0] + q[0] as f64 - p.eye[0];
            let ry = p.origin[1] + q[1] as f64 - p.eye[1];
            let rz = p.origin[2] + q[2] as f64 - p.eye[2];
            let dist_sq = rx * rx + ry * ry + rz * rz;

            // Projected pixel height ~ scale / dist * pxPerUnit, compared
            // squared and by multiplication (no sqrt, no division).
            let s = self.scales[iu] as f64;
            let s2ppu = s * s * p.px_per_unit_sq;
            let mut li = 0;
            while li < LOD_COUNT - 1 && dist_sq * LOD_PX_MIN_SQ[li] > s2ppu {
                li += 1;
            }
            if dist_sq * LOD_PX_MIN_SQ[li] <= s2ppu && li < p.lod_count {
                out.index.push(i);
                out.lod.push(li as u8);
            }
        }
    }
}

/// The pool for `workers` threads, shared by all fields (created lazily,
/// replaced when the requested size changes).
fn pool(workers: usize) -> Option<Arc<rayon::ThreadPool>> {
    static POOL: Mutex<Option<(usize, Arc<rayon::ThreadPool>)>> = Mutex::new(None);
    let mut slot = POOL.lock().ok()?;
    if let Some((n, p)) = slot.as_ref() {
        if *n == workers {
            return Some(p.clone());
        }
    }
    let p = rayon::ThreadPoolBuilder::new()
        .num_threads(workers)
        .thread_name(|i| format!("phx-cull-{i}"))
        .build()
        .ok()?;
    let p = Arc::new(p);
    *slot = Some((workers, p.clone()));
    Some(p)
}

pub struct InstanceField {
    data: FieldData,
    outs: Vec<ChunkOut>,
    spawned: Vec<u8>,
    /// Spawned indices for the next FFI `Cull`, set by `SetSpawned` (an
    /// empty set goes through `ClearSpawned`: the FFI rejects empty slices).
    pending_spawned: Vec<u32>,
    /// Merged, capped result: asteroid indices per LOD.
    lists: [Vec<u32>; LOD_COUNT],
    /// LODs in order of first appearance in the merged stream.
    lod_order: Vec<u8>,
    lod_count: usize,
    workers: usize,
    parallel_min: usize,
}

impl InstanceField {
    pub fn new(
        pos: &[f32],
        scales: &[f32],
        chunk_offsets: &[u32],
        chunk_indices: &[u32],
        chunk_centroids: &[f32],
    ) -> Self {
        let n = scales.len();
        assert!(pos.len() >= n * 3, "InstanceField: pos too short");
        assert!(!chunk_offsets.is_empty(), "InstanceField: no chunk offsets");
        let chunks = chunk_offsets.len() - 1;
        assert!(
            chunk_centroids.len() >= chunks * 3,
            "InstanceField: centroids too short"
        );
        assert!(
            chunk_offsets[chunks] as usize <= chunk_indices.len(),
            "InstanceField: chunk offsets exceed indices"
        );
        assert!(
            chunk_indices
                .iter()
                .take(chunk_offsets[chunks] as usize)
                .all(|&i| (i as usize) < n),
            "InstanceField: chunk index out of range"
        );
        Self {
            data: FieldData {
                pos: pos[..n * 3].chunks_exact(3).map(|v| [v[0], v[1], v[2]]).collect(),
                scales: scales.to_vec(),
                chunk_offsets: chunk_offsets.to_vec(),
                chunk_indices: chunk_indices.to_vec(),
                chunk_centroids: chunk_centroids[..chunks * 3]
                    .chunks_exact(3)
                    .map(|v| [v[0], v[1], v[2]])
                    .collect(),
            },
            outs: (0..chunks).map(|_| ChunkOut::default()).collect(),
            spawned: vec![0; n],
            pending_spawned: Vec::new(),
            lists: Default::default(),
            lod_order: Vec::new(),
            lod_count: LOD_COUNT,
            workers: 0,
            parallel_min: DEFAULT_PARALLEL_MIN,
        }
    }

    pub fn set_parallel_min_count(&mut self, n: usize) {
        self.parallel_min = n;
    }

    /// Cull the field. `spawned` are 0-based indices of asteroids that are
    /// real entities (not drawn here). Returns the number of instances.
    #[allow(clippy::too_many_arguments)]
    pub fn cull_field(
        &mut self,
        eye: [f64; 3],
        fwd: [f64; 3],
        origin: [f64; 3],
        px_per_unit_sq: f64,
        render_dist_sq: f64,
        max_drawn: usize,
        spawned: &[u32],
    ) -> usize {
        let p = CullParams {
            eye,
            origin,
            fwd,
            fwd_len_sq: fwd[0] * fwd[0] + fwd[1] * fwd[1] + fwd[2] * fwd[2],
            px_per_unit_sq,
            render_dist_sq,
            lod_count: self.lod_count,
        };
        for &s in spawned {
            if let Some(f) = self.spawned.get_mut(s as usize) {
                *f = 1;
            }
        }

        let chunks = self.data.chunk_count();
        let parallel = self.workers >= 2 && self.data.scales.len() >= self.parallel_min;
        let pool = if parallel { pool(self.workers) } else { None };
        match pool {
            Some(pool) => {
                let data = &self.data;
                let mask = &self.spawned[..];
                let outs = &mut self.outs;
                pool.install(|| {
                    outs.par_iter_mut().enumerate().for_each(|(c, out)| {
                        data.cull_chunk(c, &p, max_drawn, mask, out);
                    });
                });
            }
            None => {
                // Inline: each chunk only needs the budget that is left.
                let mut drawn = 0usize;
                for c in 0..chunks {
                    let limit = max_drawn.saturating_sub(drawn);
                    self.data
                        .cull_chunk(c, &p, limit, &self.spawned, &mut self.outs[c]);
                    drawn += self.outs[c].index.len();
                }
            }
        }

        for &s in spawned {
            if let Some(f) = self.spawned.get_mut(s as usize) {
                *f = 0;
            }
        }

        // Join: chunk order, first `max_drawn` survivors.
        for l in &mut self.lists {
            l.clear();
        }
        self.lod_order.clear();
        let mut remaining = max_drawn;
        for out in &self.outs {
            let take = out.index.len().min(remaining);
            for k in 0..take {
                let lod = out.lod[k] as usize;
                if self.lists[lod].is_empty() {
                    self.lod_order.push(lod as u8);
                }
                self.lists[lod].push(out.index[k]);
            }
            remaining -= take;
            if remaining == 0 {
                break;
            }
        }
        max_drawn - remaining
    }

    /// The indices of LOD `lod` after the last cull.
    pub fn indices(&self, lod: usize) -> &[u32] {
        self.lists.get(lod).map_or(&[], |l| l.as_slice())
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl InstanceField {
    /// `pos` is `x, y, z` per asteroid, `chunk_offsets` the `chunks + 1`
    /// prefix offsets into `chunk_indices` (0-based asteroid indices),
    /// `chunk_centroids` is `x, y, z` per chunk. Copied.
    #[bind(name = "Create")]
    pub fn create(
        pos: &[f32],
        scales: &[f32],
        chunk_offsets: &[u32],
        chunk_indices: &[u32],
        chunk_centroids: &[f32],
    ) -> InstanceField {
        InstanceField::new(pos, scales, chunk_offsets, chunk_indices, chunk_centroids)
    }

    /// Worker threads for the cull. 0 and 1 mean single-threaded.
    pub fn set_workers(&mut self, workers: u32) {
        self.workers = workers as usize;
    }

    /// Number of LOD levels that have a mesh; asteroids that select a
    /// higher level are skipped.
    pub fn set_lod_count(&mut self, count: u32) {
        self.lod_count = (count as usize).min(LOD_COUNT);
    }

    /// 0-based indices of asteroids that are real entities and are not drawn
    /// by the next `Cull`. Copied.
    #[bind(name = "SetSpawned")]
    pub fn set_spawned(&mut self, spawned: &[u32]) {
        self.pending_spawned.clear();
        self.pending_spawned.extend_from_slice(spawned);
    }

    /// No spawned asteroids for the next `Cull`.
    #[bind(name = "ClearSpawned")]
    pub fn clear_spawned(&mut self) {
        self.pending_spawned.clear();
    }

    /// Cull the field (see the module docs), skipping the asteroids given to
    /// `SetSpawned`. Returns the number of instances to draw.
    #[bind(name = "Cull")]
    #[allow(clippy::too_many_arguments)]
    pub fn cull(
        &mut self,
        eye_x: f64,
        eye_y: f64,
        eye_z: f64,
        fwd_x: f64,
        fwd_y: f64,
        fwd_z: f64,
        origin_x: f64,
        origin_y: f64,
        origin_z: f64,
        px_per_unit_sq: f64,
        render_dist_sq: f64,
        max_drawn: u32,
    ) -> u32 {
        let spawned = std::mem::take(&mut self.pending_spawned);
        let n = self.cull_field(
            [eye_x, eye_y, eye_z],
            [fwd_x, fwd_y, fwd_z],
            [origin_x, origin_y, origin_z],
            px_per_unit_sq,
            render_dist_sq,
            max_drawn as usize,
            &spawned,
        ) as u32;
        self.pending_spawned = spawned;
        n
    }

    /// Instances of LOD `lod` (0-based) after the last cull.
    pub fn get_count(&self, lod: u32) -> u32 {
        self.indices(lod as usize).len() as u32
    }

    /// Number of LODs that had instances in the last cull.
    pub fn get_lod_order_len(&self) -> u32 {
        self.lod_order.len() as u32
    }

    /// The `k`th LOD (0-based) by first appearance in the last cull.
    pub fn get_lod_order(&self, k: u32) -> u32 {
        self.lod_order.get(k as usize).copied().unwrap_or(0) as u32
    }

    /// Record the instanced draw of LOD `lod` with `mesh`. The indices are
    /// copied into the vertex ring on the calling thread.
    pub fn draw(&self, pass: &RenderPass, r: &mut Renderer, lod: u32, mesh: &mut Mesh) {
        let list = self.indices(lod as usize);
        if !list.is_empty() {
            pass.draw_instanced_indices(r, mesh, list);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Lcg(u64);
    impl Lcg {
        fn next(&mut self) -> f64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((self.0 >> 11) as f64) / ((1u64 << 53) as f64)
        }
    }

    const CHUNKS: usize = 32;

    /// A belt like the generator makes: ring around the origin, chunked by
    /// angle, centroids rounded to f32.
    fn make_belt(n: usize, seed: u64) -> (InstanceField, Vec<[f32; 3]>, Vec<f32>) {
        let mut rng = Lcg(seed);
        let mut pos = Vec::new();
        let mut scales = Vec::new();
        let mut chunk_of = Vec::new();
        for _ in 0..n {
            let a = rng.next() * std::f64::consts::TAU;
            let r = 2.0e5 + (rng.next() - 0.5) * 4.0e4;
            let p = [
                (a.cos() * r) as f32,
                ((rng.next() - 0.5) * 3000.0) as f32,
                (a.sin() * r) as f32,
            ];
            pos.push(p);
            scales.push((10.0 + 190.0 * rng.next() * rng.next()) as f32);
            let ang = (p[2] as f64).atan2(p[0] as f64);
            let sz = std::f64::consts::TAU / CHUNKS as f64;
            let c = (((ang + std::f64::consts::PI) / sz).floor() as usize).min(CHUNKS - 1);
            chunk_of.push(c);
        }
        let mut offsets = vec![0u32; CHUNKS + 1];
        let mut indices = Vec::new();
        let mut cent = vec![0f32; CHUNKS * 3];
        for c in 0..CHUNKS {
            let mut sum = [0f64; 3];
            let mut cnt = 0;
            for i in 0..n {
                if chunk_of[i] == c {
                    indices.push(i as u32);
                    for k in 0..3 {
                        sum[k] += pos[i][k] as f64;
                    }
                    cnt += 1;
                }
            }
            offsets[c + 1] = indices.len() as u32;
            if cnt > 0 {
                for k in 0..3 {
                    cent[c * 3 + k] = (sum[k] / cnt as f64) as f32;
                }
            }
        }
        let flat: Vec<f32> = pos.iter().flat_map(|p| *p).collect();
        let f = InstanceField::new(&flat, &scales, &offsets, &indices, &cent);
        (f, pos, scales)
    }

    /// The Lua loop, written the plain way (chunk order = field order).
    fn reference(
        f: &InstanceField,
        eye: [f64; 3],
        fwd: [f64; 3],
        origin: [f64; 3],
        ppu: f64,
        dist_sq: f64,
        max: usize,
        spawned: &[u32],
    ) -> Vec<(u32, u8)> {
        let d = &f.data;
        let mut out = Vec::new();
        let fl = fwd[0] * fwd[0] + fwd[1] * fwd[1] + fwd[2] * fwd[2];
        'chunks: for c in 0..d.chunk_count() {
            let (b, e) = (d.chunk_offsets[c] as usize, d.chunk_offsets[c + 1] as usize);
            if b == e {
                continue;
            }
            let ce = d.chunk_centroids[c];
            let r = [
                origin[0] + ce[0] as f64 - eye[0],
                origin[1] + ce[1] as f64 - eye[1],
                origin[2] + ce[2] as f64 - eye[2],
            ];
            let cd = r[0] * r[0] + r[1] * r[1] + r[2] * r[2];
            if cd > dist_sq {
                continue;
            }
            if fl > 1e-9 && cd > 1e-9 {
                let dot = (r[0] * fwd[0] + r[1] * fwd[1] + r[2] * fwd[2]) * (1.0 / cd.sqrt())
                    / fl.sqrt();
                if dot < -0.3 {
                    continue;
                }
            }
            for &i in &d.chunk_indices[b..e] {
                if out.len() >= max {
                    break 'chunks;
                }
                if spawned.contains(&i) {
                    continue;
                }
                let q = d.pos[i as usize];
                let rx = origin[0] + q[0] as f64 - eye[0];
                let ry = origin[1] + q[1] as f64 - eye[1];
                let rz = origin[2] + q[2] as f64 - eye[2];
                let ds = rx * rx + ry * ry + rz * rz;
                let s = d.scales[i as usize] as f64;
                let s2 = s * s * ppu;
                let mut li = 1usize;
                while li < 8 && ds * LOD_PX_MIN_SQ[li - 1] > s2 {
                    li += 1;
                }
                if ds * LOD_PX_MIN_SQ[li - 1] <= s2 {
                    out.push((i, (li - 1) as u8));
                }
            }
        }
        out
    }

    fn flatten(f: &InstanceField) -> Vec<Vec<u32>> {
        (0..LOD_COUNT).map(|l| f.indices(l).to_vec()).collect()
    }

    fn by_lod(stream: &[(u32, u8)]) -> Vec<Vec<u32>> {
        let mut v = vec![Vec::new(); LOD_COUNT];
        for &(i, l) in stream {
            v[l as usize].push(i);
        }
        v
    }

    fn ppu() -> f64 {
        let fov = 70.0f64 * 0.5 * (std::f64::consts::PI / 180.0);
        (720.0 / (2.0 * fov.tan())).powi(2)
    }

    #[test]
    fn matches_scalar_reference_with_caps_and_spawned() {
        let (mut f, _, _) = make_belt(30_000, 7);
        let ppu = ppu();
        let spawned: Vec<u32> = (0..30_000).step_by(97).collect();
        let cams = [
            ([0.0, 0.0, 0.0], [0.0, 0.0, -1.0]),
            ([5.0e5, 2.0e4, 1.0e5], [-0.8, -0.1, -0.5]),
            ([1.0e5, 900.0, 1.9e5], [0.3, 0.0, 0.9]),
            ([0.0, 3.0e4, 0.0], [0.0, 0.0, 0.0]),
        ];
        for (eye, fwd) in cams {
            for max in [0usize, 1, 100, 5000, 1_000_000] {
                let want = reference(&f, eye, fwd, [10.0, 0.0, -5.0], ppu, 4.0e12, max, &spawned);
                let n = f.cull_field(eye, fwd, [10.0, 0.0, -5.0], ppu, 4.0e12, max, &spawned);
                assert_eq!(n as usize, want.len());
                assert_eq!(flatten(&f), by_lod(&want), "max {max}");
                // first-appearance LOD order
                let mut order = Vec::new();
                for &(_, l) in &want {
                    if !order.contains(&l) {
                        order.push(l);
                    }
                }
                assert_eq!(f.lod_order, order);
            }
        }
    }

    #[test]
    fn identical_output_for_1_2_and_4_workers() {
        let (mut f, _, _) = make_belt(40_000, 99);
        let ppu = ppu();
        f.set_parallel_min_count(0);
        for (eye, fwd) in [
            ([0.0, 0.0, 0.0], [0.0, 0.0, -1.0]),
            ([2.2e5, 100.0, 3.0e4], [-1.0, 0.0, 0.2]),
        ] {
            for max in [50usize, 5000, 1_000_000] {
                let mut results = Vec::new();
                for workers in [0u32, 1, 2, 4] {
                    f.set_workers(workers);
                    let n = f.cull_field(eye, fwd, [0.0; 3], ppu, 4.0e12, max, &[3, 4, 5]);
                    results.push((n, flatten(&f), f.lod_order.clone()));
                }
                assert!(results[0].0 > 0);
                for r in &results[1..] {
                    assert_eq!(r, &results[0], "max {max}");
                }
            }
        }
    }

    #[test]
    fn lod_count_skips_missing_levels() {
        let (mut f, _, _) = make_belt(5_000, 3);
        f.lod_count = 3;
        f.cull_field([0.0; 3], [0.0, 0.0, -1.0], [0.0; 3], ppu(), 4.0e12, 100_000, &[]);
        for l in 3..LOD_COUNT {
            assert!(f.indices(l).is_empty());
        }
    }
}
