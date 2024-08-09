use super::{EntityId, TunnelId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subscriber {
    tunnel_id: TunnelId,
    entity_id: Option<EntityId>,
}

impl Subscriber {
    pub fn new(tunnel_id: TunnelId, entity_id: Option<EntityId>) -> Self {
        Self {
            tunnel_id,
            entity_id,
        }
    }

    pub fn tunnel_id(&self) -> TunnelId {
        self.tunnel_id
    }

    pub fn entity_id(&self) -> Option<EntityId> {
        self.entity_id
    }
}
