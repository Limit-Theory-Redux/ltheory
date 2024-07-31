use super::FrameStage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subscriber {
    pub(super) id: u32,
    pub(super) tunnel_id: u32,
    pub(super) entity_id: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub(super) name: String,
    pub(super) priority: i32,
    pub(super) frame_stage: FrameStage,
    pub(super) subscribers: Vec<Subscriber>,
    pub(super) processed_subscribers: Vec<usize>,
}

impl Event {
    pub fn get_next_subscriber(&mut self) -> Option<&Subscriber> {
        for (i, subscriber) in self.subscribers.iter().enumerate() {
            if !self.processed_subscribers.contains(&i) {
                self.processed_subscribers.push(i);
                return Some(subscriber);
            }
        }
        None
    }

    pub fn reset_processed_subscribers(&mut self) {
        self.processed_subscribers.clear();
    }
}
