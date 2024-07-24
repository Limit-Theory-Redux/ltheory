use super::FrameStage;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Subscriber {
    pub(crate) id: u32,
    pub(crate) tunnel_id: u32,
    pub(crate) entity_id: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub name: String,
    pub priority: i32,
    pub frame_stage: FrameStage,
    pub subscribers: Vec<Subscriber>,
    pub processed_subscribers: Vec<usize>,
}

impl Event {
    pub fn get_next_subscriber(&mut self) -> Option<&Subscriber> {
        for i in 0..self.subscribers.len() {
            if !self.processed_subscribers.contains(&i) {
                self.processed_subscribers.push(i);
                return self.subscribers.get(i);
            }
        }
        None
    }

    pub fn reset_processed_subscribers(&mut self) {
        self.processed_subscribers.clear();
    }
}
