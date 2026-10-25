//! Common code for threaded and immediate renderers.

use crate::render::{Renderer, ResourceHandle, ResourceId};

impl Renderer {
    /// Mint a new GPU resource: a unique `ResourceId` bundled with the means
    /// to destroy it (see `ResourceHandle`). This is the only way to obtain
    /// either, so a resource can never exist without its destructor wired up.
    pub fn create_resource(&mut self) -> ResourceHandle {
        let id = ResourceId(self.data.next_resource_id);
        self.data.next_resource_id += 1;
        ResourceHandle::new(id, self.data.destroy_tx.clone())
    }
}
