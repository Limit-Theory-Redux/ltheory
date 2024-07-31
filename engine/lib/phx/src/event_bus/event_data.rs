use super::{EventPayload, FrameStage};

#[derive(Debug, Clone)]
pub struct EventData {
    pub delta_time: f64,
    pub frame_stage: FrameStage,
    pub tunnel_id: u32,
    pub payload: Option<EventPayload>,
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
