use crossbeam::channel::Sender;

use crate::render::ResourceId;

/// An owning handle to an executor-side GPU resource.
///
/// A destructor can never reach `&mut Renderer`, so a resource cannot submit
/// its own `DestroyResource` command when it goes away. This type closes that
/// gap: it pairs the `ResourceId` with a producer end of the `Renderer`'s
/// destroy queue, and its `Drop` enqueues the id. The `Renderer` drains the
/// queue once per frame (see `end_frame_triple_buffered`).
///
/// The queue itself lives *only* in the `Renderer` - what a resource holds is
/// a send handle, not a copy of the queue.
///
/// Obtainable only via `Renderer::create_resource()`, which is also the only
/// way to mint a `ResourceId`. That is deliberate: minting an id and arranging
/// for its destruction can't drift apart into two steps one of which is
/// forgotten at a new creation site.
///
/// Deliberately **not** `Clone` - two handles for one id would enqueue two
/// destroys. Share the owning wrapper (`Rf<Tex2DShared>` etc.) instead, so the
/// handle drops exactly once, when the last clone does.
#[derive(Debug)]
pub struct ResourceHandle {
    id: ResourceId,
    destroy_tx: Sender<ResourceId>,
}

impl ResourceHandle {
    /// Only `Renderer::create_resource` constructs these - see the type docs.
    pub(super) fn new(id: ResourceId, destroy_tx: Sender<ResourceId>) -> Self {
        Self { id, destroy_tx }
    }

    pub fn id(&self) -> ResourceId {
        self.id
    }
}

impl Drop for ResourceHandle {
    fn drop(&mut self) {
        // The queue is unbounded, so this never blocks - which matters in a
        // destructor. It only fails once the `Renderer` (and with it the GL
        // context that owns the resource) is gone, in which case there is
        // nothing left to destroy.
        let _ = self.destroy_tx.send(self.id);
    }
}
