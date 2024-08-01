#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subscriber {
    id: u32,
    tunnel_id: u32,
    entity_id: Option<u64>,
}

impl Subscriber {
    pub fn new(id: u32, tunnel_id: u32, entity_id: Option<u64>) -> Self {
        Self {
            id,
            tunnel_id,
            entity_id,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn tunnel_id(&self) -> u32 {
        self.tunnel_id
    }

    pub fn entity_id(&self) -> Option<u64> {
        self.entity_id
    }
}
