//! Main-thread staging rings (doc/engine/render-api-v2.md, 1.2): the uniform
//! ring (per-pass and per-draw blocks, 256-byte aligned) and the vertex ring
//! (instance data and instanced index lists) share one allocator,
//! [`StagingRing`].
//!
//! The main thread allocates and writes into CPU staging chunks of fixed
//! capacity (so write pointers stay valid), the executor uploads them before
//! it runs the pass commands that reference them, and then gives the memory
//! back: threaded mode returns the chunks over a channel, immediate mode
//! inline. A chunk that filled up travels as one owned `Vec` (no copy); only
//! the filled part of the chunk still being written is copied out, into a
//! small pooled `Vec`.

/// Bytes per uniform staging chunk. A chunk maps to exactly one GPU buffer.
pub const CHUNK_SIZE: usize = 256 * 1024;
/// Bytes per vertex-ring chunk (the instance data of one draw must fit in one).
pub const VERTEX_CHUNK_SIZE: usize = 1024 * 1024;
/// `max(GL UNIFORM_BUFFER_OFFSET_ALIGNMENT, wgpu minimum)`.
pub const UNIFORM_ALIGN: u32 = 256;
/// Alignment of vertex-ring allocations (covers `vec4` attributes).
pub const VERTEX_ALIGN: u32 = 16;
pub const MAX_FRAMES_IN_FLIGHT: usize = 3;
/// Recycled partial-run buffers kept around.
const RUN_POOL_LIMIT: usize = 64;

/// Where a block lives: the chunk (GPU buffer) within the frame slot and the
/// byte offset inside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct RingOffset {
    pub buffer: u16,
    pub offset: u32,
}

/// A run of bytes to upload before the pass commands run: `bytes[skip..]`
/// goes to buffer `at.buffer` at byte offset `at.offset`. The executor hands
/// `bytes` back for recycling once uploaded.
#[derive(Debug, Clone)]
pub struct RingChunk {
    pub at: RingOffset,
    pub bytes: Vec<u8>,
    pub skip: u32,
}

impl RingChunk {
    /// The bytes to upload.
    pub fn data(&self) -> &[u8] {
        &self.bytes[self.skip as usize..]
    }
}

/// A chunk's memory on its way back to the main thread.
#[derive(Debug)]
pub struct ReturnedChunk {
    pub vertex: bool,
    pub bytes: Vec<u8>,
}

/// A filled chunk waiting for the next flush. Its memory stays alive (and
/// writable through earlier `alloc` pointers) until `take_pending`.
struct ClosedChunk {
    index: u16,
    data: Vec<u8>,
    sent: usize,
}

/// Chunked bump allocator over CPU staging memory.
pub struct StagingRing {
    chunk_size: usize,
    align: usize,
    slot: usize,
    /// Current chunk. Its capacity is fixed, so pointers into it stay valid
    /// until `begin_frame`.
    open: Vec<u8>,
    /// Chunk index of `open` within the slot.
    chunk: u16,
    /// Bytes of `open` already handed to `take_pending`.
    sent: usize,
    /// Filled chunks, waiting for the next flush.
    closed: Vec<ClosedChunk>,
    /// Recycled chunk memory (capacity `chunk_size`).
    spare: Vec<Vec<u8>>,
    /// Recycled small buffers for partial runs.
    runs: Vec<Vec<u8>>,
    /// Bytes allocated since `begin_frame`, and in the frame before (stats).
    frame_bytes: u64,
    prev_frame_bytes: u64,
}

impl StagingRing {
    pub fn new(chunk_size: usize, align: usize) -> Self {
        Self {
            chunk_size,
            align,
            slot: 0,
            open: Vec::with_capacity(chunk_size),
            chunk: 0,
            sent: 0,
            closed: Vec::new(),
            spare: Vec::new(),
            runs: Vec::new(),
            frame_bytes: 0,
            prev_frame_bytes: 0,
        }
    }

    /// Bytes allocated during the last completed frame.
    pub fn last_frame_bytes(&self) -> u64 {
        self.prev_frame_bytes
    }

    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    /// The frame slot (`frame_index % MAX_FRAMES_IN_FLIGHT`) the next
    /// allocations belong to.
    pub fn slot(&self) -> u8 {
        self.slot as u8
    }

    /// Start a new frame in the slot for `frame_index`. Everything allocated
    /// before must have been taken with `take_pending`.
    pub fn begin_frame(&mut self, frame_index: u64) {
        debug_assert!(
            !self.has_pending(),
            "StagingRing::begin_frame with unsent data"
        );
        self.slot = (frame_index % MAX_FRAMES_IN_FLIGHT as u64) as usize;
        self.prev_frame_bytes = self.frame_bytes;
        self.frame_bytes = 0;
        self.open.clear();
        self.chunk = 0;
        self.sent = 0;
    }

    /// Allocate `size` zeroed bytes aligned to the ring's alignment. Never
    /// flushes: when the chunk is full it is set aside and a fresh one
    /// opens, so earlier pointers stay valid (and their writes still reach
    /// the GPU) until the caller's next flush point.
    pub fn alloc(&mut self, size: u32) -> (RingOffset, *mut u8) {
        let size = size.max(1) as usize;
        assert!(
            size <= self.chunk_size,
            "staging allocation of {size} bytes exceeds the {} byte ring chunk",
            self.chunk_size
        );
        self.frame_bytes += size as u64;
        let mut start = self.open.len().next_multiple_of(self.align);
        if start + size > self.chunk_size {
            self.close_chunk();
            start = 0;
        }
        // Zero-fill up to the end of the allocation. Capacity is fixed, so
        // this never reallocates.
        self.open.resize(start + size, 0);
        debug_assert_eq!(self.open.capacity(), self.chunk_size);
        let at = RingOffset {
            buffer: self.chunk,
            offset: start as u32,
        };
        // `open` never reallocates (fixed capacity) and `start..start+size`
        // is in bounds, so the pointer stays valid until the next flush.
        let ptr = self.open[start..].as_mut_ptr();
        (at, ptr)
    }

    /// Allocate and copy `bytes` in one step.
    pub fn alloc_copy(&mut self, bytes: &[u8]) -> RingOffset {
        let (at, ptr) = self.alloc(bytes.len() as u32);
        #[allow(unsafe_code)]
        // SAFETY: `ptr` points at `bytes.len()` writable bytes just allocated.
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
        }
        at
    }

    fn close_chunk(&mut self) {
        let fresh = self
            .spare
            .pop()
            .unwrap_or_else(|| Vec::with_capacity(self.chunk_size));
        let data = std::mem::replace(&mut self.open, fresh);
        self.open.clear();
        self.closed.push(ClosedChunk {
            index: self.chunk,
            data,
            sent: self.sent,
        });
        self.sent = 0;
        self.chunk += 1;
    }

    /// Everything allocated since the last call, as upload runs. After this
    /// the bytes behind earlier pointers are on their way to the GPU;
    /// writing through them no longer has an effect.
    pub fn take_pending(&mut self) -> Vec<RingChunk> {
        let mut out = Vec::new();
        for mut closed in std::mem::take(&mut self.closed) {
            if closed.sent < closed.data.len() {
                // The whole chunk moves (no copy); only `sent..` uploads.
                out.push(RingChunk {
                    at: RingOffset {
                        buffer: closed.index,
                        offset: closed.sent as u32,
                    },
                    bytes: closed.data,
                    skip: closed.sent as u32,
                });
            } else {
                closed.data.clear();
                self.spare.push(closed.data);
            }
        }
        if self.sent < self.open.len() {
            let mut run = self.runs.pop().unwrap_or_default();
            run.clear();
            run.extend_from_slice(&self.open[self.sent..]);
            out.push(RingChunk {
                at: RingOffset {
                    buffer: self.chunk,
                    offset: self.sent as u32,
                },
                bytes: run,
                skip: 0,
            });
            self.sent = self.open.len();
        }
        out
    }

    /// Take back memory the executor has finished uploading.
    pub fn recycle(&mut self, mut bytes: Vec<u8>) {
        bytes.clear();
        if bytes.capacity() >= self.chunk_size {
            self.spare.push(bytes);
        } else if self.runs.len() < RUN_POOL_LIMIT {
            self.runs.push(bytes);
        }
    }

    pub fn has_pending(&self) -> bool {
        !self.closed.is_empty() || self.sent < self.open.len()
    }

    /// Recycled chunk buffers on hand.
    pub fn spare_chunks(&self) -> usize {
        self.spare.len()
    }
}

macro_rules! ring_wrapper {
    ($(#[$doc:meta])* $name:ident, $chunk:expr, $align:expr) => {
        $(#[$doc])*
        pub struct $name(StagingRing);

        impl $name {
            pub fn new() -> Self {
                Self(StagingRing::new($chunk, $align as usize))
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl std::ops::Deref for $name {
            type Target = StagingRing;
            fn deref(&self) -> &StagingRing {
                &self.0
            }
        }

        impl std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut StagingRing {
                &mut self.0
            }
        }
    };
}

ring_wrapper!(
    /// Per-pass and per-draw uniform blocks (256-byte aligned).
    UniformRing,
    CHUNK_SIZE,
    UNIFORM_ALIGN
);
ring_wrapper!(
    /// Instance data and instanced index lists.
    VertexRing,
    VERTEX_CHUNK_SIZE,
    VERTEX_ALIGN
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocations_are_aligned_and_zeroed() {
        let mut ring = UniformRing::new();
        let (a, pa) = ring.alloc(16);
        let (b, _) = ring.alloc(300);
        let (c, _) = ring.alloc(1);
        assert_eq!(a.offset, 0);
        assert_eq!(b.offset, 256);
        assert_eq!(c.offset, 768);
        #[allow(unsafe_code)]
        unsafe {
            assert_eq!(*pa, 0);
            *pa = 7;
        }
        let chunks = ring.take_pending();
        assert_eq!(chunks.len(), 1);
        assert_eq!(
            chunks[0].at,
            RingOffset {
                buffer: 0,
                offset: 0
            }
        );
        assert_eq!(chunks[0].data()[0], 7);
        assert_eq!(chunks[0].data().len(), 769);
        assert!(!ring.has_pending());
    }

    #[test]
    fn take_pending_sends_only_new_bytes() {
        let mut ring = UniformRing::new();
        ring.alloc_copy(&[1u8; 8]);
        let first = ring.take_pending();
        assert_eq!(first[0].data().len(), 8);
        let at = ring.alloc_copy(&[2u8; 8]);
        assert_eq!(at.offset, 256);
        // The run is contiguous: it starts where the first one ended and
        // includes the alignment gap before the new block.
        let second = ring.take_pending();
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].at.offset, 8);
        assert_eq!(second[0].data().len(), 256);
        assert_eq!(&second[0].data()[248..], &[2u8; 8]);
        assert!(ring.take_pending().is_empty());
    }

    #[test]
    fn full_chunk_rolls_to_the_next_buffer_and_keeps_pointers_valid() {
        let mut ring = UniformRing::new();
        let (first, p) = ring.alloc(CHUNK_SIZE as u32 - 256);
        assert_eq!(first.buffer, 0);
        let (second, _) = ring.alloc(1024);
        assert_eq!(
            second,
            RingOffset {
                buffer: 1,
                offset: 0
            }
        );
        // The first chunk's pointer is still valid memory.
        #[allow(unsafe_code)]
        unsafe {
            *p = 9;
        }
        let chunks = ring.take_pending();
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].at.buffer, 0);
        assert_eq!(chunks[0].data()[0], 9);
        assert_eq!(chunks[1].at.buffer, 1);
    }

    #[test]
    fn filled_chunks_move_without_copy_and_come_back() {
        let mut ring = UniformRing::new();
        ring.alloc(CHUNK_SIZE as u32 - 256);
        ring.alloc(1024); // closes chunk 0
        let mut chunks = ring.take_pending();
        assert_eq!(chunks.len(), 2);
        // The closed chunk is the whole buffer (capacity of a ring chunk);
        // the open run is a small copy.
        assert!(chunks[0].bytes.capacity() >= CHUNK_SIZE);
        assert!(chunks[1].bytes.capacity() < CHUNK_SIZE);
        assert_eq!(ring.spare_chunks(), 0);
        for c in chunks.drain(..) {
            ring.recycle(c.bytes);
        }
        assert_eq!(ring.spare_chunks(), 1);
        // The next closed chunk reuses the recycled memory.
        ring.alloc(CHUNK_SIZE as u32 - 256);
        ring.alloc(1024);
        assert_eq!(ring.spare_chunks(), 0);
    }

    #[test]
    fn a_partly_sent_chunk_uploads_only_the_new_part() {
        let mut ring = UniformRing::new();
        ring.alloc_copy(&[3u8; 16]);
        ring.take_pending();
        ring.alloc_copy(&[4u8; 16]); // at 256
        ring.alloc(CHUNK_SIZE as u32 - 128); // does not fit: closes chunk 0
        let chunks = ring.take_pending();
        assert_eq!(chunks.len(), 2);
        // Chunk 0 moves whole, but only the part after the first flush uploads.
        assert_eq!(chunks[0].at.offset, 16);
        assert_eq!(chunks[0].skip, 16);
        assert_eq!(chunks[0].data().len(), 256);
        assert_eq!(chunks[0].data()[240], 4);
        assert_eq!(chunks[1].at.buffer, 1);
    }

    #[test]
    fn vertex_ring_aligns_to_16_and_holds_large_runs() {
        let mut ring = VertexRing::new();
        let (a, _) = ring.alloc(4);
        let (b, _) = ring.alloc(84 * 3000);
        assert_eq!(a.offset, 0);
        assert_eq!(b.offset, 16);
        assert!(ring.chunk_size() >= 84 * 3000);
    }

    #[test]
    fn slots_follow_the_frame_index() {
        let mut ring = UniformRing::new();
        ring.begin_frame(4);
        assert_eq!(ring.slot(), 1);
        assert!(
            ring.alloc(8).0
                == RingOffset {
                    buffer: 0,
                    offset: 0
                }
        );
        ring.take_pending();
        ring.begin_frame(5);
        assert_eq!(ring.slot(), 2);
    }
}
