use super::{EventPayload, FrameStage};

#[derive(Debug, Clone)]
pub struct EventData {
    pub(super) delta_time: f64,
    pub(super) frame_stage: FrameStage,
    pub(super) tunnel_id: u32,
    pub(super) payload: Option<EventPayload>,
}

#[luajit_ffi_gen::luajit_ffi]
impl EventData {
    pub fn get_delta_time(&self) -> f64 {
        self.delta_time
    }

    pub fn get_frame_stage(&self) -> FrameStage {
        self.frame_stage
    }

    pub fn get_tunnel_id(&self) -> u32 {
        self.tunnel_id
    }

    pub fn get_payload(&self) -> Option<&EventPayload> {
        self.payload.as_ref()
    }
}
