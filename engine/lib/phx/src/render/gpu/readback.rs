//! Texture readback (render API v2, S8), shared by both backends.
//!
//! Two flavours, both format-agnostic: the payload is in the layout of a
//! [`TexFormat`] (tightly packed rows, row 0 first), never a GL pixel/data
//! pair.
//!
//! * `Renderer:readSync(view, x, y, w, h, fmt)` stalls until the data is
//!   there. It is for screenshots, tests and tools; nothing in a frame should
//!   call it.
//! * `Renderer:readAsync(...)` returns a [`ReadbackTicket`] at once. The
//!   executor copies the pixels into a transfer buffer (GL: a pixel pack
//!   buffer and a fence, wgpu: a mappable buffer), polls it once per frame at
//!   `BeginFrame` without ever waiting, and fills the ticket's slot when the
//!   GPU is done, typically two or three frames later.

use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

use crate::render::{
    DataFormat, PixelFormat, Renderer, ResourceId, TexRegion, TexView, ViewDim, face_layer,
    format_for_layout,
};
use crate::system::Bytes;

/// What a readback copies from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadSource {
    /// A texture of the executor.
    Texture(ResourceId),
    /// The default framebuffer (what the window shows, GL convention: row 0
    /// is the bottom row).
    Backbuffer,
}

const PENDING: u8 = 0;
const READY: u8 = 1;
const FAILED: u8 = 2;

/// Where an asynchronous readback lands. The executor fills it from the
/// render thread; the main thread polls `state` through the ticket.
#[derive(Debug)]
pub struct ReadbackSlot {
    state: AtomicU8,
    data: std::sync::Mutex<Vec<u8>>,
}

impl ReadbackSlot {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            state: AtomicU8::new(PENDING),
            data: std::sync::Mutex::new(Vec::new()),
        })
    }

    /// Executor side: the pixels arrived.
    pub fn complete(&self, bytes: Vec<u8>) {
        if let Ok(mut data) = self.data.lock() {
            *data = bytes;
        }
        self.state.store(READY, Ordering::Release);
    }

    /// Executor side: the read could not be done (unsupported format, lost
    /// device, no context).
    pub fn fail(&self) {
        self.state.store(FAILED, Ordering::Release);
    }

    pub fn is_pending(&self) -> bool {
        self.state.load(Ordering::Acquire) == PENDING
    }

    fn is_failed(&self) -> bool {
        self.state.load(Ordering::Acquire) == FAILED
    }

    fn bytes(&self) -> Vec<u8> {
        self.data.lock().map(|d| d.clone()).unwrap_or_default()
    }
}

/// The region of `view` that `x, y, w, h` (texel coordinates of the view's
/// mip level, row 0 first) addresses, clamped to the level. For a face or
/// layer view the face or layer is the region's z.
pub fn view_region(view: &TexView, x: i32, y: i32, w: i32, h: i32) -> TexRegion {
    let [ew, eh] = view.extent;
    let layer = match view.dim {
        ViewDim::D2Layer(z) => z as u32,
        ViewDim::CubeFace(face) => face_layer(face),
        _ => 0,
    };
    let x = x.clamp(0, ew as i32 - 1) as u32;
    let (y, h) = if view.dim == ViewDim::D1 {
        (0, 1)
    } else {
        (y.clamp(0, eh as i32 - 1) as u32, h)
    };
    TexRegion {
        level: view.base_mip as u32,
        origin: [x, y, layer],
        size: [
            (w.max(1) as u32).min(ew - x),
            (h.max(1) as u32).min(eh - y),
            1,
        ],
    }
}

/// A readback that may still be in flight. Lua: `:ready()` is true once the
/// read finished (successfully or not: `:failed()`), `:data()` then returns
/// the pixels as `Bytes` in the requested `TexFormat` layout, and `:free()`
/// gives the ticket up (the garbage collector does it too).
#[derive(Debug)]
pub struct ReadbackTicket {
    slot: Option<Arc<ReadbackSlot>>,
    size: [u32; 2],
}

impl ReadbackTicket {
    pub fn new(slot: Arc<ReadbackSlot>, region: &TexRegion) -> Self {
        Self {
            slot: Some(slot),
            size: [region.size[0], region.size[1]],
        }
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl ReadbackTicket {
    /// The read is over: `data` has the pixels, or the read failed (`failed`).
    /// Poll this once per frame; it never blocks.
    pub fn ready(&self) -> bool {
        self.slot.as_ref().is_some_and(|s| !s.is_pending())
    }

    /// The read could not be done; `data` is empty.
    pub fn failed(&self) -> bool {
        self.slot.as_ref().is_none_or(|s| s.is_failed())
    }

    /// The pixels in the requested format, rows from the first up (a copy;
    /// empty until `ready`, and after a failure).
    pub fn data(&self) -> Bytes {
        match &self.slot {
            Some(slot) if !slot.is_pending() && !slot.is_failed() => Bytes::from_vec(slot.bytes()),
            _ => Bytes::from_vec(Vec::new()),
        }
    }

    pub fn get_width(&self) -> i32 {
        self.size[0] as i32
    }

    pub fn get_height(&self) -> i32 {
        self.size[1] as i32
    }

    /// Give the ticket up. `data` is empty afterwards; the pixels of a read
    /// still in flight are dropped when they arrive.
    pub fn release(&mut self) {
        self.slot = None;
    }
}

/// `bytes` as `count` values of `T`, zero-filled where the read came up short.
pub fn bytes_to_vec<T: Clone + Default>(bytes: &[u8], count: usize) -> Vec<T> {
    let mut data = vec![T::default(); count];
    let byte_len = (count * std::mem::size_of::<T>()).min(bytes.len());
    #[allow(unsafe_code)] // plain data, bounded by both buffers
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), data.as_mut_ptr() as *mut u8, byte_len);
    }
    data
}

/// Read `region` of texture `id` (synchronously) in the layout `pixel` x
/// `data` and return it as values of `T`: what `Tex*::get_data` does. Layouts
/// without a `TexFormat` (three components, BGR) read as zeros.
pub fn read_layout<T: Clone + Default>(
    r: &mut Renderer,
    id: ResourceId,
    region: TexRegion,
    pixel: PixelFormat,
    data: DataFormat,
) -> Vec<T> {
    let count = region.texels()
        * PixelFormat::components(pixel) as usize
        * DataFormat::get_size(data) as usize
        / std::mem::size_of::<T>();
    let bytes = match format_for_layout(pixel, data) {
        Some(format) => r.read_texture_sync(ReadSource::Texture(id), region, format),
        None => {
            tracing::warn!("get_data: no texture format for {pixel:?} x {data:?}");
            Vec::new()
        }
    };
    bytes_to_vec(&bytes, count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::CubeFace;

    #[test]
    fn tickets_follow_their_slot() {
        let slot = ReadbackSlot::new();
        let region = TexRegion::rect(0, 0, 2, 1);
        let mut ticket = ReadbackTicket::new(slot.clone(), &region);
        assert!(!ticket.ready());
        assert_eq!(ticket.data().len(), 0);
        slot.complete(vec![1, 2, 3, 4, 5, 6, 7, 8]);
        assert!(ticket.ready() && !ticket.failed());
        assert_eq!(ticket.data().len(), 8);
        ticket.release();
        assert_eq!(ticket.data().len(), 0);

        let failed = ReadbackSlot::new();
        let ticket = ReadbackTicket::new(failed.clone(), &region);
        failed.fail();
        assert!(ticket.ready() && ticket.failed());
        assert_eq!(ticket.data().len(), 0);
    }

    #[test]
    fn view_regions_are_clamped_to_the_level() {
        let view = TexView::new(ResourceId(1), ViewDim::D2, 2, [16, 8]);
        let r = view_region(&view, 12, 6, 10, 10);
        assert_eq!(r.level, 2);
        assert_eq!(r.origin, [12, 6, 0]);
        assert_eq!(r.size, [4, 2, 1]);
        let face = TexView::new(ResourceId(1), ViewDim::CubeFace(CubeFace::NY), 0, [4, 4]);
        assert_eq!(view_region(&face, 0, 0, 4, 4).origin, [0, 0, 3]);
        let one_d = TexView::new(ResourceId(1), ViewDim::D1, 0, [8, 1]);
        assert_eq!(view_region(&one_d, 2, 5, 3, 7).size, [3, 1, 1]);
    }
}
