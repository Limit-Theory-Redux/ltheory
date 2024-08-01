use super::{FrameStage, Subscriber};

#[derive(Debug, Clone)]
pub struct Event {
    name: String,
    priority: i32,
    frame_stage: FrameStage,
    subscribers: Vec<Subscriber>,
    next_index: usize,
}

impl Event {
    pub fn new(name: String, priority: i32, frame_stage: FrameStage) -> Self {
        Self {
            name,
            priority,
            frame_stage,
            subscribers: vec![],
            next_index: 0,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn priority(&self) -> i32 {
        self.priority
    }

    pub fn frame_stage(&self) -> FrameStage {
        self.frame_stage
    }

    pub fn add_subscriber(&mut self, subscriber: Subscriber) {
        self.subscribers.push(subscriber);
        self.subscribers.sort_by_key(|a| a.id());
    }

    pub fn remove_subscriber(&mut self, tunnel_id: u32) {
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
