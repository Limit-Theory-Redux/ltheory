#![allow(unsafe_code)]
//! GPU time per render pass on GL: `glQueryCounter(GL_TIMESTAMP)` at
//! `BeginRenderPass` and `EndRenderPass` (core in GL 3.3, `ARB_timer_query`).
//!
//! Each frame ring slot owns `2 * MAX_TIMED_PASSES` query objects. A slot's
//! results are read when the slot comes round again at `BeginFrame` - after
//! the slot fence has been waited for, so they are normally ready -
//! and only if `GL_QUERY_RESULT_AVAILABLE` says so for the last query; the
//! executor never waits for a query.

use std::sync::Arc;

use super::command_executor::CommandExecutor;
use super::gpu_timing::{
    GpuSample, GpuTimingState, MAX_TIMED_PASSES, gpu_timing_enabled,
};
use crate::render::{MAX_FRAMES_IN_FLIGHT, gl};

#[derive(Default)]
struct Slot {
    /// `2 * MAX_TIMED_PASSES` query objects, generated on first use.
    queries: Vec<u32>,
    /// Label id per timed pass of the frame that used the slot.
    labels: Vec<u32>,
    /// The frame of the slot has timestamps to read.
    pending: bool,
}

pub(super) struct GlTimer {
    pub state: GpuTimingState,
    slots: [Slot; MAX_FRAMES_IN_FLIGHT],
    /// 0 = not decided yet (needs a GL context), 1 = on, 2 = off.
    mode: u8,
    /// Slot of the running frame.
    cur: usize,
    /// A timed pass is open (its begin timestamp is written).
    open: bool,
    scratch: Vec<GpuSample>,
}

impl GlTimer {
    pub fn new() -> Self {
        Self {
            state: GpuTimingState::new(false),
            slots: Default::default(),
            mode: if gpu_timing_enabled() { 0 } else { 2 },
            cur: 0,
            open: false,
            scratch: Vec::with_capacity(MAX_TIMED_PASSES),
        }
    }
}

impl CommandExecutor {
    /// Decide once (with a context current) whether GL can time passes.
    fn gpu_timer_on(&mut self) -> bool {
        if self.gpu_timer.mode == 0 {
            let ok = self.has_gl_context()
                && gl::QueryCounter::is_loaded()
                && gl::GetQueryObjectui64v::is_loaded()
                && gl::GenQueries::is_loaded();
            self.gpu_timer.mode = if ok { 1 } else { 2 };
            self.gpu_timer.state = GpuTimingState::new(ok);
        }
        self.gpu_timer.mode == 1
    }

    /// `BeginFrame` of ring slot `slot`: pick up the slot's previous frame
    /// (never waits) and clear it for the new one. Call after the slot fence.
    pub(super) fn gpu_timer_begin_frame(&mut self, slot: usize) {
        if !self.gpu_timer_on() {
            return;
        }
        self.gpu_timer_end_pass(); // never leave a begin without its end
        let t = &mut self.gpu_timer;
        t.cur = slot;
        let s = &mut t.slots[slot];
        if s.queries.is_empty() {
            let mut q = vec![0u32; 2 * MAX_TIMED_PASSES];
            unsafe { gl::GenQueries(q.len() as i32, q.as_mut_ptr()) };
            s.queries = q;
            s.labels.reserve(MAX_TIMED_PASSES);
        }
        if s.pending {
            s.pending = false;
            let n = s.labels.len();
            let mut avail = 0i32;
            unsafe {
                gl::GetQueryObjectiv(s.queries[2 * n - 1], gl::QUERY_RESULT_AVAILABLE, &mut avail);
            }
            if avail != 0 {
                t.scratch.clear();
                for (i, &label) in s.labels.iter().enumerate() {
                    let (mut b, mut e) = (0u64, 0u64);
                    unsafe {
                        gl::GetQueryObjectui64v(s.queries[2 * i], gl::QUERY_RESULT, &mut b);
                        gl::GetQueryObjectui64v(s.queries[2 * i + 1], gl::QUERY_RESULT, &mut e);
                    }
                    t.scratch.push((label, b, e));
                }
                let samples = std::mem::take(&mut t.scratch);
                t.state.finish_frame(&samples);
                t.scratch = samples;
            }
        }
        t.slots[slot].labels.clear();
    }

    /// `BeginRenderPass`: write the begin timestamp.
    pub(super) fn gpu_timer_begin_pass(&mut self, label: &Arc<str>) {
        if self.gpu_timer.mode == 2 || !self.gpu_timer_on() {
            return;
        }
        self.gpu_timer_end_pass(); // a pass left open is closed here
        let id = self.gpu_timer.state.label_id(label);
        let t = &mut self.gpu_timer;
        let s = &mut t.slots[t.cur];
        let i = s.labels.len();
        if s.queries.is_empty() || i >= MAX_TIMED_PASSES {
            return;
        }
        unsafe { gl::QueryCounter(s.queries[2 * i], gl::TIMESTAMP) };
        s.labels.push(id);
        t.open = true;
    }

    /// `EndRenderPass`: write the end timestamp of the open timed pass.
    pub(super) fn gpu_timer_end_pass(&mut self) {
        let t = &mut self.gpu_timer;
        if !t.open {
            return;
        }
        t.open = false;
        let s = &mut t.slots[t.cur];
        let i = s.labels.len() - 1;
        unsafe { gl::QueryCounter(s.queries[2 * i + 1], gl::TIMESTAMP) };
        s.pending = true;
    }
}
