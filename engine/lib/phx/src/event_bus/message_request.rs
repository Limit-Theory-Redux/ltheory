use super::{EventPayload, FrameStage};

#[derive(Debug, Clone, PartialEq)]
pub struct MessageRequestCache {
    pub(crate) frame_stage: FrameStage,
    pub(crate) priority: i32,
    pub(crate) event_name: String,
    pub(crate) stay_alive: bool,
    pub(crate) for_entity_id: Option<u64>,
    pub(crate) payload: Option<EventPayload>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MessageRequest {
    pub(crate) priority: i32,
    pub(crate) event_name: String,
    pub(crate) stay_alive: bool,
    pub(crate) for_entity_id: Option<u64>,
    pub(crate) payload: Option<EventPayload>,
}

impl From<MessageRequestCache> for MessageRequest {
    fn from(cache: MessageRequestCache) -> Self {
        MessageRequest {
            priority: cache.priority,
            event_name: cache.event_name,
            stay_alive: cache.stay_alive,
            for_entity_id: cache.for_entity_id,
            payload: cache.payload,
        }
    }
}
