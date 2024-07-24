use std::collections::HashMap;

use strum::IntoEnumIterator;

use crate::system::TimeStamp;
use super::FrameStage;

pub struct FrameTimer {
    pub(crate) last_update: HashMap<FrameStage, TimeStamp>,
}

impl FrameTimer {
    pub fn new() -> Self {
        let mut last_update = HashMap::new();
        let now = TimeStamp::now();
        for stage in FrameStage::iter() {
            last_update.insert(stage, now);
        }
        FrameTimer { last_update }
    }

    pub fn update(&mut self, stage: FrameStage) -> f64 {
        let now = TimeStamp::now();
        let last_time = self.last_update.get(&stage).cloned().unwrap_or(now);
        let delta = last_time.get_elapsed();
        self.last_update.insert(stage, now);
        delta
    }
}