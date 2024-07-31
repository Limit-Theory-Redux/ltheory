use super::{EventPayload, FrameStage};

#[derive(Debug, Clone, PartialEq)]
pub struct MessageRequestCache {
    pub(super) frame_stage: FrameStage,
    pub(super) request: MessageRequest,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MessageRequest {
    pub(super) priority: i32,
    pub(super) event_name: String,
    pub(super) stay_alive: bool,
    pub(super) for_entity_id: Option<u64>,
    pub(super) payload: Option<EventPayload>,
}

impl From<MessageRequestCache> for MessageRequest {
    fn from(cache: MessageRequestCache) -> Self {
        cache.request.clone()
    }
}
