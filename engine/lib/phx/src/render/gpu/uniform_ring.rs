//! Main-thread uniform ring allocator (doc/engine/render-api-v2.md, 1.2).
//!
//! Per-pass and per-draw uniform blocks are written into CPU staging that
//! the executor uploads before it runs the pass commands that reference
//! them. In S3 this is a single frame-slotted allocator; S4 extends it with
//! chunk recycling, GL slot fences and the vertex ring.

/// Bytes per staging chunk. A chunk maps to exactly one GPU buffer.
pub const CHUNK_SIZE: usize = 256 * 1024;
/// `max(GL UNIFORM_BUFFER_OFFSET_ALIGNMENT, wgpu minimum)`.
pub const UNIFORM_ALIGN: u32 = 256;
pub const MAX_FRAMES_IN_FLIGHT: usize = 3;

/// Where a block lives: the chunk (GPU buffer) within the frame slot and the
/// byte offset inside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct RingOffset {
    pub buffer: u16,
    pub offset: u32,
}

/// A run of bytes to upload to `at` before the pass commands run.
#[derive(Debug, Clone)]
pub struct RingChunk {
    pub at: RingOffset,
    pub bytes: Vec<u8>,
}

/// A filled chunk waiting for the next flush. Its memory stays alive (and
/// writable through earlier `alloc` pointers) until `take_pending`.
struct ClosedChunk {
    index: u16,
    data: Vec<u8>,
    sent: usize,
}

pub struct UniformRing {
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
    /// Recycled chunk memory.
    spare: Vec<Vec<u8>>,
}

impl UniformRing {
    pub fn new() -> Self {
        Self {
            slot: 0,
            open: Vec::with_capacity(CHUNK_SIZE),
            chunk: 0,
            sent: 0,
            closed: Vec::new(),
            spare: Vec::new(),
        }
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
            "UniformRing::begin_frame with unsent data"
        );
        self.slot = (frame_index % MAX_FRAMES_IN_FLIGHT as u64) as usize;
        self.open.clear();
        self.chunk = 0;
        self.sent = 0;
    }

    /// Allocate `size` zeroed bytes aligned to `UNIFORM_ALIGN`. Never
    /// flushes: when the chunk is full it is set aside and a fresh one
    /// opens, so earlier pointers stay valid (and their writes still reach
    /// the GPU) until the caller's next flush point.
    pub fn alloc(&mut self, size: u32) -> (RingOffset, *mut u8) {
        let size = size.max(1) as usize;
        assert!(
            size <= CHUNK_SIZE,
            "uniform allocation of {size} bytes exceeds the {CHUNK_SIZE} byte ring chunk"
        );
        let align = UNIFORM_ALIGN as usize;
        let mut start = self.open.len().next_multiple_of(align);
        if start + size > CHUNK_SIZE {
            self.close_chunk();
            start = 0;
        }
        // Zero-fill up to the end of the allocation. Capacity is fixed, so
        // this never reallocates.
        self.open.resize(start + size, 0);
        debug_assert_eq!(self.open.capacity(), CHUNK_SIZE);
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
            .unwrap_or_else(|| Vec::with_capacity(CHUNK_SIZE));
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
                out.push(RingChunk {
                    at: RingOffset {
                        buffer: closed.index,
                        offset: closed.sent as u32,
                    },
                    bytes: closed.data[closed.sent..].to_vec(),
                });
            }
            closed.data.clear();
            self.spare.push(closed.data);
        }
        if self.sent < self.open.len() {
            out.push(RingChunk {
                at: RingOffset {
                    buffer: self.chunk,
                    offset: self.sent as u32,
                },
                bytes: self.open[self.sent..].to_vec(),
            });
            self.sent = self.open.len();
        }
        out
    }

    pub fn has_pending(&self) -> bool {
        !self.closed.is_empty() || self.sent < self.open.len()
    }
}

impl Default for UniformRing {
    fn default() -> Self {
        Self::new()
    }
}

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
        assert_eq!(chunks[0].bytes[0], 7);
        assert_eq!(chunks[0].bytes.len(), 769);
        assert!(!ring.has_pending());
    }

    #[test]
    fn take_pending_sends_only_new_bytes() {
        let mut ring = UniformRing::new();
        ring.alloc_copy(&[1u8; 8]);
        let first = ring.take_pending();
        assert_eq!(first[0].bytes.len(), 8);
        let at = ring.alloc_copy(&[2u8; 8]);
        assert_eq!(at.offset, 256);
        // The run is contiguous: it starts where the first one ended and
        // includes the alignment gap before the new block.
        let second = ring.take_pending();
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].at.offset, 8);
        assert_eq!(second[0].bytes.len(), 256);
        assert_eq!(&second[0].bytes[248..], &[2u8; 8]);
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
        assert_eq!(chunks[1].at.buffer, 1);
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
