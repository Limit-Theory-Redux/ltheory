use super::FrameStage;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MessageRequestCache {
    pub(crate) frame_stage: FrameStage,
    pub(crate) priority: i32,
    pub(crate) event_name: String,
    pub(crate) stay_alive: bool,
    pub(crate) for_entity_id: Option<u64>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MessageRequest {
    pub(crate) priority: i32,
    pub(crate) event_name: String,
    pub(crate) stay_alive: bool,
    pub(crate) for_entity_id: Option<u64>,
}

impl From<MessageRequestCache> for MessageRequest {
    fn from(cache: MessageRequestCache) -> Self {
        MessageRequest {
            priority: cache.priority,
            event_name: cache.event_name,
            stay_alive: cache.stay_alive,
            for_entity_id: cache.for_entity_id,
        }
    }
}

impl Ord for MessageRequest {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority comes first
        self.priority
            .cmp(&other.priority)
            .then_with(|| self.event_name.cmp(&other.event_name))
    }
}

impl PartialOrd for MessageRequest {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}