//! Triple-buffered frame ring for pipelined rendering.
//!
//! Allows three frames to be in-flight simultaneously:
//! - Main thread: Submitting commands for frame N+2
//! - Workers: Preparing data for frame N+1
//! - Render thread: Executing frame N
//!
//! This maximizes throughput by keeping all threads busy.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

use parking_lot::RwLock;
use tracing::debug;

use super::RenderCommand;

/// Number of frames in the ring buffer
pub const FRAME_RING_SIZE: usize = 3;

/// Data for a single frame in the ring
#[derive(Default)]
pub struct FrameData {
    /// Frame ID (monotonically increasing)
    pub frame_id: u64,
    /// Commands queued for this frame
    pub commands: Vec<RenderCommand>,
    /// Whether this frame is ready to render
    pub ready: bool,
    /// Whether this frame has been rendered
    pub rendered: bool,
}

impl FrameData {
    fn new(frame_id: u64) -> Self {
        Self {
            frame_id,
            commands: Vec::with_capacity(1024),
            ready: false,
            rendered: false,
        }
    }

    fn reset(&mut self, frame_id: u64) {
        self.frame_id = frame_id;
        self.commands.clear();
        self.ready = false;
        self.rendered = false;
    }
}

/// Triple-buffered frame ring for pipelined rendering
pub struct FrameRing {
    /// The three frame buffers
    frames: [RwLock<FrameData>; FRAME_RING_SIZE],
    /// Index where main thread submits (write)
    submit_index: AtomicUsize,
    /// Index where workers prepare data (reserved for future use)
    #[allow(dead_code)]
    prepare_index: AtomicUsize,
    /// Index where render thread reads (execute)
    render_index: AtomicUsize,
    /// Current frame ID counter
    frame_counter: AtomicU64,
}

impl FrameRing {
    /// Create a new frame ring
    pub fn new() -> Self {
        Self {
            frames: [
                RwLock::new(FrameData::new(0)),
                RwLock::new(FrameData::new(1)),
                RwLock::new(FrameData::new(2)),
            ],
            submit_index: AtomicUsize::new(0),
            prepare_index: AtomicUsize::new(0),
            render_index: AtomicUsize::new(0),
            frame_counter: AtomicU64::new(0),
        }
    }

    /// Get the current submit frame index
    pub fn submit_index(&self) -> usize {
        self.submit_index.load(Ordering::Acquire)
    }

    /// Get the current render frame index
    pub fn render_index(&self) -> usize {
        self.render_index.load(Ordering::Acquire)
    }

    /// Get the current frame ID being submitted
    pub fn current_frame_id(&self) -> u64 {
        self.frame_counter.load(Ordering::Acquire)
    }

    /// Begin a new frame for submission
    /// Returns the frame ID
    pub fn begin_frame(&self) -> u64 {
        let frame_id = self.frame_counter.fetch_add(1, Ordering::AcqRel);
        let index = self.submit_index.load(Ordering::Acquire);

        // Wait if we're about to overwrite an un-rendered frame
        // Check if the frame at this index is ready but not yet rendered
        loop {
            let frame = self.frames[index].read();
            if !frame.ready || frame.rendered {
                // Frame is available for reuse
                break;
            }
            drop(frame); // Release lock before spinning
            std::hint::spin_loop();
        }

        let mut frame = self.frames[index].write();
        frame.reset(frame_id);

        debug!("Begin frame {} at index {}", frame_id, index);
        frame_id
    }

    /// Submit a command to the current frame
    pub fn submit(&self, cmd: RenderCommand) {
        let index = self.submit_index.load(Ordering::Acquire);
        let mut frame = self.frames[index].write();
        frame.commands.push(cmd);
    }

    /// Submit multiple commands to the current frame
    pub fn submit_batch(&self, cmds: impl IntoIterator<Item = RenderCommand>) {
        let index = self.submit_index.load(Ordering::Acquire);
        let mut frame = self.frames[index].write();
        frame.commands.extend(cmds);
    }

    /// End the current frame and mark it ready for rendering
    pub fn end_frame(&self) {
        let index = self.submit_index.load(Ordering::Acquire);

        {
            let mut frame = self.frames[index].write();
            frame.ready = true;
            debug!("End frame {} at index {} with {} commands",
                frame.frame_id, index, frame.commands.len());
        }

        // Advance to next buffer
        let next_index = (index + 1) % FRAME_RING_SIZE;
        self.submit_index.store(next_index, Ordering::Release);
    }

    /// Try to get the next frame ready for rendering
    /// Returns None if no frame is ready
    pub fn try_get_render_frame(&self) -> Option<RenderFrameGuard<'_>> {
        let index = self.render_index.load(Ordering::Acquire);
        let frame = self.frames[index].read();

        if frame.ready && !frame.rendered {
            Some(RenderFrameGuard {
                ring: self,
                index,
            })
        } else {
            None
        }
    }

    /// Get commands for the current render frame
    /// Panics if called without a valid render frame
    pub fn get_render_commands(&self) -> Vec<RenderCommand> {
        let index = self.render_index.load(Ordering::Acquire);
        let frame = self.frames[index].read();
        frame.commands.clone()
    }

    /// Mark the current render frame as complete and advance
    pub fn complete_render_frame(&self) {
        let index = self.render_index.load(Ordering::Acquire);

        {
            let mut frame = self.frames[index].write();
            frame.rendered = true;
            debug!("Completed render frame {} at index {}", frame.frame_id, index);
        }

        // Advance to next buffer
        let next_index = (index + 1) % FRAME_RING_SIZE;
        self.render_index.store(next_index, Ordering::Release);
    }

    /// Get statistics about the frame ring
    pub fn stats(&self) -> FrameRingStats {
        let submit = self.submit_index.load(Ordering::Acquire);
        let render = self.render_index.load(Ordering::Acquire);

        // Calculate frames in flight
        let in_flight = if submit >= render {
            submit - render
        } else {
            FRAME_RING_SIZE - render + submit
        };

        FrameRingStats {
            frames_in_flight: in_flight,
            submit_index: submit,
            render_index: render,
            current_frame_id: self.frame_counter.load(Ordering::Acquire),
        }
    }
}

impl Default for FrameRing {
    fn default() -> Self {
        Self::new()
    }
}

/// Guard for accessing a render frame
/// Automatically advances the ring when dropped
pub struct RenderFrameGuard<'a> {
    ring: &'a FrameRing,
    index: usize,
}

impl<'a> RenderFrameGuard<'a> {
    /// Get the frame ID
    pub fn frame_id(&self) -> u64 {
        self.ring.frames[self.index].read().frame_id
    }

    /// Get the commands for this frame
    pub fn commands(&self) -> Vec<RenderCommand> {
        self.ring.frames[self.index].read().commands.clone()
    }

    /// Mark this frame as complete
    pub fn complete(self) {
        self.ring.complete_render_frame();
    }
}

/// Statistics about the frame ring
#[derive(Debug, Clone, Copy)]
pub struct FrameRingStats {
    /// Number of frames currently in flight
    pub frames_in_flight: usize,
    /// Current submit buffer index
    pub submit_index: usize,
    /// Current render buffer index
    pub render_index: usize,
    /// Latest frame ID
    pub current_frame_id: u64,
}

/// Handle for submitting frames from the main thread
pub struct FrameSubmitter {
    ring: Arc<FrameRing>,
}

impl FrameSubmitter {
    pub fn new(ring: Arc<FrameRing>) -> Self {
        Self { ring }
    }

    pub fn begin_frame(&self) -> u64 {
        self.ring.begin_frame()
    }

    pub fn submit(&self, cmd: RenderCommand) {
        self.ring.submit(cmd);
    }

    pub fn submit_batch(&self, cmds: impl IntoIterator<Item = RenderCommand>) {
        self.ring.submit_batch(cmds);
    }

    pub fn end_frame(&self) {
        self.ring.end_frame();
    }

    pub fn stats(&self) -> FrameRingStats {
        self.ring.stats()
    }
}

/// Handle for rendering frames on the render thread
pub struct FrameRenderer {
    ring: Arc<FrameRing>,
}

impl FrameRenderer {
    pub fn new(ring: Arc<FrameRing>) -> Self {
        Self { ring }
    }

    pub fn try_get_frame(&self) -> Option<RenderFrameGuard<'_>> {
        self.ring.try_get_render_frame()
    }

    pub fn stats(&self) -> FrameRingStats {
        self.ring.stats()
    }
}

/// Create a paired submitter and renderer sharing the same frame ring
pub fn create_frame_pipeline() -> (FrameSubmitter, FrameRenderer) {
    let ring = Arc::new(FrameRing::new());
    (
        FrameSubmitter::new(Arc::clone(&ring)),
        FrameRenderer::new(ring),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_ring_basic() {
        let ring = FrameRing::new();

        // Begin first frame
        let frame_id = ring.begin_frame();
        assert_eq!(frame_id, 0);

        // Submit some commands
        ring.submit(RenderCommand::Clear {
            color: Some([1.0, 0.0, 0.0, 1.0]),
            depth: Some(1.0),
        });

        // End frame
        ring.end_frame();

        // Should have a render frame ready
        let guard = ring.try_get_render_frame();
        assert!(guard.is_some());

        let guard = guard.unwrap();
        assert_eq!(guard.frame_id(), 0);
        assert_eq!(guard.commands().len(), 1);

        guard.complete();
    }

    #[test]
    fn test_frame_ring_pipeline() {
        let ring = FrameRing::new();

        // Submit and render frames in interleaved manner to avoid deadlock
        // (ring only has 3 slots, can't have more than 2 in-flight without blocking)
        for i in 0..3 {
            let frame_id = ring.begin_frame();
            assert_eq!(frame_id, i);
            ring.submit(RenderCommand::Clear {
                color: Some([i as f32 / 3.0, 0.0, 0.0, 1.0]),
                depth: None,
            });
            ring.end_frame();

            // Render immediately to prevent buffer overflow
            let guard = ring.try_get_render_frame().unwrap();
            assert_eq!(guard.frame_id(), i);
            guard.complete();
        }

        // No more frames ready
        assert!(ring.try_get_render_frame().is_none());
    }

    #[test]
    fn test_frame_ring_stats() {
        let ring = FrameRing::new();

        let stats = ring.stats();
        assert_eq!(stats.frames_in_flight, 0);

        ring.begin_frame();
        ring.end_frame();

        let stats = ring.stats();
        assert_eq!(stats.frames_in_flight, 1);

        // Render the first frame before submitting another to avoid deadlock
        ring.try_get_render_frame().unwrap().complete();

        ring.begin_frame();
        ring.end_frame();

        let stats = ring.stats();
        assert_eq!(stats.frames_in_flight, 1); // One frame rendered, one pending
    }

    #[test]
    fn test_frame_submitter_renderer() {
        let (submitter, renderer) = create_frame_pipeline();

        submitter.begin_frame();
        submitter.submit(RenderCommand::SwapBuffers);
        submitter.end_frame();

        let frame = renderer.try_get_frame().unwrap();
        assert_eq!(frame.commands().len(), 1);
        frame.complete();
    }
}
