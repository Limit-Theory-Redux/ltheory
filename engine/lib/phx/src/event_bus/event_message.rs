use super::{EventId, FrameStage, Subscriber, TunnelId};

#[derive(Debug, Clone)]
pub struct EventMessage {
    id: EventId,
    name: String,
    frame_stage: FrameStage,
    subscribers: Vec<Subscriber>,
    next_index: usize,
}

impl EventMessage {
    pub fn new(id: EventId, name: &str, frame_stage: FrameStage) -> Self {
        Self {
            id,
            name: name.into(),
            frame_stage,
            subscribers: vec![],
            next_index: 0,
        }
    }

    pub fn id(&self) -> EventId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn frame_stage(&self) -> FrameStage {
        self.frame_stage
    }

    pub fn add_subscriber(&mut self, subscriber: Subscriber) {
        self.subscribers.push(subscriber);
    }

    pub fn remove_subscriber(&mut self, tunnel_id: TunnelId) {
        self.subscribers
            .retain(|subscriber| subscriber.tunnel_id() != tunnel_id);
    }

    pub fn next_subscriber(&mut self) -> Option<&Subscriber> {
        let result = self.subscribers.get(self.next_index);
        if result.is_some() {
            self.next_index += 1;
        } else {
            self.next_index = 0;
        }
        result
    }
}
