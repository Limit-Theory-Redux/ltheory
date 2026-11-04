//! GPU buffers and the material parameter arenas
//! (doc/engine/render-api-v2.md, sections 1.2 and 2).
//!
//! A material's `MaterialParams` block lives in a slice of a long-lived
//! arena buffer. Lua writes the typed struct, `mat:commit()` sends one
//! `WriteBuffer`; a bind group refers to the slice with a uniform entry and
//! the executor binds that range when the group is set. Arenas are created
//! on demand (`CreateBuffer`), slices are recycled when their material is
//! dropped.

use std::collections::HashMap;

use super::UNIFORM_ALIGN;

/// Bytes per arena buffer: 1024 slices of one `UNIFORM_ALIGN` stride.
pub const ARENA_SIZE: u32 = 256 * 1024;

/// Index of a created GPU buffer. Allocated on the main thread; the executor
/// maps it to its own buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferId(pub u32);

/// A range of an arena buffer, `UNIFORM_ALIGN`-aligned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArenaSlice {
    pub buffer: BufferId,
    pub offset: u32,
    /// Bytes reserved (the block size rounded up to `UNIFORM_ALIGN`).
    pub reserved: u32,
}

/// Main-thread allocator over the arena buffers.
#[derive(Default)]
pub struct MaterialArenas {
    /// `(buffer, bump offset)` of the arena slices are currently carved from.
    open: Option<(BufferId, u32)>,
    /// Freed slices by reserved size, ready for reuse.
    free: HashMap<u32, Vec<ArenaSlice>>,
    next_buffer: u32,
}

impl MaterialArenas {
    pub fn new() -> Self {
        Self::default()
    }

    /// A slice for a block of `size` bytes, and the new arena buffer if one
    /// had to be opened (the caller must send `CreateBuffer` for it before
    /// the slice is used).
    pub fn alloc(&mut self, size: u32) -> (ArenaSlice, Option<BufferId>) {
        let reserved = size.max(1).next_multiple_of(UNIFORM_ALIGN);
        assert!(
            reserved <= ARENA_SIZE,
            "material parameter block of {size} bytes exceeds the {ARENA_SIZE} byte arena"
        );
        if let Some(slice) = self.free.get_mut(&reserved).and_then(|v| v.pop()) {
            return (slice, None);
        }
        let mut created = None;
        let (buffer, offset) = match self.open {
            Some((buffer, offset)) if offset + reserved <= ARENA_SIZE => (buffer, offset),
            _ => {
                let buffer = BufferId(self.next_buffer);
                self.next_buffer += 1;
                created = Some(buffer);
                (buffer, 0)
            }
        };
        self.open = Some((buffer, offset + reserved));
        (
            ArenaSlice {
                buffer,
                offset,
                reserved,
            },
            created,
        )
    }

    /// Give a slice back (its material was dropped).
    pub fn release(&mut self, slice: ArenaSlice) {
        self.free.entry(slice.reserved).or_default().push(slice);
    }

    /// Slices currently on the free lists (tests and stats).
    pub fn free_slices(&self) -> usize {
        self.free.values().map(Vec::len).sum()
    }
}

/// What a dropped `Material` hands back to the main thread (its `Drop` has no
/// `&mut Renderer`, so it queues the release on a channel the renderer
/// drains at frame end).
#[derive(Debug)]
pub enum Release {
    Slice(ArenaSlice),
    BindGroup(super::BindGroupId),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slices_are_aligned_and_share_an_arena() {
        let mut arenas = MaterialArenas::new();
        let (a, created) = arenas.alloc(80);
        assert_eq!(created, Some(BufferId(0)));
        assert_eq!((a.offset, a.reserved), (0, UNIFORM_ALIGN));
        let (b, created) = arenas.alloc(300);
        assert_eq!(created, None);
        assert_eq!((b.buffer, b.offset, b.reserved), (BufferId(0), 256, 512));
    }

    #[test]
    fn a_full_arena_opens_the_next_one() {
        let mut arenas = MaterialArenas::new();
        let slots = ARENA_SIZE / UNIFORM_ALIGN;
        for _ in 0..slots {
            arenas.alloc(16);
        }
        let (slice, created) = arenas.alloc(16);
        assert_eq!(created, Some(BufferId(1)));
        assert_eq!((slice.buffer, slice.offset), (BufferId(1), 0));
    }

    #[test]
    fn released_slices_are_reused_by_size() {
        let mut arenas = MaterialArenas::new();
        let (a, _) = arenas.alloc(80);
        let (_b, _) = arenas.alloc(80);
        arenas.release(a);
        assert_eq!(arenas.free_slices(), 1);
        let (c, created) = arenas.alloc(64);
        assert_eq!((c, created), (a, None));
        // A different size does not take it.
        let (d, _) = arenas.alloc(300);
        assert_eq!(d.reserved, 512);
        assert_eq!(arenas.free_slices(), 0);
    }
}
