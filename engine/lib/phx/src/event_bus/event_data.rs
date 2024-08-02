use super::{EventPayload, FrameStage, TunnelId};

#[derive(Debug, Clone)]
pub struct EventData {
    delta_time: f64,
    frame_stage: FrameStage,
    tunnel_id: TunnelId,
    payload: Option<EventPayload>,
}

impl EventData {
    pub fn new(
        delta_time: f64,
        frame_stage: FrameStage,
        tunnel_id: TunnelId,
        payload: Option<EventPayload>,
    ) -> Self {
        Self {
            delta_time,
            frame_stage,
            tunnel_id,
            payload,
        }
    }
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
