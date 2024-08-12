use std::collections::HashMap;

use strum::IntoEnumIterator;

use super::FrameStage;
use crate::system::TimeStamp;

pub struct FrameTimer {
    last_update: HashMap<FrameStage, TimeStamp>,
}

impl FrameTimer {
    pub fn new() -> Self {
        let now = TimeStamp::now();
        let last_update = FrameStage::iter().map(|stage| (stage, now)).collect();
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
